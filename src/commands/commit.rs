use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::Command;

use crate::cli::GlobalOptions;
use crate::config::Config;
use crate::git;
use crate::providers;
use crate::utils;
use crate::utils::hooks::{run_hooks, write_temp_commit_file, HookOptions};

/// Tokens reserved for prompt overhead when chunking diffs
const PROMPT_OVERHEAD_TOKENS: usize = 500;

pub async fn execute(options: GlobalOptions) -> Result<()> {
    // Ensure we're in a git repository
    git::assert_git_repo()?;

    // Load and validate configuration
    let mut config = load_and_validate_config()?;

    // Determine effective generate count (CLI > config > default), clamped to 1-5
    let generate_count = options
        .generate_count
        .max(config.generate_count.unwrap_or(1))
        .clamp(1, 5);

    // Prepare the diff for processing
    let (final_diff, token_count) = prepare_diff(&mut config)?;

    // If --show-prompt flag is set, just show the prompt and exit
    if options.show_prompt {
        display_prompt(&config, &final_diff, options.context.as_deref(), options.full_gitmoji);
        return Ok(());
    }

    // Run pre-generation hooks
    if !options.no_pre_hooks {
        run_pre_gen_hooks(&config, token_count, options.context.as_deref())?;
    }

    // Generate commit message(s)
    let messages = generate_commit_messages(
        &config,
        &final_diff,
        options.context.as_deref(),
        options.full_gitmoji,
        generate_count,
    )
    .await?;

    if messages.is_empty() {
        anyhow::bail!("Failed to generate any commit messages");
    }

    // Handle clipboard mode
    if options.clipboard {
        return handle_clipboard_mode(&messages);
    }

    // Handle print mode
    if options.print_message {
        print!("{}", messages[0]);
        return Ok(());
    }

    // Run pre-commit hooks on first message
    let mut final_message = messages[0].clone();
    if !options.no_pre_hooks {
        final_message = run_pre_commit_hooks(&config, &final_message)?;
    }

    // Display messages and handle commit action
    display_commit_messages(&messages);
    handle_commit_action(&options, &config, &messages, &mut final_message).await
}

/// Load configuration and apply commitlint rules
fn load_and_validate_config() -> Result<Config> {
    let mut config = Config::load()?;
    config.load_with_commitlint()?;
    config.apply_commitlint_rules()?;
    Ok(config)
}

/// Prepare the diff for processing: get staged changes, apply filters, chunk if needed
fn prepare_diff(config: &Config) -> Result<(String, usize)> {
    // Check for staged files or changes
    let staged_files = git::get_staged_files()?;
    let changed_files = if staged_files.is_empty() {
        git::get_changed_files()?
    } else {
        staged_files
    };

    if changed_files.is_empty() {
        anyhow::bail!("No changes to commit");
    }

    // If no staged files, ask user which files to stage
    let files_to_stage = if git::get_staged_files()?.is_empty() {
        select_files_to_stage(&changed_files)?
    } else {
        vec![]
    };

    // Stage selected files
    if !files_to_stage.is_empty() {
        git::stage_files(&files_to_stage)?;
    }

    // Get the diff of staged changes
    let diff = git::get_staged_diff()?;
    if diff.is_empty() {
        anyhow::bail!("No staged changes to commit");
    }

    // Apply .rcoignore if it exists
    let diff = filter_diff_by_rcoignore(&diff)?;

    // Check if diff became empty after filtering
    if diff.trim().is_empty() {
        anyhow::bail!("No changes to commit after applying .rcoignore filters");
    }

    // Check if diff is too large - implement chunking if needed
    let max_tokens = config.tokens_max_input.unwrap_or(4096);
    let token_count = utils::token::estimate_tokens(&diff)?;

    // If diff is too large, chunk it
    let final_diff = if token_count > max_tokens {
        println!(
            "{}",
            format!(
                "The diff is too large ({} tokens). Splitting into chunks...",
                token_count
            )
            .yellow()
        );
        chunk_diff(&diff, max_tokens)?
    } else {
        diff
    };

    // Check if diff is empty after chunking
    if final_diff.trim().is_empty() {
        anyhow::bail!(
            "Diff is empty after processing. This may indicate all files were excluded by .rcoignore."
        );
    }

    Ok((final_diff, token_count))
}

/// Display the prompt that would be sent to AI
fn display_prompt(config: &Config, diff: &str, context: Option<&str>, full_gitmoji: bool) {
    let prompt = config.get_effective_prompt(diff, context, full_gitmoji);
    println!("\n{}", "Prompt that would be sent to AI:".green().bold());
    println!("{}", "═".repeat(60).dimmed());
    println!("{}", prompt);
    println!("{}", "═".repeat(60).dimmed());
}

/// Run pre-generation hooks
fn run_pre_gen_hooks(config: &Config, token_count: usize, context: Option<&str>) -> Result<()> {
    if let Some(hooks) = config.pre_gen_hook.clone() {
        let envs = vec![
            ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
            (
                "RCO_MAX_TOKENS",
                (config.tokens_max_input.unwrap_or(4096)).to_string(),
            ),
            ("RCO_DIFF_TOKENS", token_count.to_string()),
            ("RCO_CONTEXT", context.unwrap_or_default().to_string()),
            (
                "RCO_PROVIDER",
                config.ai_provider.clone().unwrap_or_default(),
            ),
            ("RCO_MODEL", config.model.clone().unwrap_or_default()),
        ];
        run_hooks(HookOptions {
            name: "pre-gen",
            commands: hooks,
            strict: config.hook_strict.unwrap_or(true),
            timeout: std::time::Duration::from_millis(config.hook_timeout_ms.unwrap_or(30000)),
            envs,
        })?;
    }
    Ok(())
}

/// Handle clipboard mode - copy message to clipboard and exit
fn handle_clipboard_mode(messages: &[String]) -> Result<()> {
    let selected = if messages.len() == 1 {
        0
    } else {
        select_message_variant(messages)?
    };
    copy_to_clipboard(&messages[selected])?;
    println!("{}", "✅ Commit message copied to clipboard!".green());
    Ok(())
}

/// Display the generated commit message(s)
fn display_commit_messages(messages: &[String]) {
    if messages.len() == 1 {
        println!("\n{}", "Generated commit message:".green().bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", messages[0]);
        println!("{}", "─".repeat(50).dimmed());
    } else {
        println!(
            "\n{}",
            "Generated commit message variations:".green().bold()
        );
        println!("{}", "─".repeat(50).dimmed());
        for (i, msg) in messages.iter().enumerate() {
            println!("{}. {}", i + 1, msg);
        }
        println!("{}", "─".repeat(50).dimmed());
    }
}

/// Handle the commit action (commit, edit, select, cancel, regenerate)
async fn handle_commit_action(
    options: &GlobalOptions,
    config: &Config,
    messages: &[String],
    final_message: &mut String,
) -> Result<()> {
    let action = if options.skip_confirmation {
        CommitAction::Commit
    } else if messages.len() > 1 {
        select_commit_action_with_variants(messages.len())?
    } else {
        select_commit_action()?
    };

    match action {
        CommitAction::Commit => {
            perform_commit(final_message)?;
            run_post_commit_hooks(config, final_message).await?;
            println!("{}", "✅ Changes committed successfully!".green());
        }
        CommitAction::Edit => {
            let edited_message = edit_commit_message(final_message)?;
            perform_commit(&edited_message)?;
            run_post_commit_hooks(config, &edited_message).await?;
            println!("{}", "✅ Changes committed successfully!".green());
        }
        CommitAction::Select { index } => {
            let selected_message = messages[index].clone();
            let final_msg = if !options.no_pre_hooks {
                run_pre_commit_hooks(config, &selected_message)?
            } else {
                selected_message
            };
            perform_commit(&final_msg)?;
            run_post_commit_hooks(config, &final_msg).await?;
            println!("{}", "✅ Changes committed successfully!".green());
        }
        CommitAction::Cancel => {
            println!("{}", "Commit cancelled.".yellow());
        }
        CommitAction::Regenerate => {
            // Recursive call to regenerate
            Box::pin(execute(options.clone())).await?;
        }
    }

    Ok(())
}

fn select_files_to_stage(files: &[String]) -> Result<Vec<String>> {
    let theme = ColorfulTheme::default();
    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select files to stage")
        .items(files)
        .interact()?;

    Ok(selections.into_iter().map(|i| files[i].clone()).collect())
}

enum CommitAction {
    Commit,
    Edit,
    Cancel,
    Regenerate,
    Select { index: usize },
}

fn select_commit_action() -> Result<CommitAction> {
    let choices = vec!["Commit", "Edit message", "Cancel", "Regenerate"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()?;

    Ok(match selection {
        0 => CommitAction::Commit,
        1 => CommitAction::Edit,
        2 => CommitAction::Cancel,
        3 => CommitAction::Regenerate,
        _ => unreachable!(),
    })
}

fn select_commit_action_with_variants(num_variants: usize) -> Result<CommitAction> {
    let mut choices: Vec<String> = (1..=num_variants)
        .map(|i| format!("Use option {}", i))
        .collect();
    choices.extend(vec![
        "Edit message".to_string(),
        "Cancel".to_string(),
        "Regenerate".to_string(),
    ]);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()?;

    Ok(if selection < num_variants {
        CommitAction::Select { index: selection }
    } else {
        match selection - num_variants {
            0 => CommitAction::Edit,
            1 => CommitAction::Cancel,
            2 => CommitAction::Regenerate,
            _ => unreachable!(),
        }
    })
}

fn select_message_variant(messages: &[String]) -> Result<usize> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a commit message")
        .items(messages)
        .default(0)
        .interact()?;

    Ok(selection)
}

fn edit_commit_message(original: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Edit commit message")
        .with_initial_text(original)
        .interact_text()
        .context("Failed to read edited commit message")
}

fn perform_commit(message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .context("Failed to execute git commit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git commit failed: {}", stderr);
    }

    Ok(())
}

async fn run_post_commit_hooks(config: &Config, message: &str) -> Result<()> {
    if let Some(hooks) = config.post_commit_hook.clone() {
        let envs = vec![
            ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
            ("RCO_COMMIT_MESSAGE", message.to_string()),
            (
                "RCO_PROVIDER",
                config.ai_provider.clone().unwrap_or_default(),
            ),
            ("RCO_MODEL", config.model.clone().unwrap_or_default()),
        ];
        run_hooks(HookOptions {
            name: "post-commit",
            commands: hooks,
            strict: config.hook_strict.unwrap_or(true),
            timeout: std::time::Duration::from_millis(config.hook_timeout_ms.unwrap_or(30000)),
            envs,
        })?;
    }
    Ok(())
}

/// Run pre-commit hooks on a commit message, returning the possibly modified message
fn run_pre_commit_hooks(config: &Config, message: &str) -> Result<String> {
    if let Some(hooks) = config.pre_commit_hook.clone() {
        let commit_file = write_temp_commit_file(message)?;
        let envs = vec![
            ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
            ("RCO_COMMIT_MESSAGE", message.to_string()),
            ("RCO_COMMIT_FILE", commit_file.to_string_lossy().to_string()),
            (
                "RCO_PROVIDER",
                config.ai_provider.clone().unwrap_or_default(),
            ),
            ("RCO_MODEL", config.model.clone().unwrap_or_default()),
        ];
        run_hooks(HookOptions {
            name: "pre-commit",
            commands: hooks,
            strict: config.hook_strict.unwrap_or(true),
            timeout: std::time::Duration::from_millis(config.hook_timeout_ms.unwrap_or(30000)),
            envs,
        })?;
        // Read back possibly modified commit file
        if let Ok(updated) = std::fs::read_to_string(&commit_file) {
            if !updated.trim().is_empty() {
                return Ok(updated);
            }
        }
    }
    Ok(message.to_string())
}

async fn generate_commit_messages(
    config: &Config,
    diff: &str,
    context: Option<&str>,
    full_gitmoji: bool,
    count: u8,
) -> Result<Vec<String>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!(
        "Generating {} commit message{}...",
        count,
        if count > 1 { "s" } else { "" }
    ));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Try to use an active account first
    let provider: Box<dyn providers::AIProvider> =
        if let Some(account) = config.get_active_account()? {
            tracing::info!("Using account: {}", account.alias);
            println!(
                "{} Using account: {}",
                "🔐".dimmed(),
                account.alias.yellow()
            );
            providers::create_provider_for_account(&account, config)?
        } else {
            providers::create_provider(config)?
        };

    let messages = provider
        .generate_commit_messages(diff, context, full_gitmoji, config, count)
        .await?;

    pb.finish_with_message("Commit message(s) generated!");
    Ok(messages)
}

/// Load and parse .rcoignore file
fn load_rcoignore() -> Result<Vec<String>> {
    let repo_root = git::get_repo_root()?;
    let rcoignore_path = Path::new(&repo_root).join(".rcoignore");

    if !rcoignore_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&rcoignore_path)?;
    Ok(content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect())
}

/// Filter diff to exclude files matching .rcoignore patterns
fn filter_diff_by_rcoignore(diff: &str) -> Result<String> {
    let patterns = load_rcoignore();
    let patterns = match patterns {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Warning: Failed to read .rcoignore: {}", e);
            return Ok(diff.to_string());
        }
    };

    if patterns.is_empty() {
        return Ok(diff.to_string());
    }

    // Pre-allocate with reasonable capacity estimate
    let mut filtered = String::with_capacity(diff.len().min(1024));
    let mut include_current_file = true;

    for line in diff.lines() {
        if line.starts_with("+++ b/") || line.starts_with("--- a/") {
            let file_path = line
                .strip_prefix("+++ b/")
                .unwrap_or_else(|| line.strip_prefix("--- a/").unwrap_or(&line[6..]));

            include_current_file = !patterns.iter().any(|pattern| {
                if pattern.starts_with('/') {
                    // Exact match from root
                    file_path.trim_start_matches('/') == pattern.trim_start_matches('/')
                } else {
                    // Match anywhere in path
                    file_path.contains(pattern)
                }
            });
        }

        if include_current_file {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }

    Ok(filtered)
}

/// Chunk a large diff into smaller pieces that fit within token limit
fn chunk_diff(diff: &str, max_tokens: usize) -> Result<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::with_capacity(diff.len().min(4096));
    let mut current_tokens = 0;

    // Reserve some tokens for the prompt
    let effective_max = max_tokens.saturating_sub(PROMPT_OVERHEAD_TOKENS);

    for line in diff.lines() {
        let line_tokens = utils::token::estimate_tokens(line)?;

        if current_tokens + line_tokens > effective_max && !current_chunk.is_empty() {
            chunks.push(current_chunk);
            current_chunk = String::new();
            current_tokens = 0;
        }

        current_chunk.push_str(line);
        current_chunk.push('\n');
        current_tokens += line_tokens;
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    if chunks.is_empty() {
        return Ok(diff.to_string());
    }

    // If we have multiple chunks, combine them with a summary header
    if chunks.len() > 1 {
        tracing::info!("Split diff into {} chunks", chunks.len());

        // Generate a combined diff by concatenating all chunks
        // The AI will understand the full context
        let combined = chunks.join("\n\n---CHUNK SEPARATOR---\n\n");
        Ok(combined)
    } else {
        Ok(chunks.into_iter().next().unwrap_or_default())
    }
}

/// Copy text to clipboard with proper error handling
fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Use pbcopy with properly piped stdin
        let mut process = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn pbcopy process")?;

        // Write to stdin, handling the Result properly
        {
            let stdin = process
                .stdin
                .as_mut()
                .context("pbcopy stdin not available")?;
            stdin
                .write_all(text.as_bytes())
                .context("Failed to write to clipboard")?;
        }

        let status = process
            .wait()
            .context("Failed to wait for pbcopy process")?;

        if !status.success() {
            anyhow::bail!("pbcopy exited with error: {:?}", status);
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Check if xclip is available, otherwise try xsel as fallback
        let use_xclip = !Command::new("which")
            .arg("xclip")
            .output()?
            .stdout
            .is_empty();

        let (cmd_name, args) = if use_xclip {
            ("xclip", vec!["-selection", "clipboard"])
        } else {
            ("xsel", vec!["--clipboard", "--input"])
        };

        let mut process = Command::new(cmd_name)
            .args(&args)
            .stdin(Stdio::piped())
            .spawn()
            .context(format!("Failed to spawn {} process", cmd_name))?;

        {
            let stdin = process
                .stdin
                .as_mut()
                .context(format!("{} stdin not available", cmd_name))?;
            stdin
                .write_all(text.as_bytes())
                .context("Failed to write to clipboard")?;
        }

        let status = process
            .wait()
            .context(format!("Failed to wait for {} process", cmd_name))?;

        if !status.success() {
            anyhow::bail!("{} exited with error: {:?}", cmd_name, status);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut ctx = arboard::Clipboard::new()
            .map_err(|e| anyhow::anyhow!("Failed to access clipboard: {}", e))?;
        ctx.set_text(text.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to set clipboard contents: {}", e))?;
    }

    Ok(())
}

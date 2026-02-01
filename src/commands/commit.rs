use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::path::Path;
use std::process::Command;

use crate::cli::GlobalOptions;
use crate::config::Config;
use crate::git;
use crate::output::progress;
use crate::output::styling::Styling;
use crate::providers;
use crate::utils;
use crate::utils::hooks::{run_hooks, write_temp_commit_file, HookOptions};

/// Tokens reserved for prompt overhead when chunking diffs.
/// This accounts for system prompts, user instructions, and response tokens
/// that are sent alongside the diff content.
const PROMPT_OVERHEAD_TOKENS: usize = 500;

/// Execution context for commit message output.
struct ExecContext;

impl ExecContext {
    fn new(_options: &GlobalOptions) -> Self {
        Self
    }

    /// Print a success message.
    fn success(&self, message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    /// Print a warning message.
    fn warning(&self, message: &str) {
        eprintln!("{} {}", "!".yellow().bold(), message);
    }

    /// Print an error message.
    fn error(&self, message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }

    /// Print a header.
    fn header(&self, text: &str) {
        println!("\n{}", text.bold());
    }

    /// Print a subheader.
    fn subheader(&self, text: &str) {
        println!("{}", text.dimmed());
    }

    /// Print a divider.
    fn divider(&self, length: Option<usize>) {
        let len = length.unwrap_or(50);
        println!("{}", Styling::divider(len));
    }

    /// Print a key-value pair.
    fn key_value(&self, key: &str, value: &str) {
        println!("{}: {}", key.dimmed(), value);
    }
}

pub async fn execute(options: GlobalOptions) -> Result<()> {
    let ctx = ExecContext::new(&options);

    // Ensure we're in a git repository
    git::assert_git_repo()?;

    // Load and validate configuration
    let config = load_and_validate_config(&options)?;

    // Determine effective generate count (CLI > config > default), clamped to 1-5
    let generate_count = options
        .generate_count
        .max(config.generate_count.unwrap_or(1))
        .clamp(1, 5);

    // Prepare the diff for processing
    let (final_diff, token_count) = prepare_diff(&config, &ctx)?;

    // If --show-prompt flag is set, just show the prompt and exit
    if options.show_prompt {
        display_prompt(&config, &final_diff, options.context.as_deref(), &ctx);
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
        options.strip_thinking,
        &ctx,
    )
    .await?;

    if messages.is_empty() {
        anyhow::bail!("Failed to generate any commit messages");
    }

    // Handle clipboard mode
    if options.clipboard {
        return handle_clipboard_mode(&messages, &ctx);
    }

    // Handle print mode (for hooks compatibility)
    if options.print_message {
        print!("{}", messages[0]);
        return Ok(());
    }

    // Handle dry-run mode - preview without committing
    if options.dry_run {
        return handle_dry_run_mode(&messages, &ctx);
    }

    // Run pre-commit hooks on first message
    let mut final_message = messages[0].clone();
    if !options.no_pre_hooks {
        final_message = run_pre_commit_hooks(&config, &final_message)?;
    }

    // Display messages and handle commit action
    display_commit_messages(&messages, &ctx);
    handle_commit_action(&options, &config, &messages, &mut final_message, &ctx).await
}

/// Load configuration and apply commitlint rules
fn load_and_validate_config(options: &GlobalOptions) -> Result<Config> {
    let mut config = Config::load()?;

    // Apply CLI prompt-file override if provided
    if let Some(ref prompt_file) = options.prompt_file {
        config.set_prompt_file(Some(prompt_file.clone()));
    }

    // Apply skill if specified
    if let Some(ref skill_name) = options.skill {
        apply_skill_to_config(&mut config, skill_name)?;
    }

    config.load_with_commitlint()?;
    config.apply_commitlint_rules()?;
    Ok(config)
}

/// Apply a skill's configuration to the config
fn apply_skill_to_config(config: &mut Config, skill_name: &str) -> Result<()> {
    use crate::skills::SkillsManager;

    let mut manager = SkillsManager::new()?;
    manager.discover()?;

    let skill = manager
        .find(skill_name)
        .ok_or_else(|| anyhow::anyhow!(
            "Skill '{}' not found. Run 'rco skills list' to see available skills.",
            skill_name
        ))?;

    // Load prompt template from skill if available
    if let Some(prompt_template) = skill.load_prompt_template()? {
        config.custom_prompt = Some(prompt_template);
        tracing::info!("Loaded prompt template from skill: {}", skill_name);
    }

    println!("{} Using skill: {}", "→".cyan(), skill_name.green());
    Ok(())
}

/// Prepare the diff for processing: get staged changes, apply filters, chunk if needed
fn prepare_diff(config: &Config, ctx: &ExecContext) -> Result<(String, usize)> {
    // Check for staged files or changes
    let staged_files = git::get_staged_files()?;
    let changed_files = if staged_files.is_empty() {
        git::get_changed_files()?
    } else {
        staged_files
    };

    if changed_files.is_empty() {
        ctx.error("No changes to commit");
        ctx.subheader("Stage some changes with 'git add' or use 'git add -A' to stage all changes");
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
        ctx.error("No staged changes to commit");
        anyhow::bail!("No staged changes to commit");
    }

    // Apply .rcoignore if it exists
    let diff = filter_diff_by_rcoignore(&diff)?;

    // Check if diff became empty after filtering
    if diff.trim().is_empty() {
        ctx.error("No changes to commit after applying .rcoignore filters");
        anyhow::bail!("No changes to commit after applying .rcoignore filters");
    }

    // Check if diff is too large - implement chunking if needed
    let max_tokens = config.tokens_max_input.unwrap_or(4096);
    let token_count = utils::token::estimate_tokens(&diff)?;

    // If diff is too large, chunk it
    let final_diff = if token_count > max_tokens {
        ctx.warning(&format!(
            "The diff is too large ({} tokens). Splitting into chunks...",
            token_count
        ));
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
fn display_prompt(config: &Config, diff: &str, context: Option<&str>, ctx: &ExecContext) {
    let prompt = config.get_effective_prompt(diff, context, false);
    ctx.header("Prompt that would be sent to AI");
    ctx.divider(None);
    println!("{}", prompt);
    ctx.divider(None);
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
fn handle_clipboard_mode(messages: &[String], ctx: &ExecContext) -> Result<()> {
    let selected = if messages.len() == 1 {
        0
    } else {
        select_message_variant(messages)?
    };
    copy_to_clipboard(&messages[selected])?;
    ctx.success("Commit message copied to clipboard!");
    Ok(())
}

/// Handle dry-run mode - preview message without committing
fn handle_dry_run_mode(messages: &[String], ctx: &ExecContext) -> Result<()> {
    ctx.header("Dry Run Mode - Preview");
    ctx.divider(None);
    ctx.subheader("The following commit message would be generated:");
    println!();
    
    if messages.len() == 1 {
        println!("{}", messages[0].green());
    } else {
        ctx.subheader("Multiple variations available:");
        for (i, msg) in messages.iter().enumerate() {
            println!("\n{}. {}", i + 1, format!("Option {}", i + 1).cyan().bold());
            println!("{}", msg.green());
        }
    }
    
    ctx.divider(None);
    ctx.subheader("No commit was made. Remove --dry-run to commit.");
    Ok(())
}

/// Display the generated commit message(s)
fn display_commit_messages(messages: &[String], ctx: &ExecContext) {
    if messages.len() == 1 {
        ctx.header("Generated Commit Message");
        ctx.divider(None);
        println!("{}", messages[0]);
        ctx.divider(None);
    } else {
        ctx.header("Generated Commit Message Variations");
        ctx.divider(None);
        for (i, msg) in messages.iter().enumerate() {
            println!("{}. {}", i + 1, msg);
        }
        ctx.divider(None);
    }
}

/// Handle the commit action (commit, edit, select, cancel, regenerate)
async fn handle_commit_action(
    options: &GlobalOptions,
    config: &Config,
    messages: &[String],
    final_message: &mut str,
    ctx: &ExecContext,
) -> Result<()> {
    let action = if options.skip_confirmation {
        CommitAction::Commit
    } else if options.edit {
        // --edit flag: go straight to editor with the first message
        CommitAction::EditExternal
    } else if messages.len() > 1 {
        select_commit_action_with_variants(messages.len())?
    } else {
        select_commit_action()?
    };

    match action {
        CommitAction::Commit => {
            perform_commit(final_message)?;
            run_post_commit_hooks(config, final_message).await?;
            ctx.success("Changes committed successfully!");
        }
        CommitAction::Edit => {
            let edited_message = edit_commit_message(final_message)?;
            perform_commit(&edited_message)?;
            run_post_commit_hooks(config, &edited_message).await?;
            ctx.success("Changes committed successfully!");
        }
        CommitAction::EditExternal => {
            // Open in $EDITOR (e.g., vim, nano, code, etc.)
            let edited_message = edit_in_external_editor(final_message)?;
            if edited_message.trim().is_empty() {
                ctx.warning("Commit cancelled - empty message.");
                return Ok(());
            }
            perform_commit(&edited_message)?;
            run_post_commit_hooks(config, &edited_message).await?;
            ctx.success("Changes committed successfully!");
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
            ctx.success("Changes committed successfully!");
        }
        CommitAction::Cancel => {
            ctx.warning("Commit cancelled.");
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
    EditExternal, // Open in $EDITOR
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

/// Open commit message in $EDITOR for editing
fn edit_in_external_editor(original: &str) -> Result<String> {
    use std::env;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::process::Command;

    // Get the editor from environment
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Default editors by platform
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    // Create a temporary file with the commit message
    let mut temp_file = NamedTempFile::with_suffix(".txt")
        .context("Failed to create temporary file for editing")?;
    
    // Write the original message to the temp file
    temp_file
        .write_all(original.as_bytes())
        .context("Failed to write to temporary file")?;
    temp_file.flush().context("Failed to flush temporary file")?;
    
    let temp_path = temp_file.path().to_path_buf();
    
    // Keep the temp file from being deleted when dropped
    let _temp_file = temp_file.into_temp_path();
    
    // Open the editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .with_context(|| format!("Failed to open editor '{}'. Make sure $EDITOR is set correctly.", editor))?;
    
    if !status.success() {
        anyhow::bail!("Editor exited with error status");
    }
    
    // Read the edited message back
    let edited = std::fs::read_to_string(&temp_path)
        .context("Failed to read edited commit message from temporary file")?;
    
    Ok(edited)
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
    strip_thinking: bool,
    ctx: &ExecContext,
) -> Result<Vec<String>> {
    let pb = progress::spinner(&format!(
        "Generating {} commit message{}...",
        count,
        if count > 1 { "s" } else { "" }
    ));

    // Try to use an active account first
    let provider: Box<dyn providers::AIProvider> =
        if let Some(account) = config.get_active_account()? {
            tracing::info!("Using account: {}", account.alias);
            ctx.key_value("Using account", &account.alias);
            providers::create_provider_for_account(&account, config)?
        } else {
            providers::create_provider(config)?
        };

    let mut messages = provider
        .generate_commit_messages(diff, context, full_gitmoji, config, count)
        .await?;

    // Strip thinking tags if requested
    if strip_thinking {
        for message in &mut messages {
            *message = utils::strip_thinking(message);
        }
    }

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
    // Use the enhanced multi-level chunking from utils
    let effective_max = max_tokens.saturating_sub(PROMPT_OVERHEAD_TOKENS);
    let chunked = utils::chunk_diff(diff, effective_max);

    // Log if chunking occurred
    if chunked.contains("---CHUNK") {
        tracing::info!("Diff was chunked for token limit");
    }

    Ok(chunked)
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

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

use crate::cli::GlobalOptions;
use crate::config::Config;
use crate::git;
use crate::providers;
use crate::utils;
use crate::utils::hooks::{run_hooks, write_temp_commit_file, HookOptions};

pub async fn execute(options: GlobalOptions) -> Result<()> {
    // Ensure we're in a git repository
    git::assert_git_repo()?;

    // Get the current configuration
    let mut config = Config::load()?;

    // Load and apply commitlint configuration
    config.load_with_commitlint()?;
    config.apply_commitlint_rules()?;

    // Check for staged files or changes
    let staged_files = git::get_staged_files()?;
    let changed_files = if staged_files.is_empty() {
        git::get_changed_files()?
    } else {
        staged_files
    };

    if changed_files.is_empty() {
        println!("{}", "No changes to commit.".yellow());
        return Ok(());
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
        println!("{}", "No staged changes to commit.".yellow());
        return Ok(());
    }

    // Check if diff is too large
    let max_tokens = config.tokens_max_input.unwrap_or(4096);
    let token_count = utils::token::estimate_tokens(&diff)?;

    if token_count > max_tokens {
        println!(
            "{}",
            format!(
                "The diff is too large ({token_count} tokens). Maximum allowed: {max_tokens} tokens."
            )
            .red()
        );
        return Ok(());
    }

    // If --show-prompt flag is set, just show the prompt and exit
    if options.show_prompt {
        let prompt =
            config.get_effective_prompt(&diff, options.context.as_deref(), options.full_gitmoji);
        println!("\n{}", "Prompt that would be sent to AI:".green().bold());
        println!("{}", "═".repeat(60).dimmed());
        println!("{}", prompt);
        println!("{}", "═".repeat(60).dimmed());
        return Ok(());
    }

    // Run pre-generation hooks (optional)
    if !options.no_pre_hooks {
        if let Some(hooks) = config.pre_gen_hook.clone() {
            let envs = vec![
                ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
                ("RCO_MAX_TOKENS", (config.tokens_max_input.unwrap_or(4096)).to_string()),
                ("RCO_DIFF_TOKENS", token_count.to_string()),
                ("RCO_CONTEXT", options.context.clone().unwrap_or_default()),
                ("RCO_PROVIDER", config.ai_provider.clone().unwrap_or_default()),
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
    }

    // Generate commit message
    let commit_message = generate_commit_message(
        &config,
        &diff,
        options.context.as_deref(),
        options.full_gitmoji,
    )
    .await?;

    // Run pre-commit hooks (can modify message via temp file)
    let mut final_message = commit_message.clone();
    if !options.no_pre_hooks {
        if let Some(hooks) = config.pre_commit_hook.clone() {
            let commit_file = write_temp_commit_file(&final_message)?;
            let envs = vec![
                ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
                ("RCO_COMMIT_MESSAGE", final_message.clone()),
                ("RCO_COMMIT_FILE", commit_file.to_string_lossy().to_string()),
                ("RCO_PROVIDER", config.ai_provider.clone().unwrap_or_default()),
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
                    final_message = updated;
                }
            }
        }
    }

    // Display the generated commit message
    println!("\n{}", "Generated commit message:".green().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{commit_message}");
    println!("{}", "─".repeat(50).dimmed());

    // Ask for confirmation or allow editing
    let action = if options.skip_confirmation {
        CommitAction::Commit
    } else {
        select_commit_action()?
    };

    match action {
        CommitAction::Commit => {
            perform_commit(&final_message)?;
            // Post-commit hooks
            if !options.no_post_hooks {
                if let Some(hooks) = config.post_commit_hook.clone() {
                    let envs = vec![
                        ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
                        ("RCO_COMMIT_MESSAGE", final_message.clone()),
                        ("RCO_PROVIDER", config.ai_provider.clone().unwrap_or_default()),
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
            }
            println!("{}", "✅ Changes committed successfully!".green());
        }
        CommitAction::Edit => {
            let edited_message = edit_commit_message(&final_message)?;
            perform_commit(&edited_message)?;
            if !options.no_post_hooks {
                if let Some(hooks) = config.post_commit_hook.clone() {
                    let envs = vec![
                        ("RCO_REPO_ROOT", git::get_repo_root()?.to_string()),
                        ("RCO_COMMIT_MESSAGE", edited_message.clone()),
                        ("RCO_PROVIDER", config.ai_provider.clone().unwrap_or_default()),
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
            }
            println!("{}", "✅ Changes committed successfully!".green());
        }
        CommitAction::Cancel => {
            println!("{}", "Commit cancelled.".yellow());
        }
        CommitAction::Regenerate => {
            // Recursive call to regenerate
            Box::pin(execute(options)).await?;
        }
    }

    Ok(())
}

fn select_files_to_stage(files: &[String]) -> Result<Vec<String>> {
    let theme = ColorfulTheme::default();
    let selections = dialoguer::MultiSelect::with_theme(&theme)
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

async fn generate_commit_message(
    config: &Config,
    diff: &str,
    context: Option<&str>,
    full_gitmoji: bool,
) -> Result<String> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Generating commit message...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let provider = providers::create_provider(config)?;
    let message = provider
        .generate_commit_message(diff, context, full_gitmoji, config)
        .await?;

    pb.finish_with_message("Commit message generated!");
    Ok(message)
}

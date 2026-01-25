use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::process::Command;

use crate::cli::PrCommand;
use crate::config::Config;
use crate::git;
use crate::providers;

pub async fn execute(cmd: PrCommand) -> Result<()> {
    let config = Config::load()?;
    let repo_root = git::get_repo_root()?;

    match cmd.action {
        crate::cli::PrAction::Generate { base } => {
            generate_pr_description(&config, base.as_deref()).await
        }
        crate::cli::PrAction::Browse { base } => browse_pr_page(&repo_root, base.as_deref()),
    }
}

async fn generate_pr_description(config: &Config, base_branch: Option<&str>) -> Result<()> {
    let current_branch = git::get_current_branch()?;
    let base = base_branch.unwrap_or("main");

    println!(
        "{}",
        format!(
            "Generating PR description for branch '{}' against '{}'...",
            current_branch, base
        )
        .green()
        .bold()
    );

    // Get commits between branches
    let commits = git::get_commits_between(base, &current_branch)?;
    let diff = git::get_diff_between(base, &current_branch)?;

    if commits.is_empty() {
        println!(
            "{}",
            "No commits found to generate PR description.".yellow()
        );
        return Ok(());
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Generating PR description...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let provider = providers::create_provider(config)?;
    let description = provider
        .generate_pr_description(&commits, &diff, config)
        .await?;

    pb.finish_with_message("PR description generated!");

    // Display the description
    println!("\n{}", "Generated PR Description:".green().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", description);
    println!("{}", "─".repeat(50).dimmed());

    // Copy to clipboard option
    let choices = vec!["Copy to clipboard", "Show markdown", "Cancel"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            copy_to_clipboard(&description)?;
            println!("{}", "✅ PR description copied to clipboard!".green());
        }
        1 => {
            // Save to file for preview
            let preview_file = format!("PR_DESCRIPTION_{}.md", current_branch.replace('/', "_"));
            std::fs::write(&preview_file, &description)?;
            println!(
                "{}",
                format!("PR description saved to: {}", preview_file).green()
            );

            // Try to open in editor
            if let Ok(editor) = std::env::var("EDITOR") {
                let _ = Command::new(&editor).arg(&preview_file).status();
            }
        }
        _ => {
            println!("{}", "Cancelled.".yellow());
        }
    }

    Ok(())
}

fn browse_pr_page(_repo_root: &str, base_branch: Option<&str>) -> Result<()> {
    let current_branch = git::get_current_branch()?;
    let base = base_branch.unwrap_or("main");

    // Try to get GitHub remote URL
    let remote_url = git::get_remote_url()?;
    let pr_url = convert_to_pr_url(&remote_url, &current_branch, base)?;

    println!("{}", format!("Opening PR page: {}", pr_url).green());

    if let Err(e) = webbrowser::open(&pr_url) {
        eprintln!("Failed to open browser: {}", e);
        println!("Please open the following URL manually:");
        println!("{}", pr_url);
    }

    Ok(())
}

fn convert_to_pr_url(remote_url: &str, branch: &str, base: &str) -> Result<String> {
    // Convert SSH URL to HTTPS URL
    let url = if remote_url.contains("@") && remote_url.contains(":") {
        // SSH format: git@github.com:owner/repo.git
        let parts: Vec<&str> = remote_url.splitn(2, ':').collect();
        if parts.len() == 2 {
            let host_path = parts[1];
            let host_parts: Vec<&str> = parts[0].splitn(2, '@').collect();
            if host_parts.len() == 2 {
                let host = host_parts[1];
                let _path = host_path.trim_end_matches(".git");
                format!("https://{}/compare/{}...{}?expand=1", host, base, branch)
            } else {
                remote_url.to_string()
            }
        } else {
            remote_url.to_string()
        }
    } else if remote_url.contains("github.com") {
        // HTTPS format
        remote_url.replace(".git", "") + &format!("/compare/{}...{}?expand=1", base, branch)
    } else {
        // Non-GitHub repo
        format!("{}/compare/{}...{}", remote_url, base, branch)
    };

    Ok(url)
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut process = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn pbcopy process")?;

        {
            let stdin = process
                .stdin
                .as_mut()
                .context("pbcopy stdin not available")?;
            stdin.write_all(text.as_bytes())?;
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
            stdin.write_all(text.as_bytes())?;
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

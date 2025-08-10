use crate::cli::UpdateCommand;
use crate::update::{check_for_update, perform_update, InstallMethod};
use anyhow::Result;
use colored::*;
use semver::Version;

pub async fn execute(cmd: UpdateCommand) -> Result<()> {
    // If a specific version is requested
    if let Some(target_version) = cmd.version {
        return update_to_specific_version(&target_version, cmd.force).await;
    }

    // Check for updates
    let update_info = check_for_update().await?;

    println!("{}", "Checking for updates...".blue());
    println!(
        "Current version: {}",
        format!("v{}", update_info.current_version).cyan()
    );
    println!(
        "Latest version:  {}",
        format!("v{}", update_info.latest_version).cyan()
    );
    println!(
        "Install method:  {}",
        format!("{:?}", update_info.install_method).cyan()
    );

    // If only checking
    if cmd.check {
        if update_info.needs_update {
            println!(
                "\n{}",
                format!(
                    "Update available! Run 'rco update' to update to v{}",
                    update_info.latest_version
                )
                .yellow()
            );
        } else {
            println!("\n{}", "You're running the latest version! 🎉".green());
        }
        return Ok(());
    }

    // Check if update is needed
    if !update_info.needs_update && !cmd.force {
        println!(
            "\n{}",
            "You're already running the latest version! 🎉".green()
        );
        println!(
            "{}",
            "Use --force to reinstall the current version.".dimmed()
        );
        return Ok(());
    }

    // Warn about unknown installation method
    if update_info.install_method == InstallMethod::Unknown {
        eprintln!(
            "\n{}",
            "Warning: Could not detect installation method.".yellow()
        );
        eprintln!(
            "{}",
            "Please update manually or use the install script:".yellow()
        );
        eprintln!(
            "  {}",
            "curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash".cyan()
        );
        return Ok(());
    }

    // Confirm update
    if !confirm_update(
        &update_info.current_version,
        &update_info.latest_version,
        cmd.force,
    )? {
        println!("{}", "Update cancelled.".yellow());
        return Ok(());
    }

    // Perform update
    println!();
    perform_update(&update_info).await?;

    Ok(())
}

async fn update_to_specific_version(version_str: &str, force: bool) -> Result<()> {
    // Clean version string (remove 'v' prefix if present)
    let version_str = version_str.trim_start_matches('v');

    // Validate version
    let target_version = Version::parse(version_str)?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    // Check if already on this version
    if target_version == current_version && !force {
        println!(
            "{}",
            format!("Already running version v{}!", version_str).green()
        );
        println!(
            "{}",
            "Use --force to reinstall the current version.".dimmed()
        );
        return Ok(());
    }

    // Get installation method
    let install_method = crate::update::detect_install_method()?;

    if install_method == InstallMethod::Unknown {
        eprintln!(
            "{}",
            "Warning: Could not detect installation method.".yellow()
        );
        eprintln!(
            "{}",
            "Please update manually or use the install script:".yellow()
        );
        eprintln!(
            "  {}",
            format!("curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash -s -- --version v{}", version_str).cyan()
        );
        return Ok(());
    }

    // Create update info for specific version
    let update_info = crate::update::UpdateInfo {
        current_version: env!("CARGO_PKG_VERSION").to_string(),
        latest_version: version_str.to_string(),
        install_method,
        executable_path: std::env::current_exe()?,
        needs_update: true, // Force update
    };

    // Confirm update
    if !confirm_update(
        &update_info.current_version,
        &update_info.latest_version,
        force,
    )? {
        println!("{}", "Update cancelled.".yellow());
        return Ok(());
    }

    // Perform update
    println!();
    perform_update(&update_info).await?;

    Ok(())
}

fn confirm_update(current: &str, target: &str, force: bool) -> Result<bool> {
    use std::io::{self, Write};

    if force {
        println!(
            "\n{}",
            format!("Force updating from v{} to v{}", current, target).yellow()
        );
        return Ok(true);
    }

    print!(
        "\n{} ",
        format!("Update from v{} to v{}? [Y/n]", current, target).yellow()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input.is_empty() || input == "y" || input == "yes")
}

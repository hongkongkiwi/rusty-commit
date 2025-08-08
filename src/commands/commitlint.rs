use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::Path;

use crate::cli::CommitLintCommand;
use crate::git;

const COMMITLINT_CONFIG: &str = r#"module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'style',
        'refactor',
        'perf',
        'test',
        'build',
        'ci',
        'chore',
        'revert'
      ]
    ]
  }
};
"#;

pub async fn execute(cmd: CommitLintCommand) -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let config_path = Path::new(&repo_root).join(".commitlintrc.js");

    if config_path.exists() && !cmd.set {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Commitlint config already exists. Overwrite?")
            .default(false)
            .interact()?;

        if !overwrite {
            println!("{}", "Commitlint configuration unchanged".yellow());
            return Ok(());
        }
    }

    // Write the commitlint configuration
    fs::write(&config_path, COMMITLINT_CONFIG)
        .context("Failed to write commitlint configuration")?;

    println!("{}", "✅ Commitlint configuration created!".green());
    println!("Configuration written to: {}", config_path.display());

    // Check if package.json exists
    let package_json_path = Path::new(&repo_root).join("package.json");
    if package_json_path.exists() {
        println!("\n{}", "📦 Next steps:".bold());
        println!("1. Install commitlint dependencies:");
        println!("   npm install --save-dev @commitlint/cli @commitlint/config-conventional");
        println!("2. Add husky hook for commit-msg:");
        println!("   npx husky add .husky/commit-msg 'npx --no -- commitlint --edit \"$1\"'");
    }

    Ok(())
}

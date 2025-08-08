use anyhow::Result;
use colored::Colorize;

use crate::cli::{ConfigAction, ConfigCommand};
use crate::config::{self, Config};

pub async fn execute(cmd: ConfigCommand) -> Result<()> {
    let mut config = Config::load()?;

    match cmd.action {
        ConfigAction::Set { pairs } => {
            for pair in pairs {
                let parts: Vec<&str> = pair.splitn(2, '=').collect();
                if parts.len() != 2 {
                    eprintln!(
                        "{}",
                        format!("Invalid format: {pair}. Use KEY=value").red()
                    );
                    continue;
                }

                let key = parts[0];
                let value = parts[1];

                match config.set(key, value) {
                    Ok(_) => {
                        println!("{}", format!("✅ {key} set to: {value}").green());
                    }
                    Err(e) => {
                        eprintln!("{}", format!("❌ Failed to set {key}: {e}").red());
                    }
                }
            }
        }
        ConfigAction::Get { key } => match config.get(&key) {
            Ok(value) => {
                println!("{key}: {value}");
            }
            Err(e) => {
                eprintln!("{}", format!("❌ {e}").red());
            }
        },
        ConfigAction::Reset { all, keys } => {
            if all {
                config.reset(None)?;
                println!("{}", "✅ All configuration reset to defaults".green());
            } else if !keys.is_empty() {
                config.reset(Some(&keys))?;
                println!("{}", format!("✅ Reset keys: {}", keys.join(", ")).green());
            } else {
                eprintln!("{}", "Please specify --all or provide keys to reset".red());
            }
        }
        ConfigAction::Status => {
            println!("\n{}", "🔐 Secure Storage Status".bold());
            println!("{}", "─".repeat(50).dimmed());
            
            // Show platform info
            println!("Platform: {}", config::secure_storage::get_platform_info());
            
            let status = config::secure_storage::status_message();
            println!("Status: {}", status);
            
            if config::secure_storage::is_available() {
                println!("\n{}", "✅ API keys will be stored securely".green());
                println!("{}", "   Your API keys are encrypted and protected by your system".dimmed());
                
                // Platform-specific information
                #[cfg(target_os = "macos")]
                println!("{}", "   Stored in: macOS Keychain (login keychain)".dimmed());
                
                #[cfg(target_os = "linux")]
                println!("{}", "   Stored in: Secret Service (GNOME Keyring/KWallet)".dimmed());
                
                #[cfg(target_os = "windows")]
                println!("{}", "   Stored in: Windows Credential Manager".dimmed());
            } else {
                println!("\n{}", "⚠️  API keys will be stored in the configuration file".yellow());
                println!("{}", "   Location: ~/.config/rustycommit/config.toml".dimmed());
                
                #[cfg(not(feature = "secure-storage"))]
                {
                    println!("{}", "   To enable secure storage:".dimmed());
                    println!("{}", "   cargo install rustycommit --features secure-storage".dimmed());
                }
                
                #[cfg(feature = "secure-storage")]
                {
                    println!("{}", "   Note: Secure storage is not available on this system".dimmed());
                    println!("{}", "   Falling back to file-based storage".dimmed());
                }
            }
            
            // Show current API key status
            println!("\n{}", "Current Configuration:".bold());
            if config.api_key.is_some() || config::secure_storage::get_secret("RCO_API_KEY")?.is_some() {
                println!("{}", "🔑 API key is configured".green());
                
                // Show which storage method is being used
                if config::secure_storage::is_available() && config::secure_storage::get_secret("RCO_API_KEY")?.is_some() {
                    println!("{}", "   Stored securely in system keychain".dimmed());
                } else if config.api_key.is_some() {
                    println!("{}", "   Stored in configuration file".dimmed());
                }
            } else {
                println!("{}", "❌ No API key configured".red());
                println!("{}", "   Run: rco config set RCO_API_KEY=<your_key>".dimmed());
            }
            
            // Show AI provider
            if let Some(provider) = &config.ai_provider {
                println!("🤖 AI Provider: {}", provider);
            }
        }
    }

    Ok(())
}

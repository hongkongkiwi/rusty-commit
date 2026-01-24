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
                    eprintln!("{}", format!("Invalid format: {pair}. Use KEY=value").red());
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
                println!(
                    "{}",
                    "   Your API keys are encrypted and protected by your system".dimmed()
                );

                // Platform-specific information
                #[cfg(target_os = "macos")]
                println!(
                    "{}",
                    "   Stored in: macOS Keychain (login keychain)".dimmed()
                );

                #[cfg(target_os = "linux")]
                println!(
                    "{}",
                    "   Stored in: Secret Service (GNOME Keyring/KWallet)".dimmed()
                );

                #[cfg(target_os = "windows")]
                println!("{}", "   Stored in: Windows Credential Manager".dimmed());
            } else {
                println!(
                    "\n{}",
                    "⚠️  API keys will be stored in the configuration file".yellow()
                );
                println!(
                    "{}",
                    "   Location: ~/.config/rustycommit/config.toml".dimmed()
                );

                #[cfg(not(feature = "secure-storage"))]
                {
                    println!("{}", "   To enable secure storage:".dimmed());
                    println!(
                        "{}",
                        "   cargo install rustycommit --features secure-storage".dimmed()
                    );
                }

                #[cfg(feature = "secure-storage")]
                {
                    println!(
                        "{}",
                        "   Note: Secure storage is not available on this system".dimmed()
                    );
                    println!("{}", "   Falling back to file-based storage".dimmed());
                }
            }

            // Show current API key status
            println!("\n{}", "Current Configuration:".bold());
            if config.api_key.is_some()
                || config::secure_storage::get_secret("RCO_API_KEY")?.is_some()
            {
                println!("{}", "🔑 API key is configured".green());

                // Show which storage method is being used
                if config::secure_storage::is_available()
                    && config::secure_storage::get_secret("RCO_API_KEY")?.is_some()
                {
                    println!("{}", "   Stored securely in system keychain".dimmed());
                } else if config.api_key.is_some() {
                    println!("{}", "   Stored in configuration file".dimmed());
                }
            } else {
                println!("{}", "❌ No API key configured".red());
                println!(
                    "{}",
                    "   Run: rco config set RCO_API_KEY=<your_key>".dimmed()
                );
            }

            // Show AI provider
            if let Some(provider) = &config.ai_provider {
                println!("🤖 AI Provider: {}", provider);
            }
        }
        ConfigAction::Describe => {
            println!("\n{}", "📖 Configuration Options".bold());
            println!("{}", "═".repeat(60).dimmed());

            println!("\n{}", "Core Settings:".bold().green());
            println!("  RCO_AI_PROVIDER    AI provider to use (openai, anthropic, ollama, etc.)");
            println!("  RCO_MODEL          Model name for the provider");
            println!("  RCO_API_KEY        API key for the provider");
            println!("  RCO_API_URL        Custom API endpoint URL");

            println!("\n{}", "Commit Style:".bold().green());
            println!("  RCO_COMMIT_TYPE    Format: 'conventional' or 'gitmoji'");
            println!("  RCO_EMOJI          Include emojis: true/false");
            println!("  RCO_LANGUAGE       Output language (en, es, fr, etc.)");
            println!("  RCO_DESCRIPTION    Include description: true/false");

            println!("\n{}", "Behavior:".bold().green());
            println!("  RCO_TOKENS_MAX_INPUT   Max input tokens (default: 4096)");
            println!("  RCO_TOKENS_MAX_OUTPUT  Max output tokens (default: 500)");
            println!("  RCO_GITPUSH      Auto-push after commit: true/false");
            println!("  RCO_ONE_LINE_COMMIT    One-line format: true/false");

            println!("\n{}", "Hooks:".bold().green());
            println!("  RCO_PRE_GEN_HOOK       Command to run before generation");
            println!("  RCO_PRE_COMMIT_HOOK    Command to run after generation");
            println!("  RCO_POST_COMMIT_HOOK   Command to run after commit");
            println!("  RCO_HOOK_STRICT        Fail on hook error: true/false");
            println!("  RCO_HOOK_TIMEOUT_MS    Hook timeout in milliseconds");

            println!("\n{}", "Examples:".bold().green());
            println!("  rco config set RCO_AI_PROVIDER=anthropic");
            println!("  rco config set RCO_MODEL=claude-3-5-haiku-20241022");
            println!("  rco config set RCO_EMOJI=true RCO_LANGUAGE=es");
            println!("  rco config set RCO_PRE_GEN_HOOK='just lint'");

            println!("\n{}", "═".repeat(60).dimmed());
        }
        ConfigAction::AddProvider { provider, alias } => {
            println!("\n{}", "🔧 Add Provider Wizard".bold().green());
            println!("{}", "═".repeat(50).dimmed());

            let provider = provider.unwrap_or_else(|| {
                println!("\nAvailable providers:");
                println!("  - openai      (OpenAI API, Codex OAuth)");
                println!("  - anthropic   (Anthropic Claude)");
                println!("  - claude-code (Anthropic Claude Code OAuth)");
                println!("  - qwen        (Qwen AI)");
                println!("  - ollama      (Ollama local models)");
                println!("  - xai         (xAI Grok)");
                println!("  - gemini      (Google Gemini)");
                println!("  - perplexity  (Perplexity)");
                println!("  - azure       (Azure OpenAI)");
                println!("\nEnter provider name: ");
                "openai".to_string()
            });

            let alias = alias.unwrap_or_else(|| format!("{}-default", provider));

            println!("\nTo add provider '{alias}' with provider '{provider}':");
            println!("  1. API Key: rco config set RCO_API_KEY=<your_key>");
            println!("  2. Set as default: rco config use-account {alias}");
            println!("\nNote: Full interactive setup coming in a future update");
        }
        ConfigAction::ListAccounts => {
            println!("\n{}", "📋 Configured Accounts".bold().green());
            println!("{}", "═".repeat(50).dimmed());

            if config.has_accounts() {
                match config.list_accounts() {
                    Ok(accounts) => {
                        for account in accounts {
                            let default_marker = if account.is_default {
                                " [DEFAULT]".bold().green()
                            } else {
                                "".normal()
                            };
                            println!(
                                "{}: {}{}",
                                account.alias.yellow(),
                                account.provider,
                                default_marker
                            );
                            if let Some(model) = &account.model {
                                println!("   Model: {}", model.dimmed());
                            }
                            if let Some(api_url) = &account.api_url {
                                println!("   URL: {}", api_url.dimmed());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", format!("❌ Failed to list accounts: {e}").red());
                    }
                }
            } else {
                println!("\n{}", "No accounts configured yet.".dimmed());
                println!("{}", "Use: rco config add-provider to add an account".dimmed());
            }
        }
        ConfigAction::UseAccount { alias } => {
            println!("\n{}", format!("🔄 Switching to account: {}", alias).bold().green());

            match config.set_default_account(&alias) {
                Ok(_) => {
                    println!("{}", format!("✅ Now using account: {alias}").green());
                    println!("\n{}", "Note: Account switching requires restart of commands".dimmed());
                }
                Err(e) => {
                    eprintln!("{}", format!("❌ Failed to switch account: {e}").red());
                }
            }
        }
        ConfigAction::RemoveAccount { alias } => {
            println!("\n{}", format!("🗑️  Removing account: {}", alias).bold().yellow());

            match config.remove_account(&alias) {
                Ok(_) => {
                    println!("{}", format!("✅ Account '{alias}' removed").green());
                }
                Err(e) => {
                    eprintln!("{}", format!("❌ Failed to remove account: {e}").red());
                }
            }
        }
        ConfigAction::ShowAccount { alias } => {
            let alias = alias.as_deref().unwrap_or("default");

            println!("\n{}", format!("👤 Account: {}", alias).bold().green());
            println!("{}", "═".repeat(50).dimmed());

            match config.get_account(alias) {
                Ok(Some(account)) => {
                    println!("Alias: {}", account.alias.yellow());
                    println!("Provider: {}", account.provider);
                    println!("Default: {}", if account.is_default { "Yes" } else { "No" });

                    if let Some(model) = &account.model {
                        println!("Model: {}", model);
                    }
                    if let Some(api_url) = &account.api_url {
                        println!("API URL: {}", api_url);
                    }

                    match &account.auth {
                        crate::config::accounts::AuthMethod::ApiKey { .. } => {
                            println!("Auth: API Key 🔑");
                        }
                        crate::config::accounts::AuthMethod::OAuth { provider, account_id } => {
                            println!("Auth: OAuth ({}) - Account: {}", provider, account_id);
                        }
                        crate::config::accounts::AuthMethod::EnvVar { name } => {
                            println!("Auth: Environment Variable ({})", name);
                        }
                        crate::config::accounts::AuthMethod::Bearer { .. } => {
                            println!("Auth: Bearer Token 🔖");
                        }
                    }
                }
                Ok(None) => {
                    eprintln!("{}", format!("❌ Account '{alias}' not found").red());
                }
                Err(e) => {
                    eprintln!("{}", format!("❌ Failed to get account: {e}").red());
                }
            }
        }
    }

    Ok(())
}

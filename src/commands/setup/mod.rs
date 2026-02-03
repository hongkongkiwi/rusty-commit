use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::cli::SetupCommand;
use crate::config::Config;

#[cfg(feature = "tui")]
mod ratatui;

/// Provider option for the setup wizard
struct ProviderOption {
    name: &'static str,
    display: &'static str,
    default_model: &'static str,
    requires_key: bool,
    category: ProviderCategory,
}

#[derive(Clone, Copy, PartialEq)]
enum ProviderCategory {
    Popular,
    Local,
    Cloud,
    Enterprise,
    Specialized,
}

impl ProviderOption {
    fn all() -> Vec<Self> {
        vec![
            // ═════════════════════════════════════════════════════════════════
            // Popular providers
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "openai",
                display: "OpenAI (GPT-4o, GPT-4o-mini, GPT-5)",
                default_model: "gpt-4o-mini",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            ProviderOption {
                name: "anthropic",
                display: "Anthropic (Claude 3.5/4 Sonnet, Haiku, Opus)",
                default_model: "claude-3-5-haiku-20241022",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            ProviderOption {
                name: "gemini",
                display: "Google Gemini (2.5 Flash, 2.5 Pro)",
                default_model: "gemini-2.5-flash",
                requires_key: true,
                category: ProviderCategory::Popular,
            },
            // ═════════════════════════════════════════════════════════════════
            // Local/Self-hosted
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "ollama",
                display: "Ollama (Local models - free, private)",
                default_model: "llama3.2",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            ProviderOption {
                name: "lmstudio",
                display: "LM Studio (Local GUI for LLMs)",
                default_model: "local-model",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            ProviderOption {
                name: "llamacpp",
                display: "llama.cpp (Local inference)",
                default_model: "local-model",
                requires_key: false,
                category: ProviderCategory::Local,
            },
            // ═════════════════════════════════════════════════════════════════
            // Cloud providers - Fast Inference
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "groq",
                display: "Groq (Ultra-fast inference)",
                default_model: "llama-3.3-70b-versatile",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "cerebras",
                display: "Cerebras (Fast inference)",
                default_model: "llama-3.3-70b",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "sambanova",
                display: "SambaNova (Fast inference)",
                default_model: "Meta-Llama-3.3-70B-Instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "nebius",
                display: "Nebius (GPU cloud inference)",
                default_model: "meta-llama/Llama-3.3-70B-Instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // ═════════════════════════════════════════════════════════════════
            // Cloud providers - General
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "xai",
                display: "xAI (Grok)",
                default_model: "grok-2",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "deepseek",
                display: "DeepSeek (V3, R1 Reasoner)",
                default_model: "deepseek-chat",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "openrouter",
                display: "OpenRouter (Access 100+ models)",
                default_model: "anthropic/claude-3.5-haiku",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "mistral",
                display: "Mistral AI",
                default_model: "mistral-small-latest",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "perplexity",
                display: "Perplexity AI",
                default_model: "sonar",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "together",
                display: "Together AI",
                default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "fireworks",
                display: "Fireworks AI",
                default_model: "accounts/fireworks/models/llama-v3p3-70b-instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "replicate",
                display: "Replicate",
                default_model: "meta/meta-llama-3-70b-instruct",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // ═════════════════════════════════════════════════════════════════
            // Enterprise
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "azure",
                display: "Azure OpenAI",
                default_model: "gpt-4o",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "bedrock",
                display: "AWS Bedrock",
                default_model: "anthropic.claude-3-haiku-20240307-v1:0",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "vertex",
                display: "Google Vertex AI",
                default_model: "gemini-2.5-flash-001",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "cohere",
                display: "Cohere",
                default_model: "command-r",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            ProviderOption {
                name: "ai21",
                display: "AI21 Labs (Jamba)",
                default_model: "jamba-1.5-mini",
                requires_key: true,
                category: ProviderCategory::Enterprise,
            },
            // ═════════════════════════════════════════════════════════════════
            // China-based Providers
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "siliconflow",
                display: "SiliconFlow (China)",
                default_model: "deepseek-ai/DeepSeek-V3",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "zhipu",
                display: "Zhipu AI / ChatGLM (China)",
                default_model: "glm-4-flash",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            ProviderOption {
                name: "moonshot",
                display: "Moonshot AI / Kimi (China)",
                default_model: "moonshot-v1-8k",
                requires_key: true,
                category: ProviderCategory::Cloud,
            },
            // ═════════════════════════════════════════════════════════════════
            // Specialized Providers
            // ═════════════════════════════════════════════════════════════════
            ProviderOption {
                name: "jina",
                display: "Jina AI (Embeddings & LLMs)",
                default_model: "jina-embeddings-v3",
                requires_key: true,
                category: ProviderCategory::Specialized,
            },
            ProviderOption {
                name: "helicone",
                display: "Helicone (LLM Observability)",
                default_model: "gpt-4o-mini",
                requires_key: true,
                category: ProviderCategory::Specialized,
            },
        ]
    }

    #[allow(dead_code)]
    fn by_name(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|p| p.name == name)
    }
}

/// Commit format options
#[derive(Clone, Copy)]
enum CommitFormat {
    Conventional,
    Gitmoji,
    Simple,
}

impl CommitFormat {
    fn display(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "Conventional Commits (feat:, fix:, docs:, etc.)",
            CommitFormat::Gitmoji => "GitMoji (✨ feat:, 🐛 fix:, 📝 docs:, etc.)",
            CommitFormat::Simple => "Simple (no prefix)",
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "conventional",
            CommitFormat::Gitmoji => "gitmoji",
            CommitFormat::Simple => "simple",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            CommitFormat::Conventional,
            CommitFormat::Gitmoji,
            CommitFormat::Simple,
        ]
    }
}

/// Language options
struct LanguageOption {
    code: &'static str,
    display: &'static str,
}

impl LanguageOption {
    fn all() -> Vec<Self> {
        vec![
            LanguageOption {
                code: "en",
                display: "English",
            },
            LanguageOption {
                code: "zh",
                display: "Chinese (中文)",
            },
            LanguageOption {
                code: "es",
                display: "Spanish (Español)",
            },
            LanguageOption {
                code: "fr",
                display: "French (Français)",
            },
            LanguageOption {
                code: "de",
                display: "German (Deutsch)",
            },
            LanguageOption {
                code: "ja",
                display: "Japanese (日本語)",
            },
            LanguageOption {
                code: "ko",
                display: "Korean (한국어)",
            },
            LanguageOption {
                code: "ru",
                display: "Russian (Русский)",
            },
            LanguageOption {
                code: "pt",
                display: "Portuguese (Português)",
            },
            LanguageOption {
                code: "it",
                display: "Italian (Italiano)",
            },
            LanguageOption {
                code: "other",
                display: "Other (specify)",
            },
        ]
    }
}

pub async fn execute(cmd: SetupCommand) -> Result<()> {
    // Check if TUI should be used
    let use_tui = !cmd.no_tui && atty::is(atty::Stream::Stdout);

    if use_tui {
        #[cfg(feature = "tui")]
        {
            // Use the TUI interface
            use crate::commands::setup::ratatui::tui_main;
            tui_main().await?;
            return Ok(());
        }

        #[cfg(not(feature = "tui"))]
        {
            tracing::warn!("TUI feature not enabled, falling back to dialoguer");
        }
    }

    print_welcome_header();

    // Determine if we're doing quick or advanced setup
    let is_advanced = if cmd.defaults {
        // Non-interactive defaults mode
        return apply_defaults().await;
    } else if cmd.advanced {
        true
    } else {
        // Ask user which mode they prefer
        println!();
        println!("{}", "Choose your setup mode:".bold());
        println!();

        let modes = vec![
            "🚀 Quick Setup - Just the essentials (recommended)",
            "⚙️  Advanced Setup - Full configuration options",
        ];

        let selection = Select::new()
            .with_prompt("Select mode")
            .items(&modes)
            .default(0)
            .interact()?;

        selection == 1
    };

    if is_advanced {
        run_advanced_setup().await
    } else {
        run_quick_setup().await
    }
}

fn print_welcome_header() {
    println!();
    println!(
        "{} {}",
        "🚀".green(),
        "Welcome to Rusty Commit Setup!".bold().white()
    );
    println!();
    println!(
        "{}",
        "   Let's get you set up with AI-powered commit messages.".dimmed()
    );
    println!();
}

async fn run_quick_setup() -> Result<()> {
    let mut config = Config::load()?;

    // Step 1: Provider selection
    let provider = select_provider_quick()?;
    config.ai_provider = Some(provider.name.to_string());
    config.model = Some(provider.default_model.to_string());

    // Step 2: API key (if needed)
    if provider.requires_key {
        let api_key = prompt_for_api_key(provider.name)?;
        if !api_key.is_empty() {
            config.api_key = Some(api_key);
        }
    } else {
        println!();
        println!(
            "{} {} doesn't require an API key - great for privacy!",
            "ℹ️".blue(),
            provider.name.bright_cyan()
        );
    }

    // Step 3: Commit format
    let format = select_commit_format()?;
    config.commit_type = Some(format.as_str().to_string());
    config.emoji = Some(matches!(format, CommitFormat::Gitmoji));

    // Save configuration
    config.save()?;

    print_completion_message(&config, false);
    Ok(())
}

async fn run_advanced_setup() -> Result<()> {
    let mut config = Config::load()?;

    // Section 1: AI Provider Configuration
    print_section_header("🤖 AI Provider Configuration");

    let provider = select_provider_advanced()?;
    config.ai_provider = Some(provider.name.to_string());

    // Custom model selection
    let default_model = provider.default_model;
    let use_custom_model = Confirm::new()
        .with_prompt(format!(
            "Use default model ({}), or specify a custom one?",
            default_model.bright_cyan()
        ))
        .default(true)
        .interact()?;

    if use_custom_model {
        config.model = Some(default_model.to_string());
    } else {
        let custom_model: String = Input::new()
            .with_prompt("Enter model name")
            .default(default_model.to_string())
            .interact()?;
        config.model = Some(custom_model);
    }

    // API key or custom endpoint
    if provider.requires_key {
        let api_key = prompt_for_api_key(provider.name)?;
        if !api_key.is_empty() {
            config.api_key = Some(api_key);
        }

        // Custom API URL option
        let use_custom_url = Confirm::new()
            .with_prompt("Use a custom API endpoint URL?")
            .default(false)
            .interact()?;

        if use_custom_url {
            let custom_url: String = Input::new()
                .with_prompt("Enter custom API URL")
                .default(format!("https://api.{}.com/v1", provider.name))
                .interact()?;
            config.api_url = Some(custom_url);
        }
    }

    // Section 2: Commit Message Style
    print_section_header("📝 Commit Message Style");

    let format = select_commit_format()?;
    config.commit_type = Some(format.as_str().to_string());
    config.emoji = Some(matches!(format, CommitFormat::Gitmoji));

    // Capitalization
    config.description_capitalize = Some(
        Confirm::new()
            .with_prompt("Capitalize the first letter of commit messages?")
            .default(true)
            .interact()?,
    );

    // Period at end
    config.description_add_period = Some(
        Confirm::new()
            .with_prompt("Add period at the end of commit messages?")
            .default(false)
            .interact()?,
    );

    // Max length
    let max_length: usize = Input::new()
        .with_prompt("Maximum commit message length")
        .default(100)
        .validate_with(|input: &usize| -> Result<(), &str> {
            if *input >= 50 && *input <= 200 {
                Ok(())
            } else {
                Err("Please enter a value between 50 and 200")
            }
        })
        .interact()?;
    config.description_max_length = Some(max_length);

    // Language selection
    let language = select_language()?;
    config.language = Some(language.to_string());

    // Section 3: Behavior Settings
    print_section_header("⚙️  Behavior Settings");

    // Generate count
    let generate_count: u8 = Input::new()
        .with_prompt("Number of commit variations to generate (1-5)")
        .default(1)
        .validate_with(|input: &u8| -> Result<(), &str> {
            if *input >= 1 && *input <= 5 {
                Ok(())
            } else {
                Err("Please enter a value between 1 and 5")
            }
        })
        .interact()?;
    config.generate_count = Some(generate_count);

    // Git push option
    config.gitpush = Some(
        Confirm::new()
            .with_prompt("Automatically push commits to remote?")
            .default(false)
            .interact()?,
    );

    // One-line commits
    config.one_line_commit = Some(
        Confirm::new()
            .with_prompt("Always use one-line commits (no body)?")
            .default(false)
            .interact()?,
    );

    // Enable commit body
    config.enable_commit_body = Some(
        Confirm::new()
            .with_prompt("Allow multi-line commit messages with body?")
            .default(false)
            .interact()?,
    );

    // Section 4: Advanced Features
    print_section_header("🔧 Advanced Features");

    // Learn from history
    config.learn_from_history = Some(
        Confirm::new()
            .with_prompt("Learn commit style from repository history?")
            .default(false)
            .interact()?,
    );

    if config.learn_from_history == Some(true) {
        let history_count: usize = Input::new()
            .with_prompt("Number of commits to analyze for style")
            .default(50)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input >= 10 && *input <= 200 {
                    Ok(())
                } else {
                    Err("Please enter a value between 10 and 200")
                }
            })
            .interact()?;
        config.history_commits_count = Some(history_count);
    }

    // Clipboard on timeout
    config.clipboard_on_timeout = Some(
        Confirm::new()
            .with_prompt("Copy commit message to clipboard on timeout/error?")
            .default(true)
            .interact()?,
    );

    // Hook settings
    config.hook_strict = Some(
        Confirm::new()
            .with_prompt("Strict hook mode (fail on hook errors)?")
            .default(true)
            .interact()?,
    );

    let hook_timeout: u64 = Input::new()
        .with_prompt("Hook timeout (milliseconds)")
        .default(30000)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input >= 1000 && *input <= 300000 {
                Ok(())
            } else {
                Err("Please enter a value between 1000 and 300000")
            }
        })
        .interact()?;
    config.hook_timeout_ms = Some(hook_timeout);

    // Section 5: Token Limits (for advanced users)
    print_section_header("🎯 Token Limits (Optional)");

    let configure_tokens = Confirm::new()
        .with_prompt("Configure token limits? (Most users can skip this)")
        .default(false)
        .interact()?;

    if configure_tokens {
        let max_input: usize = Input::new()
            .with_prompt("Maximum input tokens")
            .default(4096)
            .interact()?;
        config.tokens_max_input = Some(max_input);

        let max_output: u32 = Input::new()
            .with_prompt("Maximum output tokens")
            .default(500)
            .interact()?;
        config.tokens_max_output = Some(max_output);
    }

    // Save configuration
    config.save()?;

    print_completion_message(&config, true);
    Ok(())
}

fn select_provider_quick() -> Result<ProviderOption> {
    println!();
    println!("{}", "Select your AI provider:".bold());
    println!(
        "{}",
        "   This determines which AI will generate your commit messages.".dimmed()
    );
    println!();

    let providers = ProviderOption::all();
    let _popular: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Popular)
        .map(|p| p.display)
        .collect();

    let _local: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Local)
        .map(|p| p.display)
        .collect();

    let _cloud: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Cloud)
        .map(|p| p.display)
        .collect();

    let _enterprise: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Enterprise)
        .map(|p| p.display)
        .collect();

    let specialized: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Specialized)
        .map(|p| p.display)
        .collect();

    let mut all_displays = Vec::new();
    let mut provider_indices: Vec<usize> = Vec::new();

    // Popular section
    all_displays.push("─── Popular Providers ───".dimmed().to_string());
    for (idx, p) in providers.iter().enumerate() {
        if p.category == ProviderCategory::Popular {
            all_displays.push(p.display.to_string());
            provider_indices.push(idx);
        }
    }

    // Local section
    all_displays.push("─── Local/Private ───".dimmed().to_string());
    for (idx, p) in providers.iter().enumerate() {
        if p.category == ProviderCategory::Local {
            all_displays.push(p.display.to_string());
            provider_indices.push(idx);
        }
    }

    // Cloud section
    all_displays.push("─── Cloud Providers ───".dimmed().to_string());
    for (idx, p) in providers.iter().enumerate() {
        if p.category == ProviderCategory::Cloud {
            all_displays.push(p.display.to_string());
            provider_indices.push(idx);
        }
    }

    // Enterprise section
    all_displays.push("─── Enterprise ───".dimmed().to_string());
    for (idx, p) in providers.iter().enumerate() {
        if p.category == ProviderCategory::Enterprise {
            all_displays.push(p.display.to_string());
            provider_indices.push(idx);
        }
    }

    // Specialized section
    if !specialized.is_empty() {
        all_displays.push("─── Specialized ───".dimmed().to_string());
        for (idx, p) in providers.iter().enumerate() {
            if p.category == ProviderCategory::Specialized {
                all_displays.push(p.display.to_string());
                provider_indices.push(idx);
            }
        }
    }

    let selection = Select::new()
        .with_prompt("AI Provider")
        .items(&all_displays)
        .default(1) // First real item (after header)
        .interact()?;

    // Count headers before the selected item to determine actual provider index
    let header_count = all_displays[..=selection]
        .iter()
        .filter(|s| s.starts_with('─'))
        .count();

    let provider_idx = if selection > 0 {
        selection.saturating_sub(header_count)
    } else {
        0
    };

    let provider = if provider_idx < provider_indices.len() {
        providers
            .into_iter()
            .nth(provider_indices[provider_idx])
            .unwrap()
    } else {
        // Fallback to first popular provider
        providers
            .into_iter()
            .find(|p| p.category == ProviderCategory::Popular)
            .unwrap()
    };

    println!();
    println!(
        "{} Selected: {} {}",
        "✓".green(),
        provider.name.bright_cyan(),
        format!("(model: {})", provider.default_model).dimmed()
    );

    Ok(provider)
}

fn select_provider_advanced() -> Result<ProviderOption> {
    println!();
    println!("{}", "Select your AI provider:".bold());
    println!();

    let providers = ProviderOption::all();
    let items: Vec<_> = providers.iter().map(|p| p.display).collect();

    let selection = Select::new()
        .with_prompt("AI Provider")
        .items(&items)
        .default(0)
        .interact()?;

    let provider = providers.into_iter().nth(selection).unwrap();

    println!();
    println!("{} Selected: {}", "✓".green(), provider.name.bright_cyan());

    Ok(provider)
}

fn prompt_for_api_key(provider_name: &str) -> Result<String> {
    println!();
    println!("{}", "API Key Configuration".bold());
    println!(
        "{}",
        format!(
            "   Get your API key from the {} dashboard",
            provider_name.bright_cyan()
        )
        .dimmed()
    );
    println!(
        "{}",
        "   Your key will be stored securely in your system's keychain.".dimmed()
    );
    println!();

    let api_key: String = Input::new()
        .with_prompt(format!(
            "Enter your {} API key",
            provider_name.bright_cyan()
        ))
        .allow_empty(true)
        .interact()?;

    let trimmed = api_key.trim();

    if trimmed.is_empty() {
        println!();
        println!(
            "{} No API key provided. You can set it later with: {}",
            "⚠️".yellow(),
            "rco config set RCO_API_KEY=<your_key>".bright_cyan()
        );
    } else {
        // Show last 4 characters for confirmation
        let masked = if trimmed.len() > 4 {
            format!("{}****", &trimmed[trimmed.len() - 4..])
        } else {
            "****".to_string()
        };
        println!();
        println!("{} API key saved: {}", "✓".green(), masked.dimmed());
    }

    Ok(trimmed.to_string())
}

fn select_commit_format() -> Result<CommitFormat> {
    println!();
    println!("{}", "Commit Message Format".bold());
    println!(
        "{}",
        "   Choose how your commit messages should be formatted.".dimmed()
    );
    println!();

    let formats = CommitFormat::all();
    let items: Vec<_> = formats.iter().map(|f| f.display()).collect();

    let selection = Select::new()
        .with_prompt("Commit format")
        .items(&items)
        .default(0)
        .interact()?;

    let format = formats.into_iter().nth(selection).unwrap();

    println!();
    println!(
        "{} Selected: {}",
        "✓".green(),
        format.as_str().bright_cyan()
    );

    // Show example
    let example = match format {
        CommitFormat::Conventional => "feat(auth): Add login functionality",
        CommitFormat::Gitmoji => "✨ feat(auth): Add login functionality",
        CommitFormat::Simple => "Add login functionality",
    };
    println!("  Example: {}", example.dimmed());

    Ok(format)
}

fn select_language() -> Result<String> {
    println!();
    println!("{}", "Output Language".bold());
    println!(
        "{}",
        "   What language should commit messages be generated in?".dimmed()
    );
    println!();

    let languages = LanguageOption::all();
    let items: Vec<_> = languages.iter().map(|l| l.display).collect();

    let selection = Select::new()
        .with_prompt("Language")
        .items(&items)
        .default(0)
        .interact()?;

    let lang = &languages[selection];

    if lang.code == "other" {
        let custom: String = Input::new()
            .with_prompt("Enter language code (e.g., 'nl' for Dutch)")
            .interact()?;
        Ok(custom)
    } else {
        Ok(lang.code.to_string())
    }
}

fn print_section_header(title: &str) {
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!("{}", title.bold());
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
}

fn print_completion_message(config: &Config, is_advanced: bool) {
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("{} Setup complete! 🎉", "✓".green().bold());
    println!();

    // Show summary
    println!("{}", "Configuration Summary:".bold());
    println!();

    if let Some(provider) = &config.ai_provider {
        println!("  {} Provider: {}", "•".cyan(), provider.bright_white());
    }
    if let Some(model) = &config.model {
        println!("  {} Model: {}", "•".cyan(), model.bright_white());
    }
    if let Some(commit_type) = &config.commit_type {
        println!(
            "  {} Commit format: {}",
            "•".cyan(),
            commit_type.bright_white()
        );
    }
    if let Some(language) = &config.language {
        if language != "en" {
            println!("  {} Language: {}", "•".cyan(), language.bright_white());
        }
    }

    println!();
    println!("{} You're ready to go!", "→".cyan());
    println!();
    println!("   Try it now:  {}", "rco".bold().bright_cyan().underline());
    println!();

    if is_advanced {
        println!("   Make a commit:  {}", "git add . && rco".dimmed());
        println!();
        println!(
            "{} Modify settings anytime: {}",
            "→".cyan(),
            "rco setup --advanced".bright_cyan()
        );
        println!(
            "{} Or use: {}",
            "→".cyan(),
            "rco config set <key>=<value>".bright_cyan()
        );
    } else {
        println!(
            "   Want more options? Run: {}",
            "rco setup --advanced".bright_cyan()
        );
    }

    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
}

async fn apply_defaults() -> Result<()> {
    let mut config = Config::load()?;

    // Apply sensible defaults without prompting
    config.ai_provider = Some("openai".to_string());
    config.model = Some("gpt-4o-mini".to_string());
    config.commit_type = Some("conventional".to_string());
    config.description_capitalize = Some(true);
    config.description_add_period = Some(false);
    config.description_max_length = Some(100);
    config.language = Some("en".to_string());
    config.generate_count = Some(1);
    config.emoji = Some(false);
    config.gitpush = Some(false);
    config.one_line_commit = Some(false);
    config.enable_commit_body = Some(false);
    config.learn_from_history = Some(false);
    config.clipboard_on_timeout = Some(true);
    config.hook_strict = Some(true);
    config.hook_timeout_ms = Some(30000);
    config.tokens_max_input = Some(4096);
    config.tokens_max_output = Some(500);

    config.save()?;

    println!();
    println!("{} Default configuration applied!", "✓".green().bold());
    println!();
    println!("   Provider: openai (gpt-4o-mini)");
    println!("   Format: conventional commits");
    println!();
    println!(
        "   Set your API key: {}",
        "rco config set RCO_API_KEY=<your_key>".bright_cyan()
    );
    println!();

    Ok(())
}

//! Selection helper functions for setup wizard

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};

use super::providers::{CommitFormat, LanguageOption, ProviderCategory, ProviderOption};

/// Prompt user for API key
pub fn prompt_for_api_key(provider_name: &str) -> Result<String> {
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

/// Select commit format
pub fn select_commit_format() -> Result<CommitFormat> {
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

/// Select output language
pub fn select_language() -> Result<String> {
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

/// Select provider with categorized display (quick mode)
pub fn select_provider_quick() -> Result<ProviderOption> {
    println!();
    println!("{}", "Select your AI provider:".bold());
    println!(
        "{}",
        "   This determines which AI will generate your commit messages.".dimmed()
    );
    println!();

    let providers = ProviderOption::all();

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
    let specialized: Vec<_> = providers
        .iter()
        .filter(|p| p.category == ProviderCategory::Specialized)
        .collect();

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

/// Select provider with simple list (advanced mode)
pub fn select_provider_advanced() -> Result<ProviderOption> {
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

//! UI utilities for setup wizard

use colored::Colorize;

/// Print a section header with decorative border
pub fn print_section_header(title: &str) {
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

/// Print welcome header
pub fn print_welcome_header() {
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

/// Print completion message with configuration summary
pub fn print_completion_message(
    ai_provider: &str,
    model: &str,
    commit_type: &str,
    language: &str,
    is_advanced: bool,
) {
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("{} Setup complete!", "✓".green().bold());
    println!();

    // Show summary
    println!("{}", "Configuration Summary:".bold());
    println!();

    println!("  {} Provider: {}", "•".cyan(), ai_provider.bright_white());
    println!("  {} Model: {}", "•".cyan(), model.bright_white());
    println!(
        "  {} Commit format: {}",
        "•".cyan(),
        commit_type.bright_white()
    );
    if language != "en" {
        println!("  {} Language: {}", "•".cyan(), language.bright_white());
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

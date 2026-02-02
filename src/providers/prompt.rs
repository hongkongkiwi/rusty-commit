//! Prompt building utilities for commit message generation
//!
//! This module contains the core prompt construction logic used by all providers.
//! It handles system prompts, user prompts, file type categorization, and project context.

use crate::config::Config;
use crate::git;
use crate::utils::commit_style::CommitStyleProfile;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

/// Split the prompt into system and user parts for providers that support it
pub fn split_prompt(
    diff: &str,
    context: Option<&str>,
    config: &Config,
    full_gitmoji: bool,
) -> (String, String) {
    let system_prompt = build_system_prompt(config, full_gitmoji);
    let user_prompt = build_user_prompt(diff, context, full_gitmoji, config);
    (system_prompt, user_prompt)
}

/// Build the system prompt part (role definition, rules)
pub fn build_system_prompt(config: &Config, full_gitmoji: bool) -> String {
    let mut prompt = String::new();

    prompt.push_str("You are an expert at writing clear, concise git commit messages.\n\n");

    // Core constraints
    prompt.push_str("OUTPUT RULES:\n");
    prompt.push_str("- Return ONLY the commit message, with no additional explanation, markdown formatting, or code blocks\n");
    prompt.push_str("- Do not include any reasoning, thinking, analysis, <thinking> tags, or XML-like tags in your response\n");
    prompt.push_str("- Never explain your choices or provide commentary\n");
    prompt.push_str(
        "- If you cannot generate a meaningful commit message, return \"chore: update\"\n\n",
    );

    // Add style guidance from history if enabled
    if config.learn_from_history.unwrap_or(false) {
        if let Some(style_guidance) = get_style_guidance(config) {
            prompt.push_str("REPO STYLE (learned from commit history):\n");
            prompt.push_str(&style_guidance);
            prompt.push('\n');
        }
    }

    // Add locale if specified
    if let Some(locale) = &config.language {
        prompt.push_str(&format!(
            "- Generate the commit message in {} language\n",
            locale
        ));
    }

    // Add commit type preference
    let commit_type = config.commit_type.as_deref().unwrap_or("conventional");
    match commit_type {
        "conventional" => {
            prompt.push_str("- Use conventional commit format: <type>(<scope>): <description>\n");
            prompt.push_str(
                "- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore\n",
            );
            if config.omit_scope.unwrap_or(false) {
                prompt.push_str("- Omit the scope, use format: <type>: <description>\n");
            }
        }
        "gitmoji" => {
            if full_gitmoji {
                prompt.push_str("- Use GitMoji format with full emoji specification from https://gitmoji.dev/\n");
                prompt.push_str("- Common emojis: ✨(feat), 🐛(fix), 📝(docs), 🚀(deploy), ♻️(refactor), ✅(test), 🔧(chore), ⚡(perf), 🎨(style), 📦(build), 👷(ci)\n");
                prompt.push_str("- For breaking changes, add 💥 after the type\n");
            } else {
                prompt.push_str("- Use GitMoji format: <emoji> <type>: <description>\n");
                prompt.push_str("- Common emojis: 🐛(fix), ✨(feat), 📝(docs), 🚀(deploy), ✅(test), ♻️(refactor), 🔧(chore), ⚡(perf), 🎨(style), 📦(build), 👷(ci)\n");
            }
        }
        _ => {}
    }

    // Description requirements
    let max_length = config.description_max_length.unwrap_or(100);
    prompt.push_str(&format!(
        "- Keep the description under {} characters\n",
        max_length
    ));

    if config.description_capitalize.unwrap_or(true) {
        prompt.push_str("- Capitalize the first letter of the description\n");
    }

    if !config.description_add_period.unwrap_or(false) {
        prompt.push_str("- Do not end the description with a period\n");
    }

    // Add commit body guidance if enabled
    if config.enable_commit_body.unwrap_or(false) {
        prompt.push_str("\nCOMMIT BODY (optional):\n");
        prompt.push_str(
            "- Add a blank line after the description, then explain WHY the change was made\n",
        );
        prompt.push_str("- Use bullet points for multiple changes\n");
        prompt.push_str("- Wrap body text at 72 characters\n");
        prompt
            .push_str("- Focus on motivation and context, not what changed (that's in the diff)\n");
    }

    prompt
}

/// Get style guidance from commit history analysis
fn get_style_guidance(config: &Config) -> Option<String> {
    // Get cached style profile or analyze fresh
    if let Some(cached) = &config.style_profile {
        // Use cached profile if available
        return Some(cached.clone());
    }

    // Analyze recent commits - default now 50 for better learning
    let count = config.history_commits_count.unwrap_or(50);

    match git::get_recent_commit_messages(count) {
        Ok(commits) => {
            if commits.is_empty() {
                return None;
            }

            let profile = CommitStyleProfile::analyze_from_commits(&commits);

            // Only use profile if we have enough confident data (at least 10 commits with patterns)
            // Increased from 5 to 10 for better confidence
            if profile.is_empty() || commits.len() < 10 {
                return None;
            }

            Some(profile.to_prompt_guidance())
        }
        Err(e) => {
            tracing::warn!("Failed to get commit history for style analysis: {}", e);
            None
        }
    }
}

/// Build the user prompt part (actual task + diff)
pub fn build_user_prompt(
    diff: &str,
    context: Option<&str>,
    _full_gitmoji: bool,
    _config: &Config,
) -> String {
    let mut prompt = String::new();

    // Add project context if available
    if let Some(project_context) = get_project_context() {
        prompt.push_str(&format!("Project Context: {}\n\n", project_context));
    }

    // Add file type summary with detailed extension info
    let file_summary = extract_file_summary(diff);
    if !file_summary.is_empty() {
        prompt.push_str(&format!("Files Changed: {}\n\n", file_summary));
    }

    // Add chunk indicator with more detail if diff was chunked
    if diff.contains("---CHUNK") {
        let chunk_count = diff.matches("---CHUNK").count();
        if chunk_count > 1 {
            prompt.push_str(&format!(
                "Note: This diff was split into {} chunks due to size. Focus on the overall purpose of the changes across all chunks.\n\n",
                chunk_count
            ));
        } else {
            prompt.push_str("Note: The diff was split into chunks due to size. Focus on the overall purpose of the changes.\n\n");
        }
    }

    // Add context if provided
    if let Some(ctx) = context {
        prompt.push_str(&format!("Additional context: {}\n\n", ctx));
    }

    prompt.push_str("Generate a commit message for the following git diff:\n");
    prompt.push_str("```diff\n");
    prompt.push_str(diff);
    prompt.push_str("\n```\n");

    // Add reminder about output format
    prompt.push_str("\nRemember: Return ONLY the commit message, no explanations or markdown.");

    prompt
}

/// Extract file type summary from diff
pub fn extract_file_summary(diff: &str) -> String {
    let mut files: Vec<String> = Vec::new();
    let mut extensions: HashSet<String> = HashSet::new();
    let mut file_types: HashMap<String, usize> = HashMap::new();

    for line in diff.lines() {
        if line.starts_with("+++ b/") {
            let path = line.strip_prefix("+++ b/").unwrap_or(line);
            if path != "/dev/null" {
                files.push(path.to_string());
                // Extract extension and categorize
                if let Some(ext) = std::path::Path::new(path).extension() {
                    if let Some(ext_str) = ext.to_str() {
                        let ext_lower = ext_str.to_lowercase();
                        extensions.insert(ext_lower.clone());

                        // Categorize file type
                        let category = categorize_file_type(&ext_lower);
                        *file_types.entry(category).or_insert(0) += 1;
                    }
                } else {
                    // No extension - might be a config file or script
                    if path.contains("Makefile")
                        || path.contains("Dockerfile")
                        || path.contains("LICENSE")
                    {
                        *file_types.entry("config".to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    if files.is_empty() {
        return String::new();
    }

    // Build summary
    let mut summary = format!("{} file(s)", files.len());

    // Add file type categories
    if !file_types.is_empty() {
        let mut type_list: Vec<_> = file_types.into_iter().collect();
        type_list.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        let type_str: Vec<_> = type_list.iter().map(|(t, c)| format!("{} {}", c, t)).collect();
        summary.push_str(&format!(" ({})", type_str.join(", ")));
    }

    // Add extension info if not too many
    if !extensions.is_empty() && extensions.len() <= 5 {
        let ext_list: Vec<_> = extensions.into_iter().collect();
        summary.push_str(&format!(" [.{}]", ext_list.join(", .")));
    }

    // Add first few file names if small number
    if files.len() <= 3 {
        summary.push_str(&format!(": {}", files.join(", ")));
    }

    summary
}

/// Categorize file extension into a type
fn categorize_file_type(ext: &str) -> String {
    match ext {
        // Programming languages
        "rs" => "Rust",
        "py" => "Python",
        "js" => "JavaScript",
        "ts" => "TypeScript",
        "jsx" | "tsx" => "React",
        "go" => "Go",
        "java" => "Java",
        "kt" => "Kotlin",
        "swift" => "Swift",
        "c" | "cpp" | "cc" | "h" | "hpp" => "C/C++",
        "rb" => "Ruby",
        "php" => "PHP",
        "cs" => "C#",
        "scala" => "Scala",
        "r" => "R",
        "m" => "Objective-C",
        "lua" => "Lua",
        "pl" => "Perl",

        // Web
        "html" | "htm" => "HTML",
        "css" | "scss" | "sass" | "less" => "CSS",
        "vue" => "Vue",
        "svelte" => "Svelte",

        // Data/Config
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "xml" => "XML",
        "csv" => "CSV",
        "sql" => "SQL",

        // Documentation
        "md" | "markdown" => "Markdown",
        "rst" => "reStructuredText",
        "txt" => "Text",

        // Build/Config
        "sh" | "bash" | "zsh" | "fish" => "Shell",
        "ps1" => "PowerShell",
        "bat" | "cmd" => "Batch",
        "dockerfile" => "Docker",
        "makefile" | "mk" => "Make",
        "cmake" => "CMake",

        // Other
        _ => "Other",
    }
    .to_string()
}

/// Get project context from .rco/context.txt or README
pub fn get_project_context() -> Option<String> {
    // Try .rco/context.txt first
    if let Ok(repo_root) = git::get_repo_root() {
        let context_path = Path::new(&repo_root).join(".rco").join("context.txt");
        if context_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&context_path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        // Try README.md - extract first paragraph
        let readme_path = Path::new(&repo_root).join("README.md");
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                // Find first non-empty line that's not a header
                for line in content.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        // Return first sentence or up to 100 chars
                        let context = if let Some(idx) = trimmed.find('.') {
                            trimmed[..idx + 1].to_string()
                        } else {
                            trimmed.chars().take(100).collect()
                        };
                        if !context.is_empty() {
                            return Some(context);
                        }
                    }
                }
            }
        }

        // Try Cargo.toml for Rust projects
        let cargo_path = Path::new(&repo_root).join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                // Extract description from Cargo.toml
                let mut in_package = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed == "[package]" {
                        in_package = true;
                    } else if trimmed.starts_with('[') && trimmed != "[package]" {
                        in_package = false;
                    } else if in_package && trimmed.starts_with("description") {
                        if let Some(idx) = trimmed.find('=') {
                            let desc = trimmed[idx + 1..].trim().trim_matches('"');
                            if !desc.is_empty() {
                                return Some(format!("Rust project: {}", desc));
                            }
                        }
                    }
                }
            }
        }

        // Try package.json for Node projects
        let package_path = Path::new(&repo_root).join("package.json");
        if package_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
                        if !desc.is_empty() {
                            return Some(format!("Node.js project: {}", desc));
                        }
                    }
                }
            }
        }
    }

    None
}

/// Build the combined prompt for providers without system message support
pub fn build_prompt(
    diff: &str,
    context: Option<&str>,
    config: &Config,
    full_gitmoji: bool,
) -> String {
    let (system, user) = split_prompt(diff, context, config, full_gitmoji);
    format!("{}\n\n---\n\n{}", system, user)
}

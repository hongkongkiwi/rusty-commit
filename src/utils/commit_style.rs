use std::collections::HashMap;

/// Represents a learned style profile from commit history
#[derive(Debug, Default)]
pub struct CommitStyleProfile {
    /// Most common commit types used (e.g., "feat", "fix")
    pub type_frequencies: HashMap<String, usize>,
    /// Whether scopes are commonly used
    pub uses_scopes: bool,
    /// Most common scopes
    pub scope_frequencies: HashMap<String, usize>,
    /// Average description length
    pub avg_description_length: f64,
    /// Most common prefix format
    pub prefix_format: PrefixFormat,
    /// Whether gitmoji is commonly used
    pub uses_gitmoji: bool,
    /// Most common emojis used
    pub emoji_frequencies: HashMap<String, usize>,
    /// Whether descriptions typically end with periods
    pub adds_period: bool,
    /// Whether descriptions are typically capitalized
    pub capitalizes_description: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum PrefixFormat {
    #[default]
    Conventional, // feat(scope): description
    ConventionalNoScope, // feat: description
    GitMoji,             // ✨ feat: description
    GitMojiDev,          // Full gitmoji.dev format
    Simple,              // Just type: description
    Other,
}

impl CommitStyleProfile {
    /// Analyze commits and generate a style profile
    pub fn analyze_from_commits<T: AsRef<str>>(commits: &[T]) -> Self {
        let mut profile = Self::default();

        if commits.is_empty() {
            return profile;
        }

        let total = commits.len() as f64;

        // Count types, scopes, and analyze each commit
        let mut total_desc_len = 0;
        let mut desc_count = 0;
        let mut periods = 0;
        let mut capitalized = 0;

        for commit in commits {
            let commit_str = commit.as_ref();

            // Extract type
            if let Some((prefix, _)) = commit_str.split_once(':') {
                // Check for gitmoji
                if let Some(emoji) = prefix.chars().next() {
                    if is_emoji(emoji) {
                        profile.uses_gitmoji = true;
                        // Extract emoji
                        let emoji_str = commit_str
                            .chars()
                            .take_while(|c| !c.is_ascii_alphanumeric())
                            .collect::<String>();
                        if !emoji_str.is_empty() {
                            *profile
                                .emoji_frequencies
                                .entry(emoji_str.trim().to_string())
                                .or_insert(0) += 1;
                        }
                        // Get type after emoji - extract just the type before any scope
                        let type_part = prefix
                            .chars()
                            .skip_while(|c| !c.is_ascii_alphanumeric())
                            .collect::<String>();
                        // Extract just the type (before '(' if present)
                        let clean_type = if let Some((t, _)) = type_part.split_once('(') {
                            t.to_string()
                        } else {
                            type_part
                        };
                        if !clean_type.is_empty() {
                            *profile.type_frequencies.entry(clean_type).or_insert(0) += 1;
                        }
                        profile.prefix_format = PrefixFormat::GitMoji;
                    } else {
                        // No emoji, check for conventional format
                        if let Some((type_part, scope_part)) = prefix.split_once('(') {
                            profile.uses_scopes = true;
                            // Extract scope (text between '(' and ')')
                            if let Some((scope, _)) = scope_part.split_once(')') {
                                if !scope.is_empty() {
                                    *profile
                                        .scope_frequencies
                                        .entry(scope.to_string())
                                        .or_insert(0) += 1;
                                }
                            }
                            *profile
                                .type_frequencies
                                .entry(type_part.to_string())
                                .or_insert(0) += 1;
                            profile.prefix_format = PrefixFormat::Conventional;
                        } else {
                            profile.prefix_format = PrefixFormat::ConventionalNoScope;
                            *profile
                                .type_frequencies
                                .entry(prefix.to_string().trim().to_string())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }

            // Analyze description
            if let Some(desc) = commit_str.split_once(':').map(|x| x.1) {
                let desc = desc.trim();
                total_desc_len += desc.len();
                desc_count += 1;

                // Check for period at end
                if desc.ends_with('.') {
                    periods += 1;
                }

                // Check if first char is capitalized
                if let Some(first) = desc.chars().next() {
                    if first.is_ascii_uppercase() {
                        capitalized += 1;
                    }
                }
            }
        }

        // Calculate averages and percentages
        if desc_count > 0 {
            profile.avg_description_length = total_desc_len as f64 / desc_count as f64;
            profile.adds_period = (periods as f64 / total) > 0.3; // 30% threshold
            profile.capitalizes_description = (capitalized as f64 / desc_count as f64) > 0.5;
            // 50% threshold
        }

        profile
    }

    /// Generate style guidance text for the AI prompt
    pub fn to_prompt_guidance(&self) -> String {
        let mut guidance = String::new();

        // Add type guidance if we have data
        if !self.type_frequencies.is_empty() {
            let top_types: Vec<_> = self
                .type_frequencies
                .iter()
                .filter(|(t, _)| is_valid_commit_type(t))
                .take(3)
                .collect();

            if !top_types.is_empty() {
                let types_list: Vec<String> = top_types.iter().map(|(t, _)| (*t).clone()).collect();

                guidance.push_str(&format!(
                    "- Common commit types in this repo: {}\n",
                    types_list.join(", ")
                ));
            }
        }

        // Add scope guidance
        if self.uses_scopes && !self.scope_frequencies.is_empty() {
            let top_scopes: Vec<_> = self.scope_frequencies.keys().take(3).cloned().collect();

            if !top_scopes.is_empty() {
                guidance.push_str(&format!(
                    "- Common scopes in this repo: {}\n",
                    top_scopes.join(", ")
                ));
            }
        }

        // Add description length guidance
        if self.avg_description_length > 0.0 {
            let target_len = self.avg_description_length as usize;
            guidance.push_str(&format!(
                "- Keep descriptions around {} characters (based on repo style)\n",
                target_len
            ));
        }

        // Add capitalization guidance
        if self.capitalizes_description {
            guidance.push_str("- Capitalize the first letter of the description\n");
        }

        // Add period guidance
        if self.adds_period {
            guidance.push_str("- End the description with a period\n");
        } else {
            guidance.push_str("- Do not end the description with a period\n");
        }

        // Add gitmoji guidance
        if self.uses_gitmoji {
            let top_emojis: Vec<_> = self.emoji_frequencies.keys().take(3).cloned().collect();

            if !top_emojis.is_empty() {
                guidance.push_str(&format!(
                    "- Common emojis used: {} (prefer gitmoji format)\n",
                    top_emojis.join(", ")
                ));
            }
        }

        // Add prefix format guidance
        match self.prefix_format {
            PrefixFormat::Conventional => {
                guidance.push_str("- Use format: <type>(<scope>): <description>\n");
            }
            PrefixFormat::ConventionalNoScope => {
                guidance.push_str("- Use format: <type>: <description> (no scope)\n");
            }
            PrefixFormat::GitMoji => {
                guidance.push_str("- Use format: <emoji> <type>: <description>\n");
            }
            PrefixFormat::GitMojiDev => {
                guidance.push_str("- Use full gitmoji.dev format\n");
            }
            _ => {}
        }

        guidance
    }

    /// Check if profile has any meaningful data
    pub fn is_empty(&self) -> bool {
        self.type_frequencies.is_empty() && !self.uses_scopes
    }
}

/// Check if a character is an emoji
fn is_emoji(c: char) -> bool {
    // Basic check for common emoji ranges
    c as u32 > 0x1F600 || // Emoticons
    (c as u32 >= 0x1F300 && c as u32 <= 0x1F9FF) || // Misc symbols
    (c as u32 >= 0x2600 && c as u32 <= 0x26FF) || // Misc symbols
    (c as u32 >= 0x2700 && c as u32 <= 0x27BF) || // Dingbats
    (c as u32 >= 0xFE00 && c as u32 <= 0xFE0F) || // Variation selectors
    c == '🎉' || c == '🚀' || c == '✨' || c == '🐛' ||
    c == '🔥' || c == '💄' || c == '🎨' || c == '⚡' ||
    c == '🍱' || c == '🔧' || c == '🚑' || c == '🔀' ||
    c == '📝' || c == '✅' || c == '⬆' || c == '⬇'
}

/// Check if a string is a valid commit type
fn is_valid_commit_type(t: &str) -> bool {
    matches!(
        t.to_lowercase().as_str(),
        "feat"
            | "fix"
            | "docs"
            | "style"
            | "refactor"
            | "perf"
            | "test"
            | "build"
            | "ci"
            | "chore"
            | "revert"
            | "breaking"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty_commits() {
        let commits: Vec<String> = vec![];
        let profile = CommitStyleProfile::analyze_from_commits(&commits);
        assert!(profile.is_empty());
    }

    #[test]
    fn test_analyze_conventional_commits() {
        let commits = vec![
            "feat(auth): add login functionality",
            "fix(api): resolve token refresh issue",
            "docs(readme): update installation instructions",
            "feat(auth): implement logout",
        ];

        let profile = CommitStyleProfile::analyze_from_commits(&commits);

        // Should detect types
        assert!(profile.type_frequencies.contains_key("feat"));
        assert!(profile.type_frequencies.contains_key("fix"));
        assert!(profile.type_frequencies.contains_key("docs"));

        // Should detect scopes
        assert!(profile.uses_scopes);
        assert!(profile.scope_frequencies.contains_key("auth"));
        assert!(profile.scope_frequencies.contains_key("api"));

        // Should not detect gitmoji
        assert!(!profile.uses_gitmoji);
    }

    #[test]
    fn test_analyze_gitmoji_commits() {
        let commits = vec![
            "✨ feat(auth): add login functionality",
            "🐛 fix(api): resolve token refresh issue",
            "📝 docs: update installation instructions",
        ];

        let profile = CommitStyleProfile::analyze_from_commits(&commits);

        // Should detect types
        assert!(profile.type_frequencies.contains_key("feat"));
        assert!(profile.type_frequencies.contains_key("fix"));
        assert!(profile.type_frequencies.contains_key("docs"));

        // Should detect gitmoji
        assert!(profile.uses_gitmoji);
    }

    #[test]
    fn test_generate_prompt_guidance() {
        let commits = vec!["feat(auth): add login", "fix(api): resolve issue"];

        let profile = CommitStyleProfile::analyze_from_commits(&commits);
        let guidance = profile.to_prompt_guidance();

        // Should include type guidance
        assert!(guidance.contains("feat"));
        assert!(guidance.contains("fix"));

        // Should not be empty
        assert!(!guidance.is_empty());
    }
}

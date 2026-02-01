//! Utilities for stripping thinking tags from AI responses.
//!
//! This module provides functions to remove `<thinking>` tags and their content
//! from AI model outputs, which is useful for reasoning models that include
//! their thought process in the response.

use regex::Regex;

/// Strips `<thinking>` tags and their content from AI responses.
///
/// This handles various formats used by reasoning models:
/// - `<thinking>...</thinking>`
/// - `<think>...</think>`
/// - `[THINKING]...[/THINKING]`
/// - `[[THINKING]]...[[/THINKING]]`
/// - Code block variations
///
/// # Examples
///
/// ```
/// let input = "Here's the commit message:\n<thinking>I should write a clear message</thinking>\nfeat: add login";
/// let output = rusty_commit::utils::strip_thinking(input);
/// // The thinking block is removed, extra newlines are cleaned up
/// assert!(output.contains("feat: add login"));
/// ```
pub fn strip_thinking(text: &str) -> String {
    // Track content before and after thinking blocks
    let mut result = String::with_capacity(text.len());
    let mut current = text;
    let mut has_thinking = false;

    while !current.is_empty() {
        // Find the next opening tag (case-insensitive)
        let opening_pos = find_thinking_opening(current);

        if let Some(start) = opening_pos {
            // Add content before the opening tag
            result.push_str(&current[..start]);

            // Find the closing tag
            let (after_content, found) = find_and_consume_thinking_block(&current[start..]);

            if found {
                has_thinking = true;
                current = after_content;
            } else {
                // No closing tag found, include the rest
                break;
            }
        } else {
            // No more opening tags, add the rest
            result.push_str(current);
            break;
        }
    }

    // Clean up any leading/trailing whitespace from removed blocks
    if has_thinking {
        cleanup_thinking_artifacts(&result)
    } else {
        result
    }
}

/// Find the position of the opening thinking tag (case-insensitive)
fn find_thinking_opening(text: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();

    // Check for various opening tag patterns
    let patterns = [
        "<thinking>",
        "<think>",
        "[thinking]",
        "[[thinking]]",
        "```thinking",
        "<!--thinking",
    ];

    let mut min_pos = None;

    for pattern in &patterns {
        if let Some(pos) = lower.find(pattern) {
            if let Some(current_min) = min_pos {
                if pos < current_min {
                    min_pos = Some(pos);
                }
            } else {
                min_pos = Some(pos);
            }
        }
    }

    min_pos
}

/// Find and consume a thinking block, returning what's after the closing tag
fn find_and_consume_thinking_block(text: &str) -> (&str, bool) {
    let lower = text.to_ascii_lowercase();

    // Define opening and closing tag pairs
    let tag_pairs = [
        ("<thinking>", "</thinking>"),
        ("<think>", "</think>"),
        ("[thinking]", "[/thinking]"),
        ("[[thinking]]", "[[/thinking]]"),
        ("```thinking", "```"),
        ("<!--thinking", "-->"),
    ];

    // Find the earliest closing tag after an opening tag
    let mut earliest_closing: Option<usize> = None;
    let mut earliest_after_closing: Option<&str> = None;

    for (opening, closing) in &tag_pairs {
        if let Some(opening_pos) = lower.find(opening) {
            let content_after_opening = &text[opening_pos + opening.len()..];
            if let Some(closing_pos) = content_after_opening.to_ascii_lowercase().find(closing) {
                // closing_pos is relative to content_after_opening
                let absolute_closing = opening_pos + opening.len() + closing_pos;

                if let Some(current) = earliest_closing {
                    if absolute_closing < current {
                        earliest_closing = Some(absolute_closing);
                        earliest_after_closing = Some(&text[absolute_closing + closing.len()..]);
                    }
                } else {
                    earliest_closing = Some(absolute_closing);
                    earliest_after_closing = Some(&text[absolute_closing + closing.len()..]);
                }
            }
        }
    }

    if let Some(after) = earliest_after_closing {
        (after, true)
    } else {
        (text, false)
    }
}

/// Clean up artifacts left by removed thinking blocks
fn cleanup_thinking_artifacts(text: &str) -> String {
    // Remove multiple consecutive blank lines that may result from removing thinking blocks
    let re = Regex::new(r"\n\s*\n\s*\n+").unwrap();
    let result = re.replace_all(text, "\n\n");
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_thinking_basic() {
        let input = "feat: add login\n<thinking>I should write a clear message</thinking>";
        let output = strip_thinking(input);
        assert_eq!(output.trim(), "feat: add login");
    }

    #[test]
    fn test_strip_thinking_anthropic_format() {
        let input = "feat: add login\n<think> I should write a clear message</think>";
        let output = strip_thinking(input);
        assert_eq!(output.trim(), "feat: add login");
    }

    #[test]
    fn test_strip_thinking_multiple_blocks() {
        let input =
            "First part\n<thinking>block 1</thinking>\nMiddle\n<thinking>block 2</thinking>\nLast";
        let output = strip_thinking(input);
        assert!(output.contains("First part"));
        assert!(output.contains("Middle"));
        assert!(output.contains("Last"));
        assert!(!output.contains("<thinking>"));
    }

    #[test]
    fn test_strip_thinking_no_thinking() {
        let input = "feat: add login feature";
        let output = strip_thinking(input);
        assert_eq!(output, "feat: add login feature");
    }

    #[test]
    fn test_strip_thinking_unclosed_tag() {
        let input = "feat: add login\n<thinking unclosed";
        let output = strip_thinking(input);
        // Should include the unclosed tag since no closing tag was found
        assert!(output.contains("<thinking"));
    }
}

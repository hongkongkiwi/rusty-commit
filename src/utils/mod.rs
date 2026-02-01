pub mod commit_style;
pub mod hooks;
pub mod retry;
pub mod token;
pub mod version;

/// Strips `<thinking>` tags and their content from AI responses.
///
/// This handles various formats used by reasoning models:
/// - `<thinking>...</thinking>`
/// - `<think>...</think>`
/// - `[THINKING]...[/THINKING]`
/// - `[[THINKING]]...[[/THINKING]]`
/// - Any nested or multiline variations
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
    let re = regex::Regex::new(r"\n\s*\n\s*\n+").unwrap();
    let result = re.replace_all(text, "\n\n");
    result.trim().to_string()
}

// ============================================================================
// Diff Chunking Utilities
// ============================================================================

use crate::utils::token::estimate_tokens;

/// Represents a single file diff with its metadata
#[derive(Debug, Clone)]
struct FileDiff {
    /// The file path (e.g., "src/main.rs")
    path: String,
    /// The raw diff content for this file
    content: String,
    /// Estimated token count of the content
    token_count: usize,
}

/// Represents a chunk of diffs that can be sent to the AI
#[derive(Debug, Clone)]
struct DiffChunk {
    /// The diff content for this chunk
    content: String,
    /// Files included in this chunk
    files: Vec<String>,
    /// Total token count
    token_count: usize,
}

/// Parses a unified diff into individual file diffs.
///
/// Returns a vector of FileDiff, one per modified/new/deleted file.
fn parse_diff_into_files(diff: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;

    for line in diff.lines() {
        // Check if this is a new file header
        if line.starts_with("+++ b/") {
            // Save previous file if exists
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            let path = line.strip_prefix("+++ b/").unwrap_or(line).to_string();
            let tokens = estimate_tokens(&path).unwrap_or_default();

            current_file = Some(FileDiff {
                path,
                content: String::new(),
                token_count: tokens,
            });
            continue;
        }

        // Check if this is a deleted file header
        if line.starts_with("--- a/") && current_file.is_none() {
            let path = line.strip_prefix("--- a/").unwrap_or(line).to_string();
            let tokens = estimate_tokens(&path).unwrap_or_default();

            current_file = Some(FileDiff {
                path,
                content: String::new(),
                token_count: tokens,
            });
            continue;
        }

        // Add line to current file or start a new file if needed
        if let Some(ref mut file) = current_file {
            file.content.push_str(line);
            file.content.push('\n');
            file.token_count += estimate_tokens(line).unwrap_or(1);
        } else if !line.is_empty() {
            // Lines before the first file header (common diff headers)
            let mut first_file = FileDiff {
                path: "header".to_string(),
                content: line.to_string(),
                token_count: estimate_tokens(line).unwrap_or(1),
            };
            first_file.content.push('\n');
            current_file = Some(first_file);
        }
    }

    // Save the last file
    if let Some(file) = current_file {
        files.push(file);
    }

    files
}

/// Merges file diffs greedily until reaching the token limit.
///
/// This is the first level of chunking - it groups whole files together
/// to maximize context while staying under the token limit.
fn merge_diffs_into_chunks(
    files: &[FileDiff],
    max_tokens: usize,
) -> Vec<DiffChunk> {
    let mut chunks = Vec::new();
    let mut current_chunk = DiffChunk {
        content: String::new(),
        files: Vec::new(),
        token_count: 0,
    };

    for file in files {
        // Add file header overhead
        let header_overhead = estimate_tokens(&format!("diff --git a/{}", file.path)).unwrap_or(5);

        let would_exceed = if current_chunk.content.is_empty() {
            file.token_count > max_tokens
        } else {
            current_chunk.token_count + header_overhead + file.token_count > max_tokens
        };

        if would_exceed && !current_chunk.content.is_empty() {
            // Save current chunk and start a new one
            chunks.push(current_chunk);
            current_chunk = DiffChunk {
                content: String::new(),
                files: Vec::new(),
                token_count: 0,
            };
        }

        // Add file to current chunk
        if !current_chunk.content.is_empty() {
            current_chunk.content.push_str("\n");
        }
        current_chunk
            .content
            .push_str(&format!("diff --git a/{}\n", file.path));
        current_chunk.content.push_str(&file.content);
        current_chunk.files.push(file.path.clone());
        current_chunk.token_count += header_overhead + file.token_count;
    }

    // Add the last chunk
    if !current_chunk.content.is_empty() {
        chunks.push(current_chunk);
    }

    chunks
}

/// Splits a file diff by git hunks (--- a/... / +++ b/ ... @@ ... @@).
///
/// This is the second level of chunking - if a single file is too large,
/// we split it by individual hunks.
fn split_file_by_hunks(content: &str, max_tokens: usize) -> Vec<String> {
    let mut hunks = Vec::new();
    let mut current_hunk = String::new();
    let mut current_tokens = 0;

    // Track hunk boundaries
    let hunk_header_pattern = regex::Regex::new(r"^@@ -\d+,\d+ \+\d+,\d+ @@").unwrap();

    for line in content.lines() {
        let line_tokens = estimate_tokens(line).unwrap_or(1) + 1; // +1 for newline

        // Check if this is the start of a new hunk
        if hunk_header_pattern.is_match(line) {
            if !current_hunk.is_empty() {
                hunks.push(current_hunk);
                current_hunk = String::new();
                current_tokens = 0;
            }
        }

        if current_tokens + line_tokens > max_tokens && !current_hunk.is_empty() {
            hunks.push(current_hunk);
            current_hunk = String::new();
            current_tokens = 0;
        }

        current_hunk.push_str(line);
        current_hunk.push('\n');
        current_tokens += line_tokens;
    }

    if !current_hunk.is_empty() {
        hunks.push(current_hunk);
    }

    // If we never entered a hunk, return the original content
    if hunks.is_empty() && !content.is_empty() {
        hunks.push(content.to_string());
    }

    hunks
}

/// Performs multi-level diff chunking for large diffs.
///
/// This function implements a three-tier approach to chunking:
/// 1. **File-level merging**: Greedily combine entire file diffs until token limit
/// 2. **Hunk-level splitting**: If a single file is too large, split by hunks
/// 3. **Line-level splitting**: For extremely large hunks, split by lines
///
/// # Arguments
///
/// * `diff` - The full git diff string
/// * `max_tokens` - Maximum tokens allowed per chunk (prompt overhead will be subtracted)
///
/// # Returns
///
/// A single string containing the chunked diff, with separators between chunks
pub fn chunk_diff(diff: &str, max_tokens: usize) -> String {
    // Early return for small diffs
    let total_tokens = match estimate_tokens(diff) {
        Ok(t) => t,
        Err(_) => return diff.to_string(),
    };

    if total_tokens <= max_tokens {
        return diff.to_string();
    }

    // Parse diff into individual files
    let files = parse_diff_into_files(diff);

    // First try: merge whole files
    let file_chunks = merge_diffs_into_chunks(&files, max_tokens);

    if file_chunks.len() == 1 {
        // Single file but still too large - need to split by hunks
        let chunk = &file_chunks[0];
        if chunk.token_count > max_tokens {
            let hunk_chunks = split_file_by_hunks(&chunk.content, max_tokens);
            let total_hunks = hunk_chunks.len();
            if total_hunks > 1 {
                return hunk_chunks
                    .into_iter()
                    .enumerate()
                    .map(|(i, hunk)| format!("---CHUNK {} OF {}---\n{}\n---END CHUNK---", i + 1, total_hunks, hunk.trim()))
                    .collect::<Vec<_>>()
                    .join("\n\n");
            }
        }
        return chunk.content.clone();
    }

    // Multiple file chunks - join with separator
    file_chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| {
            let file_list = chunk.files.join(", ");
            format!("---CHUNK {} OF MULTIPLE FILES---\nFiles: {}\n\n{}\n---END CHUNK---", i + 1, file_list, chunk.content.trim())
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

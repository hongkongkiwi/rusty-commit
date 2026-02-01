//! Utilities for chunking large git diffs into token-safe pieces.
//!
//! This module provides multi-level diff chunking to handle large diffs that
//! exceed model token limits:
//! 1. **File-level merging**: Greedily combine entire file diffs until token limit
//! 2. **Hunk-level splitting**: If a single file is too large, split by hunks
//! 3. **Line-level splitting**: For extremely large hunks, split by lines

use regex::Regex;

use crate::utils::token::estimate_tokens;

/// Represents a single file diff with its metadata
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// The file path (e.g., "src/main.rs")
    pub path: String,
    /// The raw diff content for this file
    pub content: String,
    /// Estimated token count of the content
    pub token_count: usize,
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
pub fn parse_diff_into_files(diff: &str) -> Vec<FileDiff> {
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

        // Check if this is a deleted file header (+++ /dev/null)
        if line.starts_with("+++ /dev/null") && current_file.is_none() {
            // This is a deleted file, parse the path from --- a/
            let path = extract_deleted_file_path(diff);
            if let Some(p) = path.clone() {
                let tokens = estimate_tokens(&p).unwrap_or_default();
                current_file = Some(FileDiff {
                    path: p,
                    content: String::new(),
                    token_count: tokens,
                });
            }
            continue;
        }

        // Check if this is a deleted file old path (we need to capture path before +++)
        if line.starts_with("--- a/") && current_file.is_none() {
            let path = line.strip_prefix("--- a/").unwrap_or(line).to_string();
            let tokens = estimate_tokens(&path).unwrap_or_default();
            // Create file - will be updated if +++ b/ has different path
            current_file = Some(FileDiff {
                path,
                content: String::new(),
                token_count: tokens,
            });
            continue;
        }

        // Add line to current file if we have one
        if let Some(ref mut file) = current_file {
            file.content.push_str(line);
            file.content.push('\n');
            file.token_count += estimate_tokens(line).unwrap_or(1);
        }
    }

    // Save the last file
    if let Some(file) = current_file {
        files.push(file);
    }

    files
}

/// Extract the path from a deleted file diff (where +++ is /dev/null)
fn extract_deleted_file_path(diff: &str) -> Option<String> {
    for line in diff.lines() {
        if line.starts_with("--- a/") {
            return Some(line.strip_prefix("--- a/").unwrap_or(line).to_string());
        }
    }
    None
}

/// Merges file diffs greedily until reaching the token limit.
///
/// This is the first level of chunking - it groups whole files together
/// to maximize context while staying under the token limit.
fn merge_diffs_into_chunks(files: &[FileDiff], max_tokens: usize) -> Vec<DiffChunk> {
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
            current_chunk.content.push('\n');
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
    let hunk_header_pattern = Regex::new(r"^@@ -\d+,\d+ \+\d+,\d+ @@").unwrap();

    for line in content.lines() {
        let line_tokens = estimate_tokens(line).unwrap_or(1) + 1; // +1 for newline

        // Check if this is the start of a new hunk
        if hunk_header_pattern.is_match(line) && !current_hunk.is_empty() {
            hunks.push(current_hunk);
            current_hunk = String::new();
            current_tokens = 0;
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
                    .map(|(i, hunk)| {
                        format!(
                            "---CHUNK {} OF {}---\n{}\n---END CHUNK---",
                            i + 1,
                            total_hunks,
                            hunk.trim()
                        )
                    })
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
            format!(
                "---CHUNK {} OF MULTIPLE FILES---\nFiles: {}\n\n{}\n---END CHUNK---",
                i + 1,
                file_list,
                chunk.content.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_into_files() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n+use std::io;\n fn main() {\n }\n";
        let files = parse_diff_into_files(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/main.rs");
        assert!(files[0].content.contains("use std::io"));
    }

    #[test]
    fn test_chunk_diff_small() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n+++ b/src/main.rs\n fn main() {}\n";
        let result = chunk_diff(diff, 1000);
        assert_eq!(result, diff);
    }

    #[test]
    fn test_parse_diff_header_only() {
        let diff = "diff --git a/.gitignore b/.gitignore\nnew file mode 100644\n--- /dev/null\n+++ b/.gitignore\n@@ -0,0 +1 @@\n+*.tmp\n";
        let files = parse_diff_into_files(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".gitignore");
    }

    #[test]
    fn test_parse_diff_deleted_file() {
        let diff = "diff --git a/old.txt b/old.txt\ndeleted file mode 100644\n--- a/old.txt\n+++ /dev/null\n@@ -1 +0,0 @@\n-old content\n";
        let files = parse_diff_into_files(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "old.txt");
    }
}

//! Utility modules for rusty-commit.
//!
//! This module organizes various utility functions and helpers used throughout
//! the application. Each submodule focuses on a specific responsibility.

pub mod commit_style;
pub mod diff_chunking;
pub mod hooks;
pub mod retry;
pub mod thinking_strip;
pub mod token;
pub mod version;

// Re-export commonly used functions for convenience
pub use diff_chunking::chunk_diff;
pub use thinking_strip::strip_thinking;

//! Unified output module for Rusty Commit CLI.
//!
//! Provides consistent styling, progress tracking, and error formatting
//! across all commands.

pub mod error;
pub mod prelude;
pub mod progress;
pub mod styling;

pub use prelude::{OutputFormat, OutputLevel};

#[allow(dead_code)]
pub use styling::{Color, Styling, Theme};

/// Trait for formatting output based on the current format.
#[allow(dead_code)]
pub trait OutputFormatter: std::fmt::Display {
    /// Write a section header.
    fn section_header(&mut self, title: &str);
    /// Write a success message.
    fn success(&mut self, message: &str);
    /// Write an info message.
    fn info(&mut self, message: &str);
    /// Write a warning message.
    fn warning(&mut self, message: &str);
    /// Write an error message.
    fn error(&mut self, message: &str);
    /// Write a key-value pair.
    fn key_value(&mut self, key: &str, value: &str);
    /// Write a hint or suggestion.
    fn hint(&mut self, message: &str);
    /// Write timing information.
    fn timing(&mut self, component: &str, duration_ms: u64);
    /// Begin a list or collection.
    fn begin_list(&mut self);
    /// Add item to list.
    fn list_item(&mut self, index: usize, text: &str);
    /// End list.
    fn end_list(&mut self);
    /// Write a divider.
    fn divider(&mut self);
    /// Write a blank line.
    fn blank_line(&mut self);
}

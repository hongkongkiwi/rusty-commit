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

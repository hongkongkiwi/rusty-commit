//! Prelude types for output module.

use clap::ValueEnum;

/// Output format for commands.
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    /// Beautiful colored output (default).
    #[default]
    Pretty,
    /// Machine-readable JSON output.
    Json,
    /// Markdown-formatted output.
    Markdown,
}

/// Represents the verbosity level for output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputLevel {
    /// Quiet mode - minimal output.
    Quiet,
    /// Normal mode - standard output.
    #[default]
    Normal,
    /// Verbose mode - detailed output.
    Verbose,
    /// Debug mode - includes timing and internal details.
    Debug,
}

impl OutputLevel {
    pub fn is_verbose_or_higher(&self) -> bool {
        matches!(self, Self::Verbose | Self::Debug)
    }

    pub fn is_debug(&self) -> bool {
        matches!(self, Self::Debug)
    }
}

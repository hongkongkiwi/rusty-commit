//! Styling constants and helpers for Rusty Commit CLI.
//!
//! Provides a consistent color scheme and styling across all output.

use colored::{Color as ColoredColor, Colorize};

/// Primary color palette for Rusty Commit.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Primary accent color (used for headers and emphasis).
    pub primary: Color,
    /// Secondary accent color (used for subheaders).
    pub secondary: Color,
    /// Success color (green).
    pub success: Color,
    /// Warning color (amber/yellow).
    pub warning: Color,
    /// Error color (red).
    pub error: Color,
    /// Neutral/gray color for borders and dividers.
    pub neutral: Color,
    /// Dimmed text color.
    pub dimmed: Color,
    /// Highlight color for selections.
    pub highlight: Color,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            primary: Color::MutedBlue,
            secondary: Color::Purple,
            success: Color::Green,
            warning: Color::Amber,
            error: Color::Red,
            neutral: Color::Gray,
            dimmed: Color::Gray,
            highlight: Color::Cyan,
        }
    }
}

/// Extended color definitions beyond standard colored::Color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// Standard colors from colored crate.
    Standard(ColoredColor),
    /// Muted blue (primary accent).
    MutedBlue,
    /// Muted purple (secondary accent).
    Purple,
    /// Muted amber (warning - not bright yellow).
    Amber,
    /// Muted red (error - not harsh red).
    Red,
    /// Muted green (success - not harsh green).
    Green,
    /// Gray (neutral).
    Gray,
    /// Bright cyan.
    Cyan,
}

impl Color {
    /// Apply this color to a colored::Colorize string.
    pub fn apply<T: colored::Colorize>(&self, text: T) -> colored::ColoredString {
        match self {
            Color::Standard(c) => text.color(*c),
            Color::MutedBlue => text.cyan(),
            Color::Purple => text.purple(),
            Color::Amber => text.yellow(),
            Color::Red => text.red(),
            Color::Green => text.green(),
            Color::Gray => text.dimmed(),
            Color::Cyan => text.cyan(),
        }
    }

    /// Get the underlying colored::Color.
    pub fn to_colored(self) -> ColoredColor {
        match self {
            Color::Standard(c) => c,
            Color::MutedBlue => ColoredColor::Cyan,
            Color::Purple => ColoredColor::Magenta,
            Color::Amber => ColoredColor::Yellow,
            Color::Red => ColoredColor::Red,
            Color::Green => ColoredColor::Green,
            Color::Gray => ColoredColor::White,
            Color::Cyan => ColoredColor::Cyan,
        }
    }
}

/// Output theme configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct Theme {
    /// Whether to use colors in output.
    pub use_colors: bool,
    /// Whether to use emojis.
    pub use_emoji: bool,
    /// Character to use for dividers.
    pub divider_char: char,
    /// Box drawing characters.
    pub box_chars: BoxStyle,
    /// Color palette.
    pub palette: Palette,
}

impl Theme {
    /// Create a new theme with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a theme for minimal output (no colors, no emoji).
    pub fn minimal() -> Self {
        Self {
            use_colors: false,
            use_emoji: false,
            divider_char: '-',
            box_chars: BoxStyle::Ascii,
            palette: Palette::default(),
        }
    }

    /// Create a theme for JSON output.
    pub fn json() -> Self {
        Self {
            use_colors: false,
            use_emoji: false,
            divider_char: '-',
            box_chars: BoxStyle::None,
            palette: Palette::default(),
        }
    }

    /// Create a theme for markdown output.
    pub fn markdown() -> Self {
        Self {
            use_colors: false,
            use_emoji: true,
            divider_char: '-',
            box_chars: BoxStyle::None,
            palette: Palette::default(),
        }
    }
}

/// Box drawing style for panels and sections.
#[derive(Debug, Clone, Copy, Default)]
pub enum BoxStyle {
    /// No box drawing characters.
    #[default]
    None,
    /// ASCII characters only.
    Ascii,
    /// Unicode box drawing characters (rounded corners).
    Unicode,
    /// Unicode box drawing characters (sharp corners).
    UnicodeSharp,
}

impl BoxStyle {
    /// Get the corner characters for this box style.
    pub fn corners(&self) -> (char, char, char, char) {
        match self {
            BoxStyle::None => (' ', ' ', ' ', ' '),
            BoxStyle::Ascii => ('+', '+', '+', '+'),
            BoxStyle::Unicode => ('╭', '╮', '╰', '╯'),
            BoxStyle::UnicodeSharp => ('┌', '┐', '└', '┘'),
        }
    }

    /// Get the horizontal line character.
    pub fn horizontal(&self) -> char {
        match self {
            BoxStyle::None => ' ',
            BoxStyle::Ascii => '-',
            BoxStyle::Unicode | BoxStyle::UnicodeSharp => '─',
        }
    }

    /// Get the vertical line character.
    pub fn vertical(&self) -> char {
        match self {
            BoxStyle::None | BoxStyle::Ascii => '|',
            BoxStyle::Unicode | BoxStyle::UnicodeSharp => '│',
        }
    }
}

/// Styling utilities and helpers.
#[derive(Debug, Clone, Default)]
pub struct Styling;

#[allow(dead_code)]
impl Styling {
    /// Get the styled header format.
    pub fn header(text: &str) -> String {
        format!("{}", text.bold())
    }

    /// Get the styled subheader format.
    pub fn subheader(text: &str) -> String {
        format!("{}", text.dimmed())
    }

    /// Get the styled success format.
    pub fn success(text: &str) -> String {
        format!("{}", text.green())
    }

    /// Get the styled warning format.
    pub fn warning(text: &str) -> String {
        format!("{}", text.yellow())
    }

    /// Get the styled error format.
    pub fn error(text: &str) -> String {
        format!("{}", text.red())
    }

    /// Get the styled info format.
    pub fn info(text: &str) -> String {
        format!("{}", text.cyan())
    }

    /// Get the styled hint format.
    pub fn hint(text: &str) -> String {
        format!("{}", text.dimmed())
    }

    /// Create a divider line of specified length.
    pub fn divider(length: usize) -> String {
        "─".repeat(length)
    }

    /// Create a section box with title.
    pub fn section_box(title: &str, content: &str, theme: &Theme) -> String {
        let width = 60;
        let horizontal = theme.box_chars.horizontal();
        let (tl, tr, bl, br) = theme.box_chars.corners();

        let mut result = String::new();

        // Top border
        result.push(tl);
        result.push_str(&format!("{} ", title).bold().to_string());
        for _ in title.len() + 2..width - 1 {
            result.push(horizontal);
        }
        result.push(tr);
        result.push('\n');

        // Content
        for line in content.lines() {
            result.push(theme.box_chars.vertical());
            result.push(' ');
            result.push_str(line);
            // Pad to width
            for _ in line.len() + 1..width - 1 {
                result.push(' ');
            }
            result.push(theme.box_chars.vertical());
            result.push('\n');
        }

        // Bottom border
        result.push(bl);
        for _ in 0..width - 1 {
            result.push(horizontal);
        }
        result.push(br);

        result
    }

    /// Format a key-value pair.
    pub fn key_value(key: &str, value: &str) -> String {
        format!("{}: {}", key.dimmed(), value)
    }

    /// Format a timing entry.
    pub fn timing(component: &str, duration_ms: u64) -> String {
        let duration = if duration_ms < 1000 {
            format!("{}ms", duration_ms)
        } else {
            format!("{:.1}s", duration_ms as f64 / 1000.0)
        };
        format!("{} {}", component.dimmed(), duration.green())
    }

    /// Print a section with header, divider, content, and closing divider.
    pub fn print_section(title: &str, content: &str) {
        let divider = Self::divider(50);
        println!("\n{}", title.cyan().bold());
        println!("{}", divider.dimmed());
        println!("{}", content);
        println!("{}", divider.dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divider_length() {
        let d = Styling::divider(10);
        // Use chars().count() for unicode-aware length
        assert_eq!(d.chars().count(), 10);
        assert!(d.chars().all(|c| c == '─'));
    }

    #[test]
    fn test_key_value_format() {
        let kv = Styling::key_value("Key", "Value");
        assert!(kv.contains("Key:"));
        assert!(kv.contains("Value"));
    }

    #[test]
    fn test_timing_ms() {
        let t = Styling::timing("test", 500);
        assert!(t.contains("500ms"));
    }

    #[test]
    fn test_timing_seconds() {
        let t = Styling::timing("test", 2500);
        assert!(t.contains("2.5s"));
    }

    #[test]
    fn test_theme_has_colors_option() {
        let theme = Theme::new();
        // Theme should have a use_colors field (may be true or false depending on tty)
        let _ = theme.use_colors;
    }

    #[test]
    fn test_palette_colors() {
        let palette = Palette::default();
        // Verify palette has valid colors
        match palette.primary {
            Color::MutedBlue => {}
            Color::Standard(_) => {}
            _ => panic!("Expected MutedBlue or Standard color"),
        }
    }
}

/// Emoji constants for consistent usage.
#[allow(dead_code)]
pub mod emoji {
    use once_cell::sync::Lazy;

    /// Check mark for success.
    pub static CHECK: Lazy<&'static str> = Lazy::new(|| "✓");
    /// Cross mark for errors.
    pub static CROSS: Lazy<&'static str> = Lazy::new(|| "✗");
    /// Light bulb for hints.
    pub static HINT: Lazy<&'static str> = Lazy::new(|| "💡");
    /// Key for authentication.
    pub static KEY: Lazy<&'static str> = Lazy::new(|| "🔐");
}

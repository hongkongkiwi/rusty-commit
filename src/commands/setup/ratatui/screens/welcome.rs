//! Welcome screen for TUI setup
//!
//! This module renders the welcome screen that introduces
/// the user to the Rusty Commit setup process.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use super::SetupApp;

/// Render the welcome screen
///
/// Shows the app title, welcome message, and instructions
/// for continuing to the setup process.
pub fn render_welcome_screen(frame: &mut Frame, area: Rect, _app: &mut SetupApp) {
    // Title section with emoji and styled text
    let title = Line::from(vec![
        "🚀 ".into(),
        "Rusty Commit".bold().fg(Color::LightCyan),
        " Setup".bold(),
    ])
    .alignment(Alignment::Center);

    // Subtitle with description
    let subtitle = Line::from(vec![
        "Let's get you set up with AI-powered commit messages".dim().into(),
    ])
    .alignment(Alignment::Center);

    // Spacer
    let spacer = Line::from(vec!["".into()]);

    // Instructions
    let instructions = Line::from(vec![
        "Press ".into(),
        "[Enter]".bold().fg(Color::LightGreen),
        " to continue".into(),
    ])
    .alignment(Alignment::Center);

    // Build the content
    let content = Paragraph::new(vec![title, Text::new(""), subtitle, spacer, instructions])
        .block(
            Block::bordered()
                .title("Welcome")
                .border_style(Style::default().fg(Color::LightCyan)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

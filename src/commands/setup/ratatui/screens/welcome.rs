//! Welcome screen for TUI setup
//!
//! This module renders the welcome screen that introduces
/// the user to the Rusty Commit setup process.

use ratatui::{
    layout::{Alignment, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Render the welcome screen
///
/// Shows the app title, welcome message, and instructions
/// for continuing to the setup process.
pub fn render_welcome_screen(frame: &mut Frame, area: Rect, _app: &mut SetupApp) {
    // Title section with emoji and styled text
    let title = Line::from(vec![
        Span::from("🚀 "),
        Span::styled("Rusty Commit", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
        Span::styled(" Setup", Style::default().add_modifier(Modifier::BOLD)),
    ])
    .alignment(Alignment::Center);

    // Subtitle with description
    let subtitle = Line::from(vec![Span::styled(
        "Let's get you set up with AI-powered commit messages",
        Style::default().add_modifier(Modifier::DIM),
    )])
    .alignment(Alignment::Center);

    // Instructions
    let instructions = Line::from(vec![
        Span::from("Press "),
        Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD).fg(Color::LightGreen)),
        Span::from(" to continue"),
    ])
    .alignment(Alignment::Center);

    // Build the content
    let content = Paragraph::new(vec![title, Line::from(""), subtitle, Line::from(""), instructions])
        .block(
            Block::bordered()
                .title("Welcome")
                .border_style(Style::default().fg(Color::LightCyan)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}

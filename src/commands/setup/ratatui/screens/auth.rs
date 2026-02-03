//! Authentication screen for TUI setup
//!
//! This module renders the authentication screen where users
/// can enter their API key for the selected provider.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Render the authentication screen
///
/// Shows the selected provider and allows the user to
/// enter their API key.
pub fn render_auth_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let provider = app
        .config()
        .provider
        .as_ref()
        .expect("Provider should be set before auth screen");

    let title = format!("Authentication - {}", provider.name);
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title_widget, chunks[0]);

    // API key input (masked)
    let masked_key = if app.config().api_key.as_ref().map(|k| k.len()).unwrap_or(0) > 0 {
        "*".repeat(20)
    } else {
        "Not entered".to_string()
    };

    let auth_info = format!(
        r#"Provider: {}

Requires API Key: {}

Current API key: {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
API key entry will be implemented in a future update.
For now, use: rco auth login
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"#,
        provider.name,
        if provider.requires_key {
            "Yes - API key required"
        } else {
            "No (local provider)"
        },
        masked_key
    );

    let auth_widget = Paragraph::new(auth_info)
        .block(
            Block::bordered()
                .title("Authentication")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        );

    frame.render_widget(auth_widget, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter skip · Esc back")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

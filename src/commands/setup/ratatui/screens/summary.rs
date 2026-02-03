//! Summary screen for TUI setup
//!
//! This module renders the summary screen that shows all
/// configuration choices before saving.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Render the summary screen
///
/// Shows a summary of all configuration choices and
/// provides save/cancel options.
pub fn render_summary_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Configuration Summary")
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title, chunks[0]);

    // Summary content
    let provider_info = app
        .config()
        .provider
        .as_ref()
        .map(|p| format!("Provider: {} (model: {})", p.name, app.config().model))
        .unwrap_or_else(|| "Provider: Not selected".to_string());

    let style_info = format!(
        "Style: {:?} (emoji: {})",
        app.config().commit_style,
        if app.config().emoji { "Yes" } else { "No" }
    );

    let language_info = format!("Language: {}", app.config().language);

    let emoji_info = format!("Use Emojis: {}", if app.config().emoji { "Yes" } else { "No" });

    let capitalize_info = format!(
        "Capitalize: {}",
        if app.config().description_capitalize { "Yes" } else { "No" }
    );

    let period_info = format!(
        "Add Period: {}",
        if app.config().description_add_period { "Yes" } else { "No" }
    );

    let max_length_info = format!("Max Length: {} chars", app.config().description_max_length);

    let generate_count_info = format!("Generate Count: {}", app.config().generate_count);

    let summary = Paragraph::new(vec![
        Line::from(provider_info),
        Line::from(style_info),
        Line::from(emoji_info),
        Line::from(language_info),
        Line::from(capitalize_info),
        Line::from(period_info),
        Line::from(max_length_info),
        Line::from(generate_count_info),
        Line::from(""),
        Line::from(vec![
            "Press ".into(),
            "[Enter]".bold().fg(Color::LightGreen),
            " to save configuration".into(),
        ]),
        Line::from(vec![
            "Press ".into(),
            "[Esc]".bold().fg(Color::LightRed),
            " to go back".into(),
        ]),
    ])
    .block(
        Block::bordered()
            .title("Summary")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightCyan)),
    );

    frame.render_widget(summary, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter save · Esc go back")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

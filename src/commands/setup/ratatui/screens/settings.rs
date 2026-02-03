//! Settings screen for TUI setup
//!
//! This module renders the behavior settings screen where users
/// can configure various options for commit generation.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Render the settings screen
///
/// Shows configurable behavior settings for the user
/// to review and optionally modify.
pub fn render_settings_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Behavior Settings")
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title, chunks[0]);

    // Settings list
    let settings_items = vec![
        format!(
            "Capitalize first letter: {}",
            if app.config().description_capitalize {
                "Yes"
            } else {
                "No"
            }
        ),
        format!(
            "Add period at end: {}",
            if app.config().description_add_period {
                "Yes"
            } else {
                "No"
            }
        ),
        format!(
            "Max length: {} chars",
            app.config().description_max_length
        ),
        format!("Generate count: {}", app.config().generate_count),
        format!(
            "Use emojis: {}",
            if app.config().emoji { "Yes" } else { "No" }
        ),
        format!(
            "Auto-push: {}",
            if app.config().gitpush { "Yes" } else { "No" }
        ),
        format!(
            "Multi-line commits: {}",
            if app.config().enable_commit_body {
                "Yes"
            } else {
                "No"
            }
        ),
        format!(
            "Learn from history: {}",
            if app.config().learn_from_history {
                "Yes"
            } else {
                "No"
            }
        ),
        format!(
            "History commits: {}",
            app.config().history_commits_count
        ),
        format!(
            "Clipboard on timeout: {}",
            if app.config().clipboard_on_timeout {
                "Yes"
            } else {
                "No"
            }
        ),
        format!(
            "Hook strict mode: {}",
            if app.config().hook_strict { "Yes" } else { "No" }
        ),
        format!(
            "Hook timeout: {}ms",
            app.config().hook_timeout_ms
        ),
        format!(
            "Max input tokens: {}",
            app.config().tokens_max_input
        ),
        format!(
            "Max output tokens: {}",
            app.config().tokens_max_output
        ),
        format!("Language: {}", app.config().language),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let marker = if idx == app.menu_index() { ">" } else { " " };
            ListItem::new(format!("{} {}", marker, s))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title("Settings")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black),
        );

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑/↓ navigate · Space toggle · Enter next")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

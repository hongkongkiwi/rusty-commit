//! Style selection screen for TUI setup
//!
//! This module renders the commit style selection screen where users
/// can choose their preferred commit message format.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;
use crate::config::setup_config::CommitFormat;

/// Render the style selection screen
///
/// Shows the available commit message formats for the user
/// to choose from.
pub fn render_style_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Select Commit Message Format")
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title, chunks[0]);

    // Format options
    let formats = CommitFormat::all();
    let items: Vec<ListItem> = formats
        .iter()
        .map(|f| {
            let marker = if *f == app.config().commit_style {
                "●"
            } else {
                "○"
            };
            ListItem::new(Line::from(vec![
                marker.into(),
                " ".into(),
                f.display().into(),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title("Format")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black),
        );

    frame.render_widget(list, chunks[1]);

    // Example based on current selection
    let example = match app.config().commit_style {
        CommitFormat::Conventional => "feat(auth): Add login functionality",
        CommitFormat::Gitmoji => "✨ feat(auth): Add login functionality",
        CommitFormat::Simple => "Add login functionality",
    };

    let example_label = Paragraph::new(format!("Example: {}", example))
        .style(Style::default().dim());
    frame.render_widget(example_label, chunks[2]);
}

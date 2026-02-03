//! Model selection screen for TUI setup
//!
//! This module renders the model selection screen where users
/// can select or enter a model for their chosen AI provider.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Render the model selection screen
///
/// Shows the selected provider and allows the user to
/// select or enter a custom model.
pub fn render_model_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title with provider info
    let provider_name = app
        .config()
        .provider
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let title = format!("Model Selection - {}", provider_name);
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title_widget, chunks[0]);

    // Model selection area
    let default_model = app
        .config()
        .provider
        .as_ref()
        .map(|p| p.default_model.clone())
        .unwrap_or_else(|| "unknown".to_string());

    let current_model = if app.config().model.is_empty() {
        default_model.clone()
    } else {
        app.config().model.clone()
    };

    let model_content = format!(
        r#"Provider: {}

Default model: {}

Your selection: {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Press Enter to use default, or type a custom model
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"#,
        provider_name,
        default_model,
        current_model
    );

    let model_info = Paragraph::new(model_content)
        .block(
            Block::bordered()
                .title("Model")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan)),
        );

    frame.render_widget(model_info, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter to use default · Esc back")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

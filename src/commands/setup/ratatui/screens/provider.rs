//! Provider selection screen for TUI setup
//!
//! This module renders the provider selection screen that allows
/// users to choose an AI provider from a categorized list.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;
use crate::config::setup_config::{ProviderCategory, ProviderOption};

/// Get all providers grouped by category
fn get_providers_by_category() -> Vec<(ProviderCategory, Vec<ProviderOption>)> {
    let all_providers = ProviderOption::all();

    // Group providers by category
    let mut grouped: Vec<(ProviderCategory, Vec<ProviderOption>)> = Vec::new();

    for provider in all_providers {
        if let Some((_, group)) = grouped.iter_mut().find(|(cat, _)| *cat == provider.category) {
            group.push(provider);
        } else {
            grouped.push((provider.category, vec![provider]));
        }
    }

    grouped
}

/// Render the provider selection screen
///
/// Shows a categorized list of AI providers that the user
/// can navigate and select from.
pub fn render_provider_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Select your AI Provider")
        .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    // Provider list grouped by category
    let providers_by_category = get_providers_by_category();

    let items: Vec<ListItem> = providers_by_category
        .iter()
        .flat_map(|(category, providers)| {
            // Category header
            let header = ListItem::new(
                Line::from(vec![Span::styled(
                    format!("─── {} ───", category.display()),
                    Style::default().add_modifier(Modifier::DIM)
                )])
            ).style(Style::default().fg(Color::DarkGray));

            // Provider items
            providers.iter().map(|p| {
                ListItem::new(
                    Line::from(vec![p.display.clone().into()])
                )
                .style(
                    Style::default()
                        .fg(Color::White)
                )
            })
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title("Providers").border_style(Style::default().fg(Color::LightCyan)))
        .highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
        );

    frame.render_widget(list, chunks[1]);

    // Footer with navigation hints
    let footer = Paragraph::new("↑/↓ navigate · Enter select · Esc back")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

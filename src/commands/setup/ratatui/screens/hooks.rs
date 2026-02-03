//! Hooks screen for TUI setup
//!
//! This module renders the hooks screen where users
/// can install/uninstall git hooks.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::ratatui::app::SetupApp;

/// Hook installation options
enum HookOption {
    InstallPrepareCommitMsg,
    InstallCommitMsg,
    UninstallHooks,
    HookStrict,
    HookTimeout,
}

impl HookOption {
    fn all() -> Vec<Self> {
        vec![
            HookOption::InstallPrepareCommitMsg,
            HookOption::InstallCommitMsg,
            HookOption::UninstallHooks,
            HookOption::HookStrict,
            HookOption::HookTimeout,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            HookOption::InstallPrepareCommitMsg => "Install prepare-commit-msg hook",
            HookOption::InstallCommitMsg => "Install commit-msg hook",
            HookOption::UninstallHooks => "Uninstall all hooks",
            HookOption::HookStrict => "Hook strict mode",
            HookOption::HookTimeout => "Hook timeout",
        }
    }
}

/// Render the hooks screen
///
/// Shows current git hook status and allows
/// enabling/disabling them.
pub fn render_hooks_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Git Hooks Configuration")
        .style(Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD));
    frame.render_widget(title, chunks[0]);

    // Hook options
    let hook_options = HookOption::all();

    let items: Vec<ListItem> = hook_options
        .iter()
        .enumerate()
        .map(|(idx, option)| {
            let marker = if idx == app.menu_index() { ">" } else { " " };
            let status = match option {
                HookOption::InstallPrepareCommitMsg => {
                    let installed = check_hook_exists("prepare-commit-msg");
                    if installed {
                        "[INSTALLED]".green()
                    } else {
                        "[NOT INSTALLED]".red()
                    }
                }
                HookOption::InstallCommitMsg => {
                    let installed = check_hook_exists("commit-msg");
                    if installed {
                        "[INSTALLED]".green()
                    } else {
                        "[NOT INSTALLED]".red()
                    }
                }
                HookOption::UninstallHooks => {
                    "[ACTION]".yellow()
                }
                HookOption::HookStrict => {
                    if app.config().hook_strict { "[ON]".green() } else { "[OFF]".red() }
                }
                HookOption::HookTimeout => {
                    "[CONFIG]".cyan()
                }
            };
            ListItem::new(format!("{} {} {}", marker, option.label(), status))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title("Hooks")
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
    let footer = Paragraph::new("↑/↓ navigate · Space toggle install · Esc back")
        .style(Style::default().dim());
    frame.render_widget(footer, chunks[2]);
}

/// Check if a git hook file exists and contains rco
fn check_hook_exists(hook_name: &str) -> bool {
    use std::fs;

    if let Ok(repo_root) = std::env::current_dir() {
        let hook_path = repo_root.join(".git").join("hooks").join(hook_name);
        if hook_path.exists() {
            if let Ok(content) = fs::read_to_string(&hook_path) {
                return content.contains("rco");
            }
        }
    }
    false
}

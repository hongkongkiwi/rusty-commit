//! Main TUI runner with terminal setup
//!
//! This module provides the main entry point for the TUI setup
//! including terminal initialization, event loop, and screen rendering.

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::{CrosstermBackend, Rect};
use ratatui::Terminal;

use super::app::SetupApp;
use super::event::{Event, EventHandler};
use super::screens;

/// Result type for TUI operations
pub type TuiResult = Result<(), Box<dyn std::error::Error>>;

/// TUI terminal wrapper
///
/// Handles terminal initialization, raw mode, and cleanup.
pub struct Tui {
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Event handler
    events: EventHandler,
}

impl Tui {
    /// Create a new TUI instance
    pub fn new() -> io::Result<Self> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(Duration::from_millis(50));

        Ok(Self {
            terminal,
            events,
        })
    }

    /// Enter alternate screen and enable raw mode
    pub fn enter_alternate_screen(&mut self) -> TuiResult {
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    /// Leave alternate screen and disable raw mode
    pub fn leave_alternate_screen(&mut self) -> TuiResult {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    /// Render the current screen
    pub fn draw(&mut self, app: &mut SetupApp) -> TuiResult {
        self.terminal.draw(|frame| {
            let area = frame.area();
            self.render_screen(frame, area, app);
        })?;
        Ok(())
    }

    /// Render the appropriate screen based on app state
    fn render_screen(&self, frame: &mut ratatui::Frame, area: Rect, app: &mut SetupApp) {
        use super::ScreenType::*;
        match app.current_screen() {
            Welcome => screens::welcome::render_welcome_screen(frame, area, app),
            Provider => screens::provider::render_provider_screen(frame, area, app),
            Model => screens::model::render_model_screen(frame, area, app),
            Auth => screens::auth::render_auth_screen(frame, area, app),
            Style => screens::style::render_style_screen(frame, area, app),
            Settings => screens::settings::render_settings_screen(frame, area, app),
            Summary => screens::summary::render_summary_screen(frame, area, app),
        }
    }

    /// Handle keyboard events
    ///
    /// Returns true if the application should continue running,
    /// false if it should exit.
    pub fn handle_events(&mut self, app: &mut SetupApp) -> bool {
        match self.events.next() {
            Ok(Event::Key(key)) => {
                self.handle_key(key, app);
                true
            }
            Ok(Event::Tick) => false,
            Err(_) => false,
        }
    }

    /// Handle a key press event
    fn handle_key(&mut self, key: KeyEvent, app: &mut SetupApp) {
        use KeyCode::*;

        match key.code {
            // Navigation
            Down | j | J => {
                app.increment_menu_index(10);
            }
            Up | k | K => {
                app.decrement_menu_index();
            }
            Enter => {
                // Advance to next screen on Enter
                if !app.is_last_screen() {
                    app.next_screen();
                }
            }
            Esc | q | Q => {
                // Go back or exit
                if app.is_first_screen() {
                    // Exit on first screen
                    std::process::exit(0);
                } else {
                    app.previous_screen();
                }
            }
            // Toggle settings (Space)
            Tab => {
                // Could be used for toggling between options
            }
            _ => {}
        }
    }
}

/// Drop implementation to ensure terminal cleanup
impl Drop for Tui {
    fn drop(&mut self) {
        // Best effort cleanup - ignore errors
        let _ = self.leave_alternate_screen();
    }
}

/// Main entry point for the TUI setup
///
/// This function initializes the terminal, runs the event loop,
/// and handles screen transitions until the user completes or
/// cancels the setup.
pub async fn tui_main() -> TuiResult {
    let mut tui = Tui::new()?;
    let mut app = SetupApp::new();

    // Enter alternate screen
    tui.enter_alternate_screen()?;

    // Main event loop
    loop {
        // Draw current screen
        tui.draw(&mut app)?;

        // Handle events
        if !tui.handle_events(&mut app) {
            continue;
        }

        // Check if we're on the summary screen and should exit
        if matches!(app.current_screen(), super::ScreenType::Summary) {
            // Exit the TUI
            break;
        }
    }

    // Cleanup
    tui.leave_alternate_screen()?;

    Ok(())
}

//! Main application state for TUI setup
//!
//! This module defines the SetupApp struct that manages the
/// screen state and accumulated configuration during the TUI flow.

use crate::config::setup_config::SetupConfig;

/// Screen states for the TUI setup flow
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenType {
    /// Welcome screen
    Welcome,
    /// Provider selection screen
    Provider,
    /// Model selection/input screen
    Model,
    /// API key authentication screen
    Auth,
    /// Commit style selection screen
    Style,
    /// Git hooks configuration screen
    Hooks,
    /// Behavior settings screen
    Settings,
    /// Summary and save confirmation screen
    Summary,
}

/// Main application state for the TUI setup
///
/// This struct holds the current screen, navigation state,
/// and accumulated configuration during the setup process.
#[derive(Debug)]
pub struct SetupApp {
    /// Current screen being displayed
    current_screen: ScreenType,
    /// Previous screen for back navigation
    previous_screen: Option<ScreenType>,
    /// Configuration being built
    config: SetupConfig,
    /// Current menu/index position
    menu_index: usize,
    /// Scroll offset for long lists
    scroll_offset: usize,
}

impl SetupApp {
    /// Create a new application in the default starting state
    pub fn new() -> Self {
        Self {
            current_screen: ScreenType::Welcome,
            previous_screen: None,
            config: SetupConfig::default(),
            menu_index: 0,
            scroll_offset: 0,
        }
    }

    /// Get the current screen type
    pub fn current_screen(&self) -> ScreenType {
        self.current_screen.clone()
    }

    /// Navigate to the next screen in the flow
    pub fn next_screen(&mut self) {
        self.previous_screen = Some(self.current_screen.clone());
        self.current_screen = match self.current_screen {
            ScreenType::Welcome => ScreenType::Provider,
            ScreenType::Provider => ScreenType::Model,
            ScreenType::Model => ScreenType::Auth,
            ScreenType::Auth => ScreenType::Style,
            ScreenType::Style => ScreenType::Hooks,
            ScreenType::Hooks => ScreenType::Settings,
            ScreenType::Settings => ScreenType::Summary,
            ScreenType::Summary => ScreenType::Summary,
        };
        // Reset menu index when changing screens
        self.menu_index = 0;
        self.scroll_offset = 0;
    }

    /// Navigate back to the previous screen
    pub fn previous_screen(&mut self) {
        if let Some(prev) = self.previous_screen.clone() {
            self.current_screen = prev;
            self.previous_screen = None;
            self.menu_index = 0;
            self.scroll_offset = 0;
        }
    }

    /// Increment the menu index if not at the maximum
    pub fn increment_menu_index(&mut self, max: usize) {
        if self.menu_index < max.saturating_sub(1) {
            self.menu_index += 1;
        }
    }

    /// Decrement the menu index if not at zero
    pub fn decrement_menu_index(&mut self) {
        if self.menu_index > 0 {
            self.menu_index -= 1;
        }
    }

    /// Get the current menu index
    pub fn menu_index(&self) -> usize {
        self.menu_index
    }

    /// Set the menu index
    pub fn set_menu_index(&mut self, index: usize) {
        self.menu_index = index;
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set the scroll offset
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset;
    }

    /// Get immutable reference to the configuration
    pub fn config(&self) -> &SetupConfig {
        &self.config
    }

    /// Get mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut SetupConfig {
        &mut self.config
    }

    /// Check if we're on the first screen
    pub fn is_first_screen(&self) -> bool {
        matches!(self.current_screen, ScreenType::Welcome)
    }

    /// Check if we're on the last screen
    pub fn is_last_screen(&self) -> bool {
        matches!(self.current_screen, ScreenType::Summary)
    }
}

impl Default for SetupApp {
    fn default() -> Self {
        Self::new()
    }
}

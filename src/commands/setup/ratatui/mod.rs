//! TUI setup module
//!
//! This module provides the interactive terminal user interface for
//! the `rco setup` command using Ratatui.

mod app;
mod event;
mod runner;
mod screens;

pub use app::{SetupApp, ScreenType};
pub use event::{Event, EventHandler};
pub use runner::{tui_main, TuiResult};

//! Event handling for TUI
//!
//! This module provides event polling and handling for keyboard
//! input in the terminal UI.

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Events that can be handled by the TUI
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard input event
    Key(KeyEvent),
    /// Tick event for periodic updates
    Tick,
}

/// Event handler that polls for keyboard events
///
/// Events are received through a channel and can be processed
/// by the main application loop.
pub struct EventHandler {
    /// Receiver for events
    receiver: mpsc::Receiver<Event>,
    /// Thread handle for cleanup
    _thread: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create a new event handler with the given tick rate
    ///
    /// The tick rate determines how often the handler checks for
    /// events when no keyboard input is available.
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();

        let thread = thread::spawn(move || {
            loop {
                // Check if there's a key event ready
                if event::poll(tick_rate).expect("event poll failed") {
                    if let CrosstermEvent::Key(key) = event::read().expect("event read failed") {
                        // Only handle key press events (not releases)
                        if key.kind == KeyEventKind::Press {
                            if sender.send(Event::Key(key)).is_err() {
                                // Receiver dropped, exit thread
                                break;
                            }
                        }
                    }
                }
            }
        });

        Self {
            receiver,
            _thread: thread,
        }
    }

    /// Wait for and receive the next event
    ///
    /// This blocks until an event is received or the channel
    /// is closed.
    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}

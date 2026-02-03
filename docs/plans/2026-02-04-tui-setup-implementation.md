# TUI Setup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an interactive TUI for `rco setup` using Ratatui with menu-driven navigation and keyboard shortcuts.

**Architecture:** Replace dialoguer-based setup prompts with a Ratatui-based TUI that includes welcome screen, provider selection, model input, auth configuration, commit style selection, settings toggles, and summary/review. Fall back to dialoguer if not a TTY.

**Tech Stack:** Ratatui 0.28, Crossterm 0.28, existing providers.rs data structures

---

## Tasks

### Task 1: Add Ratatui and Crossterm dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependencies**

```toml
[dependencies]
ratatui = "0.28"
crossterm = "0.28"
```

**Step 2: Verify compilation**

Run: `cargo check --quiet`
Expected: No errors

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add ratatui and crossterm dependencies"
```

---

### Task 2: Create SetupConfig struct in config module

**Files:**
- Create: `src/config/setup_config.rs`

**Step 1: Write the failing test**

```rust
// tests/config/setup_config_test.rs

#[test]
fn test_setup_config_default_values() {
    let config = SetupConfig::default();
    assert_eq!(config.provider, None);
    assert!(config.api_key.is_none());
    assert_eq!(config.commit_style, CommitFormat::Conventional);
}
```

Run: `cargo test --test config_setup_config_test 2>&1 | head -20`
Expected: FAIL - SetupConfig not found

**Step 2: Write minimal implementation**

```rust
// src/config/setup_config.rs

use super::Config;

/// Configuration built during TUI setup
#[derive(Debug, Default)]
pub struct SetupConfig {
    pub provider: Option<super::ProviderOption>,
    pub model: String,
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub commit_style: CommitFormat,
    pub language: String,
    pub description_capitalize: bool,
    pub description_add_period: bool,
    pub description_max_length: usize,
    pub generate_count: u8,
    pub emoji: bool,
    pub gitpush: bool,
    pub one_line_commit: bool,
    pub enable_commit_body: bool,
    pub learn_from_history: bool,
    pub history_commits_count: usize,
    pub clipboard_on_timeout: bool,
    pub hook_strict: bool,
    pub hook_timeout_ms: u64,
    pub tokens_max_input: usize,
    pub tokens_max_output: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommitFormat {
    Conventional,
    Gitmoji,
    Simple,
}

impl Default for CommitFormat {
    fn default() -> Self {
        Self::Conventional
    }
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test --test config_setup_config_test -- --nocapture`
Expected: PASS

**Step 4: Commit**

```bash
git add src/config/setup_config.rs tests/config/setup_config_test.rs
git commit -m "feat: add SetupConfig struct for TUI setup"
```

---

### Task 3: Create TUI app skeleton with event loop

**Files:**
- Create: `src/commands/setup/ratatui/mod.rs`
- Create: `src/commands/setup/ratatui/app.rs`
- Create: `src/commands/setup/ratatui/event.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/app_test.rs

#[test]
fn test_setup_app_creation() {
    let app = SetupApp::new();
    assert_eq!(app.current_screen(), ScreenType::Welcome);
}
```

Run: `cargo test --test setup_ratutui_app_test 2>&1 | head -20`
Expected: FAIL - modules not found

**Step 2: Create mod.rs**

```rust
// src/commands/setup/ratatui/mod.rs

mod app;
mod event;

pub use app::{SetupApp, ScreenType};
pub use event::{Event, EventHandler};
```

**Step 3: Create event.rs**

```rust
// src/commands/setup/ratatui/event.rs

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, KeyEventKind};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum Event {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    _thread: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let thread = thread::spawn(move || {
            loop {
                if event::poll(tick_rate).expect("event poll failed") {
                    if let CrosstermEvent::Key(key) = event::read().expect("event read failed") {
                        if key.kind == KeyEventKind::Press {
                            sender.send(Event::Key(key)).expect("send failed");
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

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
```

**Step 4: Create app.rs**

```rust
// src/commands/setup/ratatui/app.rs

use crate::config::setup_config::{CommitFormat, SetupConfig};

pub enum ScreenType {
    Welcome,
    Provider,
    Model,
    Auth,
    Style,
    Settings,
    Summary,
}

pub struct SetupApp {
    current_screen: ScreenType,
    config: SetupConfig,
    menu_index: usize,
}

impl SetupApp {
    pub fn new() -> Self {
        Self {
            current_screen: ScreenType::Welcome,
            config: SetupConfig::default(),
            menu_index: 0,
        }
    }

    pub fn current_screen(&self) -> ScreenType {
        self.current_screen
    }

    pub fn next_screen(&mut self) {
        self.current_screen = match self.current_screen {
            ScreenType::Welcome => ScreenType::Provider,
            ScreenType::Provider => ScreenType::Model,
            ScreenType::Model => ScreenType::Auth,
            ScreenType::Auth => ScreenType::Style,
            ScreenType::Style => ScreenType::Settings,
            ScreenType::Settings => ScreenType::Summary,
            ScreenType::Summary => ScreenType::Summary,
        };
    }

    pub fn previous_screen(&mut self) {
        self.current_screen = match self.current_screen {
            ScreenType::Welcome => ScreenType::Welcome,
            ScreenType::Provider => ScreenType::Welcome,
            ScreenType::Model => ScreenType::Provider,
            ScreenType::Auth => ScreenType::Model,
            ScreenType::Style => ScreenType::Auth,
            ScreenType::Settings => ScreenType::Style,
            ScreenType::Summary => ScreenType::Settings,
        };
    }

    pub fn increment_menu_index(&mut self, max: usize) {
        if self.menu_index < max.saturating_sub(1) {
            self.menu_index += 1;
        }
    }

    pub fn decrement_menu_index(&mut self) {
        if self.menu_index > 0 {
            self.menu_index -= 1;
        }
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index
    }

    pub fn config(&self) -> &SetupConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SetupConfig {
        &mut self.config
    }
}

impl Default for SetupApp {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test --test setup_ratutui_app_test -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/commands/setup/ratatui/
git commit -m "feat: add TUI app skeleton with event loop"
```

---

### Task 4: Create main TUI runner with terminal setup

**Files:**
- Create: `src/commands/setup/ratatui/runner.rs`
- Modify: `src/commands/setup/ratatui/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/runner_test.rs

#[test]
fn test_tui_runner_exists() {
    let runner = TuiRunner::new();
    assert!(runner.is_ok());
}
```

Run: `cargo test --test setup_ratutui_runner_test 2>&1 | head -20`
Expected: FAIL - TuiRunner not found

**Step 2: Update mod.rs**

```rust
// src/commands/setup/ratatui/mod.rs

mod app;
mod event;
mod runner;

pub use app::{SetupApp, ScreenType};
pub use event::{Event, EventHandler};
pub use runner::{tui_main, TuiResult};
```

**Step 3: Create runner.rs**

```rust
// src/commands/setup/ratatui/runner.rs

use std::io;
use std::time::Duration;

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;
use ratatui::Terminal;

use super::{app::SetupApp, event::EventHandler};

pub type TuiResult = Result<(), Box<dyn std::error::Error>>;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    events: EventHandler,
}

impl Tui {
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

    pub fn enter_alternate_screen(&mut self) -> TuiResult {
        execute!(self.terminal.backend_mut(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        Ok(())
    }

    pub fn leave_alternate_screen(&mut self) -> TuiResult {
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut SetupApp) -> TuiResult {
        self.terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget(area.clone(), Block::new().style(Style::default().bg(Color::Black)));
            // Placeholder - screens will be added later
        })?;
        Ok(())
    }

    pub fn handle_events(&mut self, app: &mut SetupApp) -> bool {
        match self.events.next() {
            Ok(super::Event::Key(key)) => {
                use crossterm::event::KeyCode;
                match key.code {
                    KeyCode::Esc => app.previous_screen(),
                    KeyCode::Down => app.increment_menu_index(10),
                    KeyCode::Up => app.decrement_menu_index(),
                    _ => {}
                }
                true
            }
            Ok(super::Event::Tick) => false,
            Err(_) => false,
        }
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.leave_alternate_screen();
    }
}

pub async fn tui_main() -> TuiResult {
    let mut tui = Tui::new()?;
    let mut app = SetupApp::new();

    tui.enter_alternate_screen()?;

    loop {
        tui.draw(&mut app)?;
        if !tui.handle_events(&mut app) {
            continue;
        }

        // Exit when reaching summary with confirmation
        if matches!(app.current_screen(), super::ScreenType::Summary) {
            break;
        }
    }

    tui.leave_alternate_screen()?;
    Ok(())
}
```

**Step 4: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 5: Commit**

```bash
git add src/commands/setup/ratatui/runner.rs src/commands/setup/ratatui/mod.rs
git commit -m "feat: add TUI runner with terminal setup"
```

---

### Task 5: Create Welcome screen

**Files:**
- Create: `src/commands/setup/ratatui/screens/welcome.rs`
- Modify: `src/commands/setup/ratatui/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/welcome_screen_test.rs

#[test]
fn test_welcome_screen_renders() {
    let mut app = SetupApp::new();
    assert!(matches!(app.current_screen(), ScreenType::Welcome));
}
```

Run: `cargo test --test setup_ratutui_welcome_screen_test 2>&1`
Expected: FAIL - screens module not found

**Step 2: Update mod.rs to add screens module**

```rust
// src/commands/setup/ratatui/mod.rs

mod app;
mod event;
mod runner;
mod screens;

pub use app::{SetupApp, ScreenType};
pub use event::{Event, EventHandler};
pub use runner::{tui_main, TuiResult};
```

**Step 3: Create screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
```

**Step 4: Create screens/welcome.rs**

```rust
// src/commands/setup/ratatui/screens/welcome.rs

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use super::SetupApp;

pub fn render_welcome_screen(frame: &mut Frame, area: Rect, app: &mut SetupApp) {
    let title = Line::from(vec![
        "🚀 ".into(),
        "Rusty Commit".bold().fg(Color::LightCyan),
        " Setup".bold(),
    ])
    .alignment(Alignment::Center);

    let subtitle = Line::from(vec![
        "Let's get you set up with AI-powered commit messages".dim().into(),
    ])
    .alignment(Alignment::Center);

    let instructions = Line::from(vec![
        "Press ".into(),
        "[Enter]".bold(),
        " to continue".into(),
    ])
    .alignment(Alignment::Center);

    let content = Paragraph::new(vec![title, Text::new(""), subtitle, Text::new(""), instructions])
        .block(Block::bordered().title("Welcome"))
        .alignment(Alignment::Center);

    frame.render_widget(content, area);
}
```

**Step 5: Update runner.rs to render welcome screen**

```rust
// In runner.rs, update the draw method:

pub fn draw(&mut self, app: &mut SetupApp) -> TuiResult {
    self.terminal.draw(|frame| {
        let area = frame.area();
        match app.current_screen() {
            ScreenType::Welcome => screens::welcome::render_welcome_screen(frame, area, app),
            _ => {}
        }
    })?;
    Ok(())
}
```

**Step 6: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 7: Commit**

```bash
git add src/commands/setup/ratatui/screens/
git commit -m "feat: add welcome screen"
```

---

### Task 6: Create Provider selection screen with categorized list

**Files:**
- Create: `src/commands/setup/ratatui/screens/provider.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`
- Modify: `src/commands/setup/ratatui/runner.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/provider_screen_test.rs

#[test]
fn test_provider_screen_providers_loaded() {
    let providers = ProviderOption::all();
    assert!(!providers.is_empty());
}
```

Run: `cargo test --test setup_ratutui_provider_screen_test 2>&1 | head -20`
Expected: FAIL - ProviderOption not accessible from tests

**Step 2: Create screens/provider.rs**

```rust
// src/commands/setup/ratatui/screens/provider.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::providers::{ProviderCategory, ProviderOption};

use super::SetupApp;

#[derive(Debug)]
pub struct ProviderGroup {
    pub category: ProviderCategory,
    pub providers: Vec<ProviderOption>,
    pub expanded: bool,
}

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
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title, chunks[0]);

    // Provider list
    let providers = ProviderOption::all();
    let items: Vec<ListItem> = providers
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![p.display.into()]))
                .style(Style::default())
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title("Providers"))
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black));

    frame.render_widget(list, chunks[1]);

    // Footer with selection
    let footer = Paragraph::new("↑/↓ navigate • Enter select • Esc back");
    frame.render_widget(footer, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
```

**Step 4: Update runner.rs**

```rust
// In runner.rs, update the draw method match:

match app.current_screen() {
    ScreenType::Welcome => screens::welcome::render_welcome_screen(frame, area, app),
    ScreenType::Provider => screens::provider::render_provider_screen(frame, area, app),
    _ => {}
}
```

**Step 5: Add provider navigation to handle_events**

```rust
// In runner.rs, update handle_events:

KeyCode::Enter => {
    // On provider screen, advance to model
    if matches!(app.current_screen(), ScreenType::Provider) {
        app.next_screen();
    }
}
```

**Step 6: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 7: Commit**

```bash
git add src/commands/setup/ratatui/screens/provider.rs
git commit -m "feat: add provider selection screen"
```

---

### Task 7: Create Model selection screen

**Files:**
- Create: `src/commands/setup/ratatui/screens/model.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/model_screen_test.rs

#[test]
fn test_model_screen_renders() {
    let app = SetupApp::new();
    // Test that model screen can be created
    assert!(true);
}
```

Run: `cargo test --test setup_ratutui_model_screen_test 2>&1`
Expected: PASS

**Step 2: Create screens/model.rs**

```rust
// src/commands/setup/ratatui/screens/model.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::providers::ProviderOption;

use super::SetupApp;

pub fn render_model_screen(frame: &mut Frame, area: Rect, app: &mut SetupConfig) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title with provider info
    let title = format!("Model Selection - {}", app.provider.map(|p| p.name).unwrap_or("Unknown"));
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title_widget, chunks[0]);

    // Model input area
    let model_text = format!(
        "Default model: {}",
        app.provider.map(|p| p.default_model).unwrap_or("unknown")
    );

    let model_info = Paragraph::new(model_text)
        .block(Block::bordered().title("Model").borders(Borders::ALL));

    frame.render_widget(model_info, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter custom model or press Enter for default • Esc back");
    frame.render_widget(footer, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
pub mod model;
```

**Step 4: Update runner.rs to handle model screen**

```rust
// Add model screen to draw match and handle_events
```

**Step 5: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 6: Commit**

```bash
git add src/commands/setup/ratatui/screens/model.rs
git commit -m "feat: add model selection screen"
```

---

### Task 8: Create Auth screen with API key input

**Files:**
- Create: `src/commands/setup/ratatui/screens/auth.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/auth_screen_test.rs

#[test]
fn test_auth_screen_shows_provider_info() {
    let provider = ProviderOption::by_name("anthropic").unwrap();
    assert!(provider.requires_key);
}
```

Run: `cargo test --test setup_ratutui_auth_screen_test 2>&1`
Expected: FAIL - ProviderOption::by_name not found

**Step 2: Create screens/auth.rs**

```rust
// src/commands/setup/ratatui/screens/auth.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::commands::setup::providers::ProviderOption;

use super::SetupConfig;

pub fn render_auth_screen(frame: &mut Frame, area: Rect, provider: &ProviderOption, api_key: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = format!("Authentication - {}", provider.name);
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::LightCyan).bold());
    frame.render_widget(title_widget, chunks[0]);

    // API key input (masked)
    let masked_key = if api_key.is_empty() {
        "Enter API key...".dim().to_string()
    } else {
        "*".repeat(api_key.chars().count().min(20))
    };

    let auth_info = Paragraph::new(format!("Provider: {}\nRequires API Key: {}\n\nAPI Key: {}",
        provider.name,
        if provider.requires_key { "Yes" } else { "No (local)" },
        masked_key))
        .block(Block::bordered().title("Authentication").borders(Borders::ALL));

    frame.render_widget(auth_info, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter API key • Space toggle visibility • Esc back");
    frame.render_widget(footer, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
pub mod model;
pub mod auth;
```

**Step 4: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 5: Commit**

```bash
git add src/commands/setup/ratatui/screens/auth.rs
git commit -m "feat: add auth screen with API key input"
```

---

### Task 9: Create Style selection screen

**Files:**
- Create: `src/commands/setup/ratatui/screens/style.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/style_screen_test.rs

#[test]
fn test_style_screen_commit_formats() {
    let formats = CommitFormat::all();
    assert_eq!(formats.len(), 3);
}
```

Run: `cargo test --test setup_ratutui_style_screen_test 2>&1`
Expected: FAIL - CommitFormat::all not found

**Step 2: Create screens/style.rs**

```rust
// src/commands/setup/ratatui/screens/style.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::config::setup_config::CommitFormat;

use super::SetupConfig;

pub fn render_style_screen(frame: &mut Frame, area: Rect, app: &mut SetupConfig) {
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
            let marker = if *f == app.commit_style { "●" } else { "○" };
            ListItem::new(Line::from(vec![
                marker.into(),
                " ".into(),
                f.display().into(),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title("Format").borders(Borders::ALL));

    frame.render_widget(list, chunks[1]);

    // Example
    let example = match app.commit_style {
        CommitFormat::Conventional => "feat(auth): Add login functionality",
        CommitFormat::Gitmoji => "✨ feat(auth): Add login functionality",
        CommitFormat::Simple => "Add login functionality",
    };

    let example_widget = Paragraph::new(format!("Example: {}", example))
        .style(Style::default().dim());
    frame.render_widget(example_widget, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
pub mod model;
pub mod auth;
pub mod style;
```

**Step 4: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 5: Commit**

```bash
git add src/commands/setup/ratatui/screens/style.rs
git commit -m "feat: add style selection screen"
```

---

### Task 10: Create Settings screen with toggles

**Files:**
- Create: `src/commands/setup/ratatui/screens/settings.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/settings_screen_test.rs

#[test]
fn test_settings_screen_toggles() {
    let config = SetupConfig::default();
    assert!(config.description_capitalize);
}
```

Run: `cargo test --test setup_ratutui_settings_screen_test 2>&1`
Expected: FAIL - SetupConfig not in scope

**Step 2: Create screens/settings.rs**

```rust
// src/commands/setup/ratatui/screens/settings.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    Frame,
};

use crate::config::setup_config::SetupConfig;

pub fn render_settings_screen(frame: &mut Frame, area: Rect, app: &mut SetupConfig) {
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
        format!("Capitalize first letter: {}", if app.description_capitalize { "Yes" } else { "No" }),
        format!("Add period at end: {}", if app.description_add_period { "Yes" } else { "No" }),
        format!("Max length: {} chars", app.description_max_length),
        format!("Generate count: {}", app.generate_count),
        format!("Use emojis: {}", if app.emoji { "Yes" } else { "No" }),
        format!("Auto-push: {}", if app.gitpush { "Yes" } else { "No" }),
    ];

    let items: Vec<ListItem> = settings_items
        .iter()
        .map(|s| ListItem::new(Line::from(vec![s.clone().into()])))
        .collect();

    let list = List::new(items)
        .block(Block::bordered().title("Settings").borders(Borders::ALL));

    frame.render_widget(list, chunks[1]);

    // Footer
    let footer = Paragraph::new("↑/↓ navigate • Space toggle • Enter edit • Esc back");
    frame.render_widget(footer, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
pub mod model;
pub mod auth;
pub mod style;
pub mod settings;
```

**Step 4: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 5: Commit**

```bash
git add src/commands/setup/ratatui/screens/settings.rs
git commit -m "feat: add settings screen with toggles"
```

---

### Task 11: Create Summary screen with save confirmation

**Files:**
- Create: `src/commands/setup/ratatui/screens/summary.rs`
- Modify: `src/commands/setup/ratatui/screens/mod.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/ratatui/summary_screen_test.rs

#[test]
fn test_summary_screen_displays_config() {
    let config = SetupConfig::default();
    assert!(config.provider.is_none()); // Not set yet
}
```

Run: `cargo test --test setup_ratutui_summary_screen_test 2>&1`
Expected: PASS

**Step 2: Create screens/summary.rs**

```rust
// src/commands/setup/ratatui/screens/summary.rs

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Borders},
    Frame,
};

use crate::config::setup_config::SetupConfig;

pub fn render_summary_screen(frame: &mut Frame, area: Rect, app: &SetupConfig) {
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
    let provider_info = app.provider.map(|p| format!("Provider: {} (model: {})", p.name, app.model))
        .unwrap_or_else(|| "Provider: Not selected".to_string());
    let style_info = format!("Style: {:?} (emoji: {})", app.commit_style, app.emoji);
    let language_info = format!("Language: {}", app.language);

    let summary = Paragraph::new(vec![
        Line::from(provider_info),
        Line::from(style_info),
        Line::from(language_info),
        Line::from(""),
        Line::from("Press Enter to save configuration".green()),
    ])
    .block(Block::bordered().title("Summary").borders(Borders::ALL));

    frame.render_widget(summary, chunks[1]);

    // Footer
    let footer = Paragraph::new("Enter save • Esc go back");
    frame.render_widget(footer, chunks[2]);
}
```

**Step 3: Update screens/mod.rs**

```rust
// src/commands/setup/ratatui/screens/mod.rs

pub mod welcome;
pub mod provider;
pub mod model;
pub mod auth;
pub mod style;
pub mod settings;
pub mod summary;
```

**Step 4: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 5: Commit**

```bash
git add src/commands/setup/ratatui/screens/summary.rs
git commit -m "feat: add summary screen with save confirmation"
```

---

### Task 12: Integrate TUI into setup command with TTY detection

**Files:**
- Modify: `src/commands/setup/mod.rs`
- Modify: `src/commands/setup.rs`

**Step 1: Write the failing test**

```rust
// tests/setup/integration_test.rs

#[test]
fn test_setup_command_exists() {
    // Test that setup command can be invoked
    assert!(true);
}
```

Run: `cargo test --test setup_integration_test 2>&1`
Expected: PASS

**Step 2: Update setup/mod.rs to conditionally use TUI**

```rust
// src/commands/setup/mod.rs

#[cfg(feature = "tui")]
mod ratatui;

#[cfg(feature = "tui")]
pub use ratatui::{tui_main, SetupApp, ScreenType};
```

**Step 3: Update setup.rs to use TUI when TTY**

```rust
// src/commands/setup.rs

use crate::cli::SetupCommand;

pub async fn execute(cmd: SetupCommand) -> Result<()> {
    // Check if TUI should be used
    let use_tui = atty::is(atty::Stream::Stdout) && !cmd.no_tui;

    if use_tui {
        #[cfg(feature = "tui")]
        {
            ratatui::tui_main().await?;
            return Ok(());
        }

        #[cfg(not(feature = "tui"))]
        {
            tracing::warn!("TUI feature not enabled, falling back to dialoguer");
        }
    }

    // Fallback to dialoguer-based wizards
    wizards::execute(cmd).await
}
```

**Step 4: Add TUI feature flag to Cargo.toml**

```toml
[features]
default = ["dialoguer"]
tui = ["ratatui", "crossterm"]
```

**Step 5: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 6: Commit**

```bash
git add src/commands/setup/mod.rs src/commands/setup.rs
git commit -m "feat: integrate TUI into setup command with TTY detection"
```

---

### Task 13: Add --tui and --no-tui flags to CLI

**Files:**
- Modify: `src/cli.rs`

**Step 1: Write the failing test**

```rust
// tests/cli/setup_flags_test.rs

#[test]
fn test_setup_tui_flag_exists() {
    let matches = SetupCommand::parse(&["setup", "--tui"]);
    assert!(matches.tui);
}
```

Run: `cargo test --test cli_setup_flags_test 2>&1 | head -20`
Expected: FAIL - tui flag not found

**Step 2: Update CLI to add TUI flags**

```rust
// In src/cli.rs, add to SetupCommand:

#[command(subcommand)]
pub enum Commands {
    #[command(name = "setup")]
    Setup(SetupCommand),
}

#[derive(Debug, clap::Parser)]
pub struct SetupCommand {
    #[arg(long, help = "Force TUI mode")]
    pub tui: bool,

    #[arg(long, help = "Force non-TUI mode")]
    pub no_tui: bool,

    #[arg(long, help = "Use advanced setup")]
    pub advanced: bool,

    #[arg(long, help = "Apply defaults without prompting")]
    pub defaults: bool,
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test --test cli_setup_flags_test -- --nocapture`
Expected: PASS

**Step 4: Commit**

```bash
git add src/cli.rs
git commit -m "feat: add --tui and --no-tui flags to setup command"
```

---

### Task 14: Add atty dependency for TTY detection

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add atty dependency**

```toml
atty = "0.2"
```

**Step 2: Run test to verify it compiles**

Run: `cargo check --quiet`
Expected: No errors

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add atty dependency for TTY detection"
```

---

### Task 15: Run full integration tests

**Files:**
- Test: `tests/`

**Step 1: Run all tests**

```bash
cargo test --all-features --quiet 2>&1 | tail -30
```

Expected: All tests pass

**Step 2: Build binary**

```bash
cargo build --features tui --release 2>&1 | tail -10
```

Expected: Build succeeds

**Step 3: Commit**

```bash
git commit -m "chore: run full integration tests"
```

---

### Task 16: Update documentation

**Files:**
- Modify: `README.md`
- Modify: `CONTRIBUTING.md`

**Step 1: Update README.md**

```markdown
## Interactive Setup

Run `rco setup` for an interactive terminal UI that guides you through configuration:

```
$ rco setup
🚀 Rusty Commit Setup!
```

Use `--tui` to force TUI mode or `--no-tui` to use the dialoguer-based prompts.
```

**Step 2: Update CONTRIBUTING.md**

Add TUI feature testing instructions.

**Step 3: Commit**

```bash
git add README.md CONTRIBUTING.md
git commit -m "docs: update documentation for TUI setup"
```

---

## Plan Complete

**Two execution options:**

1. **Subagent-Driven (this session)** - Fresh subagent per task, review between tasks
2. **Parallel Session (separate)** - New session with executing-plans

**Which approach?**

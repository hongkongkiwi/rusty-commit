# TUI Setup Design

**Date:** 2026-02-04
**Status:** Approved

## Overview

Add an interactive TUI (Terminal User Interface) for `rco setup` using Ratatui, providing a menu-driven configuration experience with keyboard navigation.

## Architecture

```
src/commands/setup/ratatui/
├── app.rs              # Main app, event loop, terminal setup
├── config.rs           # SetupConfig struct (what we're building)
├── event.rs           # Key event handling (crossterm Events)
├── components/
│   ├── header.rs      # App title, progress indicator
│   ├── footer.rs      # Help bar
│   ├── menu.rs        # ListWidget with collapsible categories
│   ├── input.rs       # Password/input fields with validation
│   ├── toggle.rs      # Boolean settings
│   └── help_bar.rs    # Shortcut reference
├── screens/
│   ├── welcome.rs     # Welcome screen, mode selection
│   ├── provider.rs    # Provider list (categorized)
│   ├── model.rs       # Model selection or custom input
│   ├── auth.rs        # API key input, OAuth URL display
│   ├── style.rs       # Commit format selection
│   ├── settings.rs    # Toggles: capitalize, period, max_length, etc.
│   └── summary.rs     # Review config, save/cancel
└── widgets/
    ├── list.rs        # Custom list with section headers
    └── input.rs       # Stateful input widget
```

## Dependencies

```toml
ratatui = "0.28"
crossterm = "0.28"
```

## App State

```rust
pub struct SetupApp {
    current_screen: ScreenType,
    previous_screen: Option<ScreenType>,
    config: SetupConfig,
    menu_index: usize,
    scroll_offset: usize,
    events: EventHandler,
}

enum ScreenType {
    Welcome,
    Provider,
    Model { provider: ProviderOption },
    ApiKey { provider: ProviderOption },
    Style,
    Settings,
    Summary,
}

struct SetupConfig {
    provider: Option<ProviderOption>,
    model: String,
    api_key: Option<String>,
    api_url: Option<String>,
    commit_style: CommitFormat,
    language: String,
    // ... from wizards.rs
}
```

## Navigation Flow

```
welcome ──→ provider ──→ model ──→ auth ──→ style ──→ settings ──→ summary ──→ save
                ↓
           (local providers skip auth)
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate menu |
| `Enter` | Select / Confirm |
| `Esc` | Go back |
| `Space` | Toggle boolean |
| `q` / `Ctrl+C` | Quit (with confirmation) |
| `Tab` | Switch focus between panels |

## Auth Patterns

Each provider has:

- `requires_key: bool` - whether API key is needed
- `has_oauth: bool` - whether OAuth flow is available
- `custom_config: Vec<(name, prompt, default)>` - extra fields (AWS region, etc.)

### Auth Screen Layout

```
┌─ Select Provider ────────────────────────────────────────────────┐
│  Provider: Anthropic                                              │
│                                                               │
│  ┌─ Authentication Method ──────────────────────────────────┐  │
│  │  ○ API Key (recommended for most users)                  │  │
│  │  ○ OAuth (if available)                                  │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌─ API Key ────────────────────────────────────────────────┐   │
│  │  [••••••••••••••••••••••••••••••••••••••••••••••••]    │   │
│  │  sk-ant-api03-...                                       │   │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                               │
│  ℹ  Get your key from: console.anthropic.com                  │
│                                                               │
│  [← Back]  [Skip]  [Next →]                                   │
└───────────────────────────────────────────────────────────────┘
```

## Fallback Strategy

- TTY detection: if not a TTY, fall back to dialoguer-based prompts
- `--no-tui` flag to force dialoguer mode

## Provider Data (from providers.rs)

Categories:
- Popular (OpenAI, Anthropic, Gemini)
- Local (Ollama, LM Studio, llama.cpp)
- Cloud (Groq, Cerebras, SambaNova, xAI, DeepSeek, etc.)
- Enterprise (Azure, Bedrock, Vertex, Cohere, AI21)
- Specialized (Jina, Helicone)

## Implementation Phases

1. **Phase 1:** App skeleton, event loop, terminal setup
2. **Phase 2:** Welcome, Provider, Model screens
3. **Phase 3:** Auth screen with API key input
4. **Phase 4:** Style, Settings screens
5. **Phase 5:** Summary screen and save logic
6. **Phase 6:** Fallback strategy and testing

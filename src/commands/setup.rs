//! Setup command module
//!
//! Splits the original setup.rs into focused sub-modules:
//! - providers.rs: ProviderOption, CommitFormat, LanguageOption data types
//! - wizards.rs: run_quick_setup(), run_advanced_setup(), apply_defaults()
//! - prompts.rs: Selection helper functions (select_provider, select_language, etc.)
//! - ui.rs: UI utilities (print_section_header, print_completion_message)

pub mod providers;
pub mod prompts;
pub mod ui;
pub mod wizards;

pub use wizards::execute;

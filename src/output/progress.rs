//! Enhanced progress tracking for Rusty Commit CLI.
//!
//! Provides multi-step progress indicators with timing and status tracking.

use std::time::Instant;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use super::styling::{Color, Palette, Theme};

/// Default progress spinner characters.
#[allow(dead_code)]
static SPINNER_CHARS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Step status for multi-step progress.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Step is pending (not started).
    Pending,
    /// Step is currently in progress.
    Active,
    /// Step completed successfully.
    Completed,
    /// Step failed.
    Failed,
    /// Step was skipped.
    Skipped,
}

/// A single step in a multi-step workflow.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Step {
    /// The title of this step.
    title: String,
    /// Optional detail text (shown when active).
    detail: Option<String>,
    /// Current status of this step.
    status: StepStatus,
    /// When this step started (for timing).
    started_at: Option<Instant>,
    /// How long this step took (when completed).
    duration_ms: Option<u64>,
}

#[allow(dead_code)]
impl Step {
    /// Create a new pending step.
    pub fn pending(title: &str) -> Self {
        Self {
            title: title.to_string(),
            detail: None,
            status: StepStatus::Pending,
            started_at: None,
            duration_ms: None,
        }
    }

    /// Create a new active step with detail.
    pub fn active(title: &str, detail: &str) -> Self {
        Self {
            title: title.to_string(),
            detail: Some(detail.to_string()),
            status: StepStatus::Active,
            started_at: Some(Instant::now()),
            duration_ms: None,
        }
    }

    /// Mark this step as completed.
    pub fn completed(mut self) -> Self {
        self.status = StepStatus::Completed;
        if let Some(start) = self.started_at {
            self.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
        self
    }

    /// Mark this step as failed.
    pub fn failed(mut self) -> Self {
        self.status = StepStatus::Failed;
        if let Some(start) = self.started_at {
            self.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
        self
    }

    /// Set detail text.
    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    /// Get the current status.
    pub fn status(&self) -> StepStatus {
        self.status
    }

    /// Get the title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the detail text.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// Get the duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration_ms
    }
}

/// Enhanced progress tracker with multi-step support.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    /// The underlying progress bar.
    progress_bar: ProgressBar,
    /// Theme for styling.
    theme: Theme,
    /// Steps in the workflow.
    steps: Vec<Step>,
    /// Current step index.
    current_step: usize,
    /// Overall start time.
    started_at: Instant,
    /// Timing breakdown for each step.
    timing_breakdown: Vec<(String, u64)>,
}

#[allow(dead_code)]
impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        Self {
            progress_bar: pb,
            theme: Theme::new(),
            steps: Vec::new(),
            current_step: 0,
            started_at: Instant::now(),
            timing_breakdown: Vec::new(),
        }
    }

    /// Create a new tracker with theme.
    pub fn with_theme(message: &str, theme: Theme) -> Self {
        let mut tracker = Self::new(message);
        tracker.theme = theme;
        tracker
    }

    /// Add steps to the tracker.
    pub fn steps(mut self, steps: &[Step]) -> Self {
        self.steps = steps.to_vec();
        self
    }

    /// Set the current step as active.
    pub fn set_active(&mut self, index: usize) {
        if index < self.steps.len() {
            self.current_step = index;
            self.steps[index].started_at = Some(Instant::now());
            self.update_message();
        }
    }

    /// Set detail text for current step.
    pub fn set_detail(&mut self, detail: &str) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].detail = Some(detail.to_string());
            self.update_message();
        }
    }

    /// Mark the current step as completed.
    pub fn complete_current(&mut self) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].status = StepStatus::Completed;
            if let Some(start) = self.steps[self.current_step].started_at {
                let duration = start.elapsed().as_millis() as u64;
                self.timing_breakdown
                    .push((self.steps[self.current_step].title.clone(), duration));
            }
            self.current_step += 1;
            self.update_message();
        }
    }

    /// Mark a specific step as completed.
    pub fn complete_step(&mut self, index: usize) {
        if index < self.steps.len() {
            self.steps[index].status = StepStatus::Completed;
            if let Some(start) = self.steps[index].started_at {
                let duration = start.elapsed().as_millis() as u64;
                self.timing_breakdown
                    .push((self.steps[index].title.clone(), duration));
            }
            self.update_message();
        }
    }

    /// Mark the current step as failed.
    pub fn fail_current(&mut self) {
        if self.current_step < self.steps.len() {
            self.steps[self.current_step].status = StepStatus::Failed;
            self.update_message();
        }
    }

    fn update_message(&self) {
        if self.current_step < self.steps.len() {
            let step = &self.steps[self.current_step];
            let msg = match step.status {
                StepStatus::Active => {
                    if let Some(detail) = &step.detail {
                        format!("{} ({})", step.title, detail)
                    } else {
                        step.title.clone()
                    }
                }
                _ => step.title.clone(),
            };
            self.progress_bar.set_message(msg);
        }
    }

    /// Finish with a success message.
    pub fn finish_with_success(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }

    /// Finish with an error message.
    pub fn finish_with_error(&self, message: &str) {
        self.progress_bar.finish_with_message(message.to_string());
    }

    /// Get the timing breakdown.
    pub fn timing_breakdown(&self) -> &[(String, u64)] {
        &self.timing_breakdown
    }

    /// Get total elapsed time in milliseconds.
    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    /// Get formatted elapsed time.
    pub fn elapsed_formatted(&self) -> String {
        let ms = self.elapsed_ms();
        if ms < 1000 {
            format!("{}ms", ms)
        } else {
            format!("{:.1}s", ms as f64 / 1000.0)
        }
    }

    /// Get the progress bar for external control.
    pub fn progress_bar(&self) -> &ProgressBar {
        &self.progress_bar
    }

    /// Get mutable progress bar for external control.
    pub fn progress_bar_mut(&mut self) -> &mut ProgressBar {
        &mut self.progress_bar
    }
}

/// Convenience function to create a simple spinner.
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// OAuth wait spinner with consistent styling for authentication flows.
pub fn oauth_wait_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message("Waiting for authentication...".to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Create a styled progress bar with custom template.
#[allow(dead_code)]
pub fn styled_progress(message: &str, palette: &Palette) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    let template = format!(
        "{{spinner:.{}}} {{msg}}",
        match palette.primary {
            Color::MutedBlue | Color::Cyan => "cyan",
            Color::Green => "green",
            Color::Red => "red",
            Color::Amber => "yellow",
            Color::Purple => "magenta",
            _ => "green",
        }
    );
    pb.set_style(ProgressStyle::default_spinner().template(&template).unwrap());
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Format timing breakdown as a string.
#[allow(dead_code)]
pub fn format_timing_breakdown(breakdown: &[(String, u64)], total_ms: u64) -> String {
    let mut result = String::new();

    for (name, duration) in breakdown {
        let duration_str = if *duration < 1000 {
            format!("{}ms", duration)
        } else {
            format!("{:.1}s", *duration as f64 / 1000.0)
        };
        result.push_str(&format!("  {} {}\n", name.dimmed(), duration_str.green()));
    }

    // Add total
    let total_str = if total_ms < 1000 {
        format!("{}ms", total_ms)
    } else {
        format!("{:.1}s", total_ms as f64 / 1000.0)
    };
    result.push_str(&format!("  {} {}", "Total".dimmed(), total_str.green()));

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_pending() {
        let step = Step::pending("Test Step");
        assert_eq!(step.title(), "Test Step");
        assert_eq!(step.status(), StepStatus::Pending);
        assert!(step.detail().is_none());
    }

    #[test]
    fn test_step_active() {
        let step = Step::active("Active Step", "details here");
        assert_eq!(step.title(), "Active Step");
        assert_eq!(step.status(), StepStatus::Active);
        assert_eq!(step.detail(), Some("details here"));
    }

    #[test]
    fn test_step_completed() {
        let step = Step::active("Test", "detail").completed();
        assert_eq!(step.status(), StepStatus::Completed);
        assert!(step.duration_ms().is_some());
    }

    #[test]
    fn test_format_timing_breakdown_empty() {
        let result = format_timing_breakdown(&[], 0);
        assert!(result.contains("Total"));
    }

    #[test]
    fn test_format_timing_breakdown_with_items() {
        let breakdown = vec![("Step1".to_string(), 100u64), ("Step2".to_string(), 500u64)];
        let result = format_timing_breakdown(&breakdown, 600);
        assert!(result.contains("Step1"));
        assert!(result.contains("Step2"));
        assert!(result.contains("100ms"));
        assert!(result.contains("500ms"));
        assert!(result.contains("600ms"));
    }

    #[test]
    fn test_format_timing_breakdown_seconds() {
        let breakdown = vec![("Long Step".to_string(), 2500u64)];
        let result = format_timing_breakdown(&breakdown, 2500);
        assert!(result.contains("2.5s"));
    }
}

//! Configuration for output display.

use std::io::IsTerminal;

/// When to display output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputMode {
    /// Always show output regardless of test result.
    Always,
    /// Only show output when tests fail (default).
    #[default]
    OnFailure,
    /// Never show output.
    Never,
}

/// Configuration for output display.
///
/// Use the builder pattern to configure what gets displayed:
///
/// ```rust,ignore
/// use aptitude::output::{OutputConfig, OutputMode};
///
/// let config = OutputConfig::new()
///     .tool_calls(OutputMode::Always)
///     .response(OutputMode::OnFailure)
///     .truncate_at(80);
/// ```
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// When to show tool calls made during execution.
    pub tool_calls: OutputMode,
    /// When to show Claude's response output.
    pub response: OutputMode,
    /// Maximum characters before truncating parameter values.
    pub truncate_at: usize,
    /// Whether to use ANSI colors in output.
    pub colors_enabled: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            tool_calls: OutputMode::OnFailure,
            response: OutputMode::OnFailure,
            truncate_at: 60,
            colors_enabled: std::io::stdout().is_terminal(),
        }
    }
}

impl OutputConfig {
    /// Create a new output configuration with defaults.
    ///
    /// Default: `OnFailure` for both tool calls and response,
    /// 60 character truncation, colors auto-detected from TTY.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure when to show tool calls.
    pub fn tool_calls(mut self, mode: OutputMode) -> Self {
        self.tool_calls = mode;
        self
    }

    /// Configure when to show Claude's response.
    pub fn response(mut self, mode: OutputMode) -> Self {
        self.response = mode;
        self
    }

    /// Set the maximum characters before truncating parameter values.
    pub fn truncate_at(mut self, chars: usize) -> Self {
        self.truncate_at = chars;
        self
    }

    /// Enable or disable ANSI colors.
    pub fn colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    /// Create a verbose configuration that always shows everything.
    pub fn verbose() -> Self {
        Self {
            tool_calls: OutputMode::Always,
            response: OutputMode::Always,
            ..Self::default()
        }
    }

    /// Create a quiet configuration that never shows output.
    pub fn quiet() -> Self {
        Self {
            tool_calls: OutputMode::Never,
            response: OutputMode::Never,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OutputConfig::new();
        assert_eq!(config.tool_calls, OutputMode::OnFailure);
        assert_eq!(config.response, OutputMode::OnFailure);
        assert_eq!(config.truncate_at, 60);
    }

    #[test]
    fn test_verbose_config() {
        let config = OutputConfig::verbose();
        assert_eq!(config.tool_calls, OutputMode::Always);
        assert_eq!(config.response, OutputMode::Always);
    }

    #[test]
    fn test_quiet_config() {
        let config = OutputConfig::quiet();
        assert_eq!(config.tool_calls, OutputMode::Never);
        assert_eq!(config.response, OutputMode::Never);
    }

    #[test]
    fn test_builder_chain() {
        let config = OutputConfig::new()
            .tool_calls(OutputMode::Always)
            .response(OutputMode::Never)
            .truncate_at(100)
            .colors(false);

        assert_eq!(config.tool_calls, OutputMode::Always);
        assert_eq!(config.response, OutputMode::Never);
        assert_eq!(config.truncate_at, 100);
        assert!(!config.colors_enabled);
    }
}

//! Output formatting for tool calls and responses.

use crate::output::config::{OutputConfig, OutputMode};
use crate::parser::ToolCall;
use serde_json::Value;

// ANSI color codes
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

/// Formatter for test output including tool calls and agent responses.
pub struct OutputFormatter {
    config: OutputConfig,
}

impl OutputFormatter {
    /// Create a new formatter with the given configuration.
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Create a formatter with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(OutputConfig::new())
    }

    /// Check if tool calls should be shown given the test result.
    pub fn should_show_tool_calls(&self, test_passed: bool) -> bool {
        match self.config.tool_calls {
            OutputMode::Always => true,
            OutputMode::OnFailure => !test_passed,
            OutputMode::Never => false,
        }
    }

    /// Check if response should be shown given the test result.
    pub fn should_show_response(&self, test_passed: bool) -> bool {
        match self.config.response {
            OutputMode::Always => true,
            OutputMode::OnFailure => !test_passed,
            OutputMode::Never => false,
        }
    }

    /// Format a parameter value, truncating if necessary.
    pub fn format_params(&self, params: &Value) -> String {
        if let Some(obj) = params.as_object() {
            let parts: Vec<String> = obj
                .iter()
                .map(|(k, v)| {
                    let val_str = match v {
                        Value::String(s) => {
                            let truncated = self.truncate(s);
                            format!("\"{}\"", truncated)
                        }
                        other => {
                            let s = other.to_string();
                            self.truncate(&s)
                        }
                    };
                    format!("{}={}", k, val_str)
                })
                .collect();
            parts.join(", ")
        } else {
            params.to_string()
        }
    }

    /// Format a single tool call for display.
    pub fn format_tool_call(&self, call: &ToolCall) -> String {
        let params_str = self.format_params(&call.params);
        let timestamp = call.timestamp.format("%H:%M:%S");

        if self.config.colors_enabled {
            format!(
                "  [{}] {}{}{} {}",
                timestamp, CYAN, call.name, RESET, params_str
            )
        } else {
            format!("  [{}] {} {}", timestamp, call.name, params_str)
        }
    }

    /// Print tool calls if the output mode allows it.
    pub fn print_tool_calls(&self, calls: &[ToolCall], test_passed: bool) {
        if !self.should_show_tool_calls(test_passed) {
            return;
        }

        println!();
        if self.config.colors_enabled {
            println!("{}Tool calls made during execution:{}", YELLOW, RESET);
        } else {
            println!("Tool calls made during execution:");
        }

        if calls.is_empty() {
            println!("  (no tool calls)");
        } else {
            for call in calls {
                println!("{}", self.format_tool_call(call));
            }
        }
    }

    /// Print Claude's response if the output mode allows it.
    pub fn print_response(&self, response: Option<&str>, test_passed: bool) {
        if !self.should_show_response(test_passed) {
            return;
        }

        if let Some(stdout) = response {
            if !stdout.is_empty() {
                println!();
                if self.config.colors_enabled {
                    println!("{}Claude's response:{}", YELLOW, RESET);
                } else {
                    println!("Claude's response:");
                }
                for line in stdout.lines() {
                    println!("  {}", line);
                }
            }
        }
    }

    /// Truncate a string to the configured maximum length.
    /// Handles multi-byte UTF-8 characters safely.
    fn truncate(&self, s: &str) -> String {
        let max = self.config.truncate_at;
        let char_count = s.chars().count();

        if char_count <= max {
            s.to_string()
        } else {
            // Reserve 3 chars for "..."
            let truncated: String = s.chars().take(max.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_truncate_short_string() {
        let formatter = OutputFormatter::new(OutputConfig::new().truncate_at(60));
        assert_eq!(formatter.truncate("hello"), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let formatter = OutputFormatter::new(OutputConfig::new().truncate_at(10));
        assert_eq!(formatter.truncate("hello world!"), "hello w...");
    }

    #[test]
    fn test_truncate_unicode() {
        let formatter = OutputFormatter::new(OutputConfig::new().truncate_at(6));
        // Input: "日本語ですよね" (7 chars), truncate_at: 6
        // Should truncate to 3 chars + "..." = 6 total
        let result = formatter.truncate("日本語ですよね"); // 7 chars
        assert!(result.ends_with("..."));
        assert_eq!(result.chars().count(), 6); // 3 chars + "..."
        assert_eq!(result, "日本語...");
    }

    #[test]
    fn test_format_params_object() {
        let formatter = OutputFormatter::new(OutputConfig::new());
        let params = json!({"file_path": "/tmp/test.txt", "content": "hello"});
        let formatted = formatter.format_params(&params);
        assert!(formatted.contains("file_path="));
        assert!(formatted.contains("content="));
    }

    #[test]
    fn test_should_show_always() {
        let config = OutputConfig::new().tool_calls(OutputMode::Always);
        let formatter = OutputFormatter::new(config);
        assert!(formatter.should_show_tool_calls(true));
        assert!(formatter.should_show_tool_calls(false));
    }

    #[test]
    fn test_should_show_on_failure() {
        let config = OutputConfig::new().tool_calls(OutputMode::OnFailure);
        let formatter = OutputFormatter::new(config);
        assert!(!formatter.should_show_tool_calls(true));
        assert!(formatter.should_show_tool_calls(false));
    }

    #[test]
    fn test_should_show_never() {
        let config = OutputConfig::new().tool_calls(OutputMode::Never);
        let formatter = OutputFormatter::new(config);
        assert!(!formatter.should_show_tool_calls(true));
        assert!(!formatter.should_show_tool_calls(false));
    }
}

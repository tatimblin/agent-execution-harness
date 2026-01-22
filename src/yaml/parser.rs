//! YAML parsing and tool name resolution.
//!
//! This module handles YAML deserialization and string-to-Tool enum conversion.
//! All string parsing logic (case handling, aliases) lives here.

use crate::fluent::Tool;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Error type for YAML parsing issues.
#[derive(Debug, thiserror::Error)]
pub enum YamlError {
    #[error("Unknown tool: '{0}'. Available tools: Read, Write, Edit, Bash, Glob, Grep, Task, WebFetch, WebSearch, NotebookEdit, AskUserQuestion, TodoWrite")]
    UnknownTool(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// A test loaded from YAML.
#[derive(Debug, Deserialize)]
pub struct Test {
    /// Human-readable name for this test.
    pub name: String,
    /// The prompt to send to the agent.
    pub prompt: String,
    /// Agent to use for this test (defaults to "claude").
    #[serde(default)]
    pub agent: Option<String>,
    /// List of assertions to evaluate.
    pub assertions: Vec<Assertion>,
}

/// A single assertion about tool usage.
#[derive(Debug, Deserialize)]
pub struct Assertion {
    /// Tool name (case-insensitive, supports aliases).
    pub tool: String,
    /// Whether this tool should be called (default: true).
    #[serde(default = "default_true")]
    pub called: bool,
    /// Parameter patterns to match (glob, regex, or exact).
    pub params: Option<HashMap<String, String>>,
    /// Assert this tool is called after another tool.
    pub called_after: Option<String>,
    /// Assert this tool is called before another tool.
    pub called_before: Option<String>,
    /// Assert exact number of times the tool was called.
    pub call_count: Option<u32>,
    /// Assert maximum number of times the tool can be called.
    pub max_calls: Option<u32>,
    /// Assert minimum number of times the tool must be called.
    pub min_calls: Option<u32>,
    /// Assert parameters for specific call indices (1-based).
    pub nth_call_params: Option<HashMap<u32, HashMap<String, String>>>,
    /// Assert parameters for the first call.
    pub first_call_params: Option<HashMap<String, String>>,
    /// Assert parameters for the last call.
    pub last_call_params: Option<HashMap<String, String>>,
}

fn default_true() -> bool {
    true
}

/// Load a test from a YAML file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The YAML is malformed
///
/// # Example
///
/// ```rust,ignore
/// let test = load_test("tests/read_file.yaml")?;
/// println!("Running: {}", test.name);
/// ```
pub fn load_test(path: &Path) -> Result<Test> {
    let content = fs::read_to_string(path).context("Failed to read test file")?;
    let test: Test = serde_yaml::from_str(&content).context("Failed to parse YAML")?;
    Ok(test)
}

/// Parse a tool name string into a Tool enum.
///
/// This function handles:
/// - Case-insensitive matching (read, READ, Read all work)
/// - Legacy snake_case aliases (read_file -> Read)
///
/// # Errors
///
/// Returns `YamlError::UnknownTool` if the string doesn't match any known tool.
///
/// # Example
///
/// ```rust
/// use aptitude::yaml::parse_tool_name;
/// use aptitude::Tool;
///
/// assert_eq!(parse_tool_name("Read").unwrap(), Tool::Read);
/// assert_eq!(parse_tool_name("read").unwrap(), Tool::Read);
/// assert_eq!(parse_tool_name("read_file").unwrap(), Tool::Read);
/// ```
pub fn parse_tool_name(s: &str) -> Result<Tool, YamlError> {
    // Case-insensitive exact matches first
    match s.to_lowercase().as_str() {
        // Primary names (match JSONL output)
        "read" => Ok(Tool::Read),
        "write" => Ok(Tool::Write),
        "edit" => Ok(Tool::Edit),
        "bash" => Ok(Tool::Bash),
        "glob" => Ok(Tool::Glob),
        "grep" => Ok(Tool::Grep),
        "task" => Ok(Tool::Task),
        "webfetch" => Ok(Tool::WebFetch),
        "websearch" => Ok(Tool::WebSearch),
        "notebookedit" => Ok(Tool::NotebookEdit),
        "askuserquestion" => Ok(Tool::AskUserQuestion),
        "todowrite" => Ok(Tool::TodoWrite),
        "killshell" => Ok(Tool::KillShell),
        "taskoutput" => Ok(Tool::TaskOutput),

        // Legacy snake_case aliases (for backward compatibility)
        "read_file" => Ok(Tool::Read),
        "write_file" => Ok(Tool::Write),
        "edit_file" => Ok(Tool::Edit),
        "execute_command" => Ok(Tool::Bash),
        "glob_files" => Ok(Tool::Glob),
        "search_files" => Ok(Tool::Grep),
        "web_fetch" => Ok(Tool::WebFetch),
        "web_search" => Ok(Tool::WebSearch),
        "notebook_edit" => Ok(Tool::NotebookEdit),
        "ask_user" | "ask_user_question" => Ok(Tool::AskUserQuestion),
        "todo_write" => Ok(Tool::TodoWrite),
        "kill_shell" => Ok(Tool::KillShell),
        "task_output" => Ok(Tool::TaskOutput),

        _ => Err(YamlError::UnknownTool(s.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_name_primary() {
        assert_eq!(parse_tool_name("Read").unwrap(), Tool::Read);
        assert_eq!(parse_tool_name("Write").unwrap(), Tool::Write);
        assert_eq!(parse_tool_name("Bash").unwrap(), Tool::Bash);
        assert_eq!(parse_tool_name("WebFetch").unwrap(), Tool::WebFetch);
    }

    #[test]
    fn test_parse_tool_name_case_insensitive() {
        assert_eq!(parse_tool_name("read").unwrap(), Tool::Read);
        assert_eq!(parse_tool_name("READ").unwrap(), Tool::Read);
        assert_eq!(parse_tool_name("ReAd").unwrap(), Tool::Read);
    }

    #[test]
    fn test_parse_tool_name_aliases() {
        assert_eq!(parse_tool_name("read_file").unwrap(), Tool::Read);
        assert_eq!(parse_tool_name("write_file").unwrap(), Tool::Write);
        assert_eq!(parse_tool_name("execute_command").unwrap(), Tool::Bash);
        assert_eq!(parse_tool_name("search_files").unwrap(), Tool::Grep);
    }

    #[test]
    fn test_parse_tool_name_unknown() {
        assert!(parse_tool_name("unknown_tool").is_err());
        assert!(parse_tool_name("").is_err());
    }

    #[test]
    fn test_deserialize_assertion() {
        let yaml = r#"
tool: Read
called: true
params:
  file_path: "*.txt"
"#;
        let assertion: Assertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(assertion.tool, "Read");
        assert!(assertion.called);
        assert!(assertion.params.is_some());
    }

    #[test]
    fn test_deserialize_test() {
        let yaml = r#"
name: "Test reading files"
prompt: "Read the config"
assertions:
  - tool: Read
    called: true
"#;
        let test: Test = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(test.name, "Test reading files");
        assert_eq!(test.prompt, "Read the config");
        assert_eq!(test.assertions.len(), 1);
    }

    #[test]
    fn test_default_called_true() {
        let yaml = r#"
tool: Read
"#;
        let assertion: Assertion = serde_yaml::from_str(yaml).unwrap();
        assert!(assertion.called);
    }
}

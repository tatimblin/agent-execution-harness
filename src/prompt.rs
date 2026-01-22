//! Prompt builder for fluent prompt configuration and execution.
//!
//! This module provides a builder pattern for configuring and executing prompts
//! against AI agents.
//!
//! # Example
//!
//! ```rust,ignore
//! use agent_harness::{prompt, expect, Tool};
//!
//! let tool_calls = prompt("Read the config file")
//!     .in_dir("/path/to/project")
//!     .run()
//!     .unwrap();
//!
//! expect(&tool_calls).tool(Tool::Read).to_be_called();
//! ```
//!
//! # With Output Display
//!
//! ```rust,ignore
//! use agent_harness::{prompt, expect, Tool, OutputConfig};
//!
//! let result = prompt("Read the config file")
//!     .with_output(OutputConfig::verbose())
//!     .run_full()
//!     .unwrap();
//!
//! expect(&result.tool_calls).tool(Tool::Read).to_be_called();
//! ```

use crate::agents::{AgentHarness, AgentType, ExecutionConfig, NormalizedResult};
use crate::output::{OutputConfig, OutputFormatter};
use crate::parser::ToolCall;
use std::path::PathBuf;

/// Create a prompt builder for fluent configuration.
///
/// # Example
///
/// ```rust,ignore
/// use agent_harness::prompt;
///
/// let tool_calls = prompt("List all files")
///     .in_dir("./my-project")
///     .run()
///     .unwrap();
/// ```
pub fn prompt(text: &str) -> PromptBuilder {
    PromptBuilder::new(text)
}

/// Builder for configuring and executing prompts.
///
/// The builder provides a fluent interface for setting up prompt execution
/// with various options like working directory, agent type, and output display.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    text: String,
    working_dir: Option<PathBuf>,
    agent: Option<AgentType>,
    output_config: Option<OutputConfig>,
}

impl PromptBuilder {
    /// Create a new prompt builder with the given prompt text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            working_dir: None,
            agent: None,
            output_config: None,
        }
    }

    /// Set the working directory for execution.
    ///
    /// The agent will execute the prompt as if it were run from this directory.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tool_calls = prompt("List files")
    ///     .in_dir("/path/to/project")
    ///     .run()
    ///     .unwrap();
    /// ```
    pub fn in_dir(mut self, dir: &str) -> Self {
        self.working_dir = Some(PathBuf::from(dir));
        self
    }

    /// Set the working directory using a PathBuf.
    pub fn in_dir_path(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Set the agent to use (default: Claude).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use agent_harness::{prompt, AgentType};
    ///
    /// let tool_calls = prompt("Hello")
    ///     .agent(AgentType::Claude)
    ///     .run()
    ///     .unwrap();
    /// ```
    pub fn agent(mut self, agent: AgentType) -> Self {
        self.agent = Some(agent);
        self
    }

    /// Configure output display.
    ///
    /// When set, tool calls and Claude's response will be printed after execution.
    /// This is useful for debugging in Rust tests.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use agent_harness::{prompt, OutputConfig};
    ///
    /// let result = prompt("Read files")
    ///     .with_output(OutputConfig::verbose())
    ///     .run_full()
    ///     .unwrap();
    /// ```
    pub fn with_output(mut self, config: OutputConfig) -> Self {
        self.output_config = Some(config);
        self
    }

    /// Execute the prompt and return the full result including stdout.
    ///
    /// If output configuration is set via `with_output()`, tool calls and
    /// Claude's response will be printed after execution.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use agent_harness::{prompt, OutputConfig};
    ///
    /// let result = prompt("Read config.json")
    ///     .with_output(OutputConfig::verbose())
    ///     .run_full()
    ///     .unwrap();
    ///
    /// // Access tool calls and stdout
    /// println!("Made {} tool calls", result.tool_calls.len());
    /// if let Some(stdout) = &result.stdout {
    ///     println!("Response: {}", stdout);
    /// }
    /// ```
    pub fn run_full(self) -> anyhow::Result<NormalizedResult> {
        let harness = AgentHarness::new();
        let mut config = ExecutionConfig::new();

        if let Some(dir) = self.working_dir {
            config = config.with_working_dir(dir);
        }

        let result = harness.execute(self.agent, &self.text, config)?;

        // Print output if configured
        if let Some(output_config) = self.output_config {
            let formatter = OutputFormatter::new(output_config);
            // For Rust tests, we print immediately (can't know pass/fail yet)
            formatter.print_tool_calls(&result.tool_calls, true);
            formatter.print_response(result.stdout.as_deref(), true);
        }

        Ok(result)
    }

    /// Execute the prompt and return tool calls.
    ///
    /// This runs the configured agent with the prompt and collects all tool
    /// calls made during execution.
    ///
    /// If output configuration is set via `with_output()`, tool calls and
    /// Claude's response will be printed after execution.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent is not available on the system
    /// - The agent execution fails
    /// - The session log cannot be parsed
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tool_calls = prompt("Read config.json").run().unwrap();
    /// assert!(!tool_calls.is_empty());
    /// ```
    pub fn run(self) -> anyhow::Result<Vec<ToolCall>> {
        Ok(self.run_full()?.tool_calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder_creation() {
        let builder = prompt("Test prompt");
        assert_eq!(builder.text, "Test prompt");
        assert!(builder.working_dir.is_none());
        assert!(builder.agent.is_none());
        assert!(builder.output_config.is_none());
    }

    #[test]
    fn test_prompt_builder_in_dir() {
        let builder = prompt("Test").in_dir("/tmp");
        assert_eq!(builder.working_dir, Some(PathBuf::from("/tmp")));
    }

    #[test]
    fn test_prompt_builder_agent() {
        let builder = prompt("Test").agent(AgentType::Claude);
        assert_eq!(builder.agent, Some(AgentType::Claude));
    }

    #[test]
    fn test_prompt_builder_chaining() {
        let builder = prompt("Test")
            .in_dir("/tmp")
            .agent(AgentType::Claude);

        assert_eq!(builder.text, "Test");
        assert_eq!(builder.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(builder.agent, Some(AgentType::Claude));
    }

    #[test]
    fn test_prompt_builder_with_output() {
        let builder = prompt("Test")
            .with_output(OutputConfig::verbose());

        assert!(builder.output_config.is_some());
    }
}

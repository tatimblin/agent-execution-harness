//! Prompt builder for fluent prompt configuration and execution.
//!
//! This module provides a builder pattern for configuring and executing prompts
//! against AI agents.
//!
//! # Example
//!
//! ```rust,ignore
//! use aptitude::{prompt, expect, Tool};
//!
//! let tool_calls = prompt("Read the config file")
//!     .in_dir("/path/to/project")
//!     .run()
//!     .unwrap();
//!
//! expect(&tool_calls).tool(Tool::Read).to_be_called();
//! ```
//!
//! # Full Output
//!
//! ```rust,ignore
//! use aptitude::{prompt, expect, Tool};
//!
//! let output = prompt("Read the config file")
//!     .run_full()
//!     .unwrap();
//!
//! // Access tool calls
//! expect(&output.result.tool_calls).tool(Tool::Read).to_be_called();
//!
//! // Access debug info if needed
//! if let Some(stdout) = &output.stdout {
//!     println!("Response: {}", stdout);
//! }
//! ```

use crate::agents::{AgentHarness, AgentType, ExecutionConfig, ExecutionOutput};
use crate::parser::ToolCall;
use std::path::PathBuf;

/// Create a prompt builder for fluent configuration.
///
/// # Example
///
/// ```rust,ignore
/// use aptitude::prompt;
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
/// with various options like working directory and agent type.
#[derive(Debug, Clone)]
pub struct PromptBuilder {
    text: String,
    working_dir: Option<PathBuf>,
    agent: Option<AgentType>,
}

impl PromptBuilder {
    /// Create a new prompt builder with the given prompt text.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            working_dir: None,
            agent: None,
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
    /// use aptitude::{prompt, AgentType};
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

    /// Execute the prompt and return the full execution output.
    ///
    /// Returns [`ExecutionOutput`] containing both the normalized result
    /// (tool calls) and debug info (stdout, session log path).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use aptitude::prompt;
    ///
    /// let output = prompt("Read config.json")
    ///     .run_full()
    ///     .unwrap();
    ///
    /// // Access tool calls
    /// println!("Made {} tool calls", output.result.tool_calls.len());
    ///
    /// // Access debug info if needed
    /// if let Some(stdout) = &output.stdout {
    ///     println!("Response: {}", stdout);
    /// }
    /// ```
    pub fn run_full(self) -> anyhow::Result<ExecutionOutput> {
        let harness = AgentHarness::new();
        let mut config = ExecutionConfig::new();

        if let Some(dir) = self.working_dir {
            config = config.with_working_dir(dir);
        }

        harness.execute(self.agent, &self.text, config)
    }

    /// Execute the prompt and return tool calls.
    ///
    /// This is a convenience method that extracts just the tool calls
    /// from the execution result. Use [`run_full`](Self::run_full) if you
    /// need access to stdout or other debug info.
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
        Ok(self.run_full()?.result.tool_calls)
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
}

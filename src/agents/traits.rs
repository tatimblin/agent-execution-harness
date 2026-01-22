//! Core traits and types for the agent abstraction layer.

use anyhow::Result;
use std::path::PathBuf;

use super::mapping::ToolNameMapping;
use crate::parser::ToolCall;

/// Configuration for agent execution.
#[derive(Debug, Clone, Default)]
pub struct ExecutionConfig {
    /// Working directory for agent execution.
    pub working_dir: Option<PathBuf>,
    /// Extra arguments to pass to the agent CLI.
    pub extra_args: Vec<String>,
}

impl ExecutionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }
}

/// Raw result from agent execution before normalization.
#[derive(Debug)]
pub struct RawExecutionResult {
    /// Path to the session log file, if the agent produces one.
    pub session_log_path: Option<PathBuf>,
    /// Stdout from the agent command.
    pub stdout: Option<String>,
}

/// The core trait that all agent adapters must implement.
///
/// Each agent (Claude Code, Aider, Cursor, etc.) implements this trait
/// to provide a unified interface for execution and parsing.
pub trait Agent: Send + Sync {
    /// Unique identifier for this agent (e.g., "claude", "aider", "cursor").
    fn name(&self) -> &'static str;

    /// Execute the agent with a prompt.
    ///
    /// Returns a raw execution result that can be parsed for tool calls.
    fn execute(&self, prompt: &str, config: &ExecutionConfig) -> Result<RawExecutionResult>;

    /// Parse the agent's output/log format and extract tool calls.
    ///
    /// Returns tool calls in the agent's native naming convention.
    fn parse_session(&self, result: &RawExecutionResult) -> Result<Vec<ToolCall>>;

    /// Return the tool name mapping for this agent.
    ///
    /// Maps agent-specific tool names to canonical names.
    fn tool_mapping(&self) -> &ToolNameMapping;

    /// Check if this agent is available on the system.
    fn is_available(&self) -> bool;
}

//! Agent harness facade for unified agent operations.
//!
//! This module provides the main entry point for executing agents
//! and normalizing their results.

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::Arc;

use crate::parser::ToolCall;
use super::claude::ClaudeAdapter;
use super::mapping::ToolNameMapping;
use super::traits::{Agent, ExecutionConfig};

/// Supported agent types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AgentType {
    #[default]
    Claude,
    // Future agents:
    // Aider,
    // Cursor,
}

impl AgentType {
    /// Parse an agent type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "claude-code" => Some(AgentType::Claude),
            // "aider" => Some(AgentType::Aider),
            // "cursor" => Some(AgentType::Cursor),
            _ => None,
        }
    }

    /// Get the string name for this agent type.
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Claude => "claude",
            // AgentType::Aider => "aider",
            // AgentType::Cursor => "cursor",
        }
    }
}

/// Normalized execution result with canonical tool names.
#[derive(Debug)]
pub struct NormalizedResult {
    /// Tool calls with canonical names.
    pub tool_calls: Vec<ToolCall>,
    /// Name of the agent that was executed.
    pub agent_name: String,
}

/// The main facade for agent operations.
///
/// This struct provides a unified interface for executing different
/// coding agents and normalizing their results.
pub struct AgentHarness {
    agents: HashMap<AgentType, Arc<dyn Agent>>,
    default_agent: AgentType,
}

impl AgentHarness {
    /// Create a new agent harness with default agents registered.
    pub fn new() -> Self {
        let mut agents: HashMap<AgentType, Arc<dyn Agent>> = HashMap::new();

        // Register built-in agents
        agents.insert(AgentType::Claude, Arc::new(ClaudeAdapter::new()));

        Self {
            agents,
            default_agent: AgentType::Claude,
        }
    }

    /// Execute an agent and return normalized results.
    ///
    /// Tool calls are automatically converted to canonical names.
    pub fn execute(
        &self,
        agent_type: Option<AgentType>,
        prompt: &str,
        config: ExecutionConfig,
    ) -> Result<NormalizedResult> {
        let agent_type = agent_type.unwrap_or(self.default_agent);

        let agent = self
            .agents
            .get(&agent_type)
            .ok_or_else(|| anyhow::anyhow!("Agent not registered: {:?}", agent_type))?;

        if !agent.is_available() {
            bail!(
                "Agent '{}' is not available on this system",
                agent.name()
            );
        }

        // Execute the agent
        let raw_result = agent.execute(prompt, &config)?;

        // Parse tool calls
        let raw_tool_calls = agent.parse_session(&raw_result)?;

        // Normalize tool names to canonical form
        let normalized_calls = self.normalize_tool_calls(&raw_tool_calls, agent.tool_mapping());

        Ok(NormalizedResult {
            tool_calls: normalized_calls,
            agent_name: agent.name().to_string(),
        })
    }

    /// Normalize tool calls from agent-specific to canonical names.
    fn normalize_tool_calls(
        &self,
        calls: &[ToolCall],
        mapping: &ToolNameMapping,
    ) -> Vec<ToolCall> {
        calls
            .iter()
            .map(|call| ToolCall {
                name: mapping.to_canonical(&call.name),
                params: call.params.clone(),
                timestamp: call.timestamp,
            })
            .collect()
    }

    /// Get an agent by type.
    pub fn get_agent(&self, agent_type: AgentType) -> Option<&Arc<dyn Agent>> {
        self.agents.get(&agent_type)
    }

    /// List all registered agents.
    pub fn registered_agents(&self) -> Vec<&'static str> {
        self.agents.values().map(|a| a.name()).collect()
    }
}

impl Default for AgentHarness {
    fn default() -> Self {
        Self::new()
    }
}

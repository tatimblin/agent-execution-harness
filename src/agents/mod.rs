//! Agent abstraction layer for multi-agent support.
//!
//! This module provides a unified interface for different coding agents
//! (Claude Code, Aider, Cursor, etc.) using the Adapter and Facade patterns.
//!
//! # Architecture
//!
//! - [`Agent`] trait: Defines the interface all agent adapters must implement
//! - [`ToolNameMapping`]: Handles conversion between agent-specific and canonical tool names
//! - [`AgentHarness`]: Facade providing unified access to all agents
//!
//! # Example
//!
//! ```ignore
//! use agent_harness::{AgentHarness, AgentType, ExecutionConfig};
//!
//! let harness = AgentHarness::new();
//! let config = ExecutionConfig::new();
//! let output = harness.execute(Some(AgentType::Claude), "Hello", config)?;
//!
//! for call in &output.result.tool_calls {
//!     println!("Tool: {}", call.name);
//! }
//! ```

mod claude;
mod harness;
pub mod mapping;
mod traits;

pub use harness::{AgentHarness, AgentType, ExecutionOutput, NormalizedResult};
pub use mapping::ToolNameMapping;
pub use traits::{Agent, ExecutionConfig, RawExecutionResult};

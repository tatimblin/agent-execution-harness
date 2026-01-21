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
//! use crate::agents::{AgentHarness, AgentType, ExecutionConfig};
//!
//! let harness = AgentHarness::new();
//! let config = ExecutionConfig::new();
//! let result = harness.execute(Some(AgentType::Claude), "Hello", config)?;
//!
//! for call in result.tool_calls {
//!     println!("Tool: {} (canonical)", call.name);
//! }
//! ```

mod claude;
mod harness;
mod mapping;
mod traits;

pub use harness::{AgentHarness, AgentType};
pub use traits::ExecutionConfig;

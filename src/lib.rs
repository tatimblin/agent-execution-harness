//! # agent_harness
//!
//! A fluent assertion library for testing AI agent tool calls.
//!
//! This library provides a Jest-like API for asserting on tool calls made by AI coding agents.
//! It can be used with Rust's native `#[test]` framework.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use agent_harness::{prompt, expect, Tool};
//!
//! #[test]
//! fn test_agent_behavior() {
//!     let tool_calls = prompt("Read the config file").run().unwrap();
//!
//!     expect(&tool_calls)
//!         .tool(Tool::Read)
//!         .to_be_called();
//!
//!     expect(&tool_calls)
//!         .tool(Tool::Bash)
//!         .not_to_be_called();
//! }
//! ```
//!
//! ## With Working Directory
//!
//! ```rust,ignore
//! use agent_harness::{prompt, expect, Tool};
//!
//! #[test]
//! fn test_in_project_dir() {
//!     let tool_calls = prompt("List files here")
//!         .in_dir("/path/to/project")
//!         .run()
//!         .unwrap();
//!
//!     expect(&tool_calls)
//!         .tool(Tool::Bash)
//!         .to_be_called();
//! }
//! ```
//!
//! ## Analyzing Existing Sessions
//!
//! ```rust,ignore
//! use agent_harness::{parse_session, expect, Tool};
//!
//! #[test]
//! fn test_existing_session() {
//!     let tool_calls = parse_session("session.jsonl").unwrap();
//!
//!     expect(&tool_calls)
//!         .tool(Tool::Write)
//!         .after(Tool::Read);
//! }
//! ```

pub mod agents;
pub mod fluent;
pub mod output;
pub mod parser;
pub mod prompt;

#[cfg(feature = "yaml")]
pub mod yaml;

// Core types
pub use fluent::{expect, params_match, ToolCallExpectation, ToolAssertion};
pub use parser::{parse_jsonl_file as parse_session, ToolCall};

// Tool enum
pub use fluent::Tool;

// Agent execution
pub use agents::{AgentHarness, AgentType, ExecutionConfig, ExecutionOutput, NormalizedResult};

// Prompt builder
pub use prompt::{prompt, PromptBuilder};

// Output formatting
pub use output::{OutputConfig, OutputFormatter, OutputMode};

// YAML (feature-gated)
#[cfg(feature = "yaml")]
pub use yaml::{load_test, run_yaml_test, Assertion, Test as YamlTest};

//! Fluent assertion API for testing AI agent execution output.
//!
//! This module provides a Jest-like API for making assertions about tool calls
//! and stdout output. Assertions evaluate immediately (panic on failure) when
//! using methods like `to_be_called()`, or can be evaluated non-destructively
//! using `evaluate()`.
//!
//! # Example
//!
//! ```rust,ignore
//! use aptitude::{expect, expect_tools, Tool};
//!
//! let output = harness.execute(...)?;
//!
//! // Tool assertions
//! expect(&output)
//!     .tool(Tool::Read)
//!     .to_be_called();
//!
//! // Stdout assertions
//! expect(&output)
//!     .stdout()
//!     .contains("success")
//!     .to_exist();
//!
//! // For backward compatibility (tool calls only)
//! let tool_calls = parse_session("session.jsonl")?;
//! expect_tools(&tool_calls)
//!     .tool(Tool::Read)
//!     .evaluate();
//! ```

mod builder;
mod matchers;
mod stdout;
mod tool;

pub use builder::{
    expect, expect_tools, AssertionResult, ExecutionExpectation, NthCallAssertion, ToolAssertion,
    ToolCallExpectation,
};
pub use matchers::params_match;
pub use stdout::StdoutAssertion;
pub use tool::Tool;

#[cfg(test)]
mod tests;

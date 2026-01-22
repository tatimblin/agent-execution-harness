//! Fluent assertion API for testing AI agent tool calls.
//!
//! This module provides a Jest-like API for making assertions about tool calls.
//! Assertions evaluate immediately (panic on failure) when using methods like
//! `to_be_called()`, or can be evaluated non-destructively using `evaluate()`.
//!
//! # Example
//!
//! ```rust,ignore
//! use aptitude::{expect, Tool};
//!
//! let tool_calls = vec![/* ... */];
//!
//! // Immediate evaluation (panics on failure)
//! expect(&tool_calls)
//!     .tool(Tool::Read)
//!     .to_be_called();
//!
//! // Non-panicking evaluation
//! let result = expect(&tool_calls)
//!     .tool(Tool::Read)
//!     .evaluate();
//! assert!(result.passed);
//! ```

mod builder;
mod matchers;
mod tool;

pub use builder::{expect, AssertionResult, NthCallAssertion, ToolAssertion, ToolCallExpectation};
pub use matchers::params_match;
pub use tool::Tool;

#[cfg(test)]
mod tests;

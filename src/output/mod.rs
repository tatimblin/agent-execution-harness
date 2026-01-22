//! Output formatting for test results, tool calls, and agent responses.
//!
//! This module provides configurable output display for the test harness,
//! with support for showing tool calls and Claude's responses either always,
//! on failure, or never.
//!
//! # Example
//!
//! ```rust,ignore
//! use agent_harness::output::{OutputConfig, OutputFormatter, OutputMode};
//!
//! let config = OutputConfig::new()
//!     .tool_calls(OutputMode::Always)
//!     .response(OutputMode::OnFailure);
//!
//! let formatter = OutputFormatter::new(config);
//! formatter.print_tool_calls(&tool_calls, test_passed);
//! ```

mod config;
mod formatter;

pub use config::{OutputConfig, OutputMode};
pub use formatter::OutputFormatter;

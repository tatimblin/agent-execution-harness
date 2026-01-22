//! YAML test file support for the agent harness.
//!
//! This module provides functionality for loading and running tests defined in YAML files.
//! It acts as a thin layer on top of the fluent API, handling string parsing and YAML
//! deserialization.
//!
//! # Test File Format
//!
//! ```yaml
//! name: "Test file reading"
//! prompt: "Read the config file"
//! assertions:
//!   - tool: Read           # Tool name (case-insensitive)
//!     called: true
//!     params:
//!       file_path: "*config*"
//!   - tool: Bash
//!     called: false
//!   - tool: Write
//!     called_after: Read
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use aptitude::{load_test, run_yaml_test};
//!
//! let test = load_test("test.yaml").unwrap();
//! let results = run_yaml_test(&test, &tool_calls);
//! ```

mod parser;
mod runner;

pub use parser::{load_test, parse_tool_name, Assertion, Test, YamlError};
pub use runner::{run_yaml_test, TestResult};

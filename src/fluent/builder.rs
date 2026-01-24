//! Fluent assertion builder for execution output.
//!
//! This module provides the core builder types for making assertions about execution:
//! - `expect()` - Entry point for creating assertions from ExecutionOutput
//! - `expect_tools()` - Entry point for creating assertions from tool calls only
//! - `ExecutionExpectation` - Holds execution output and creates specific assertions
//! - `ToolAssertion` - Builder for assertions on a specific tool

use crate::agents::ExecutionOutput;
use crate::parser::ToolCall;
use super::matchers::params_match;
use super::stdout::StdoutAssertion;
use super::Tool;
use std::collections::HashMap;

/// Result of evaluating an assertion.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// Whether the assertion passed.
    pub passed: bool,
    /// Description of what was asserted.
    pub description: String,
    /// Failure reason if the assertion failed.
    pub reason: Option<String>,
}

impl AssertionResult {
    /// Create a passing assertion result.
    pub(crate) fn pass(description: impl Into<String>) -> Self {
        Self {
            passed: true,
            description: description.into(),
            reason: None,
        }
    }

    /// Create a failing assertion result.
    pub(crate) fn fail(description: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            passed: false,
            description: description.into(),
            reason: Some(reason.into()),
        }
    }
}

/// Create an expectation on execution output.
///
/// This is the entry point for the fluent assertion API.
///
/// # Example
///
/// ```rust,ignore
/// use aptitude::{expect, Tool};
///
/// let output = harness.execute(...)?;
/// expect(&output).tool(Tool::Read).to_be_called();
/// expect(&output).stdout().contains("done").to_exist();
/// ```
pub fn expect(output: &ExecutionOutput) -> ExecutionExpectation {
    ExecutionExpectation::new(output)
}

/// Create an expectation on tool calls only (for backward compatibility).
///
/// Use this when you only have tool calls (e.g., from log analysis).
///
/// # Example
///
/// ```rust,ignore
/// use aptitude::{expect_tools, Tool};
///
/// let tool_calls = parse_session("session.jsonl")?;
/// expect_tools(&tool_calls).tool(Tool::Read).to_be_called();
/// ```
pub fn expect_tools(tool_calls: &[ToolCall]) -> ExecutionExpectation {
    ExecutionExpectation::from_tool_calls(tool_calls)
}

/// Holds execution output and creates specific assertions.
///
/// This is the starting point for building assertions. Call `.tool()` to
/// create a `ToolAssertion` for a specific tool type, or `.stdout()` to
/// create a `StdoutAssertion` for stdout content.
#[derive(Debug, Clone)]
pub struct ExecutionExpectation {
    tool_calls: Vec<ToolCall>,
    stdout: Option<String>,
}

impl ExecutionExpectation {
    /// Create a new expectation from execution output.
    pub fn new(output: &ExecutionOutput) -> Self {
        Self {
            tool_calls: output.result.tool_calls.clone(),
            stdout: output.stdout.clone(),
        }
    }

    /// Create from just tool calls (for backward compatibility / analysis mode).
    pub fn from_tool_calls(tool_calls: &[ToolCall]) -> Self {
        Self {
            tool_calls: tool_calls.to_vec(),
            stdout: None,
        }
    }

    /// Create an assertion for a specific tool.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .tool(Tool::Read)
    ///     .to_be_called();
    /// ```
    pub fn tool(&self, tool: Tool) -> ToolAssertion {
        ToolAssertion::new(self.tool_calls.clone(), tool)
    }

    /// Create an assertion for stdout content.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .contains("success")
    ///     .to_exist();
    /// ```
    pub fn stdout(&self) -> StdoutAssertion {
        StdoutAssertion::new(self.stdout.clone())
    }
}

// Backward compatibility: keep ToolCallExpectation as an alias
#[doc(hidden)]
pub type ToolCallExpectation = ExecutionExpectation;

/// Builder for assertions on a specific tool.
///
/// Methods like `to_be_called()` evaluate immediately and panic on failure.
/// Use `evaluate()` for non-panicking evaluation.
#[derive(Debug, Clone)]
pub struct ToolAssertion {
    tool_calls: Vec<ToolCall>,
    tool: Tool,
    params: Option<HashMap<String, String>>,
    expected_count: Option<usize>,
    min_count: Option<usize>,
    max_count: Option<usize>,
    after_tool: Option<Tool>,
    before_tool: Option<Tool>,
}

impl ToolAssertion {
    /// Create a new tool assertion.
    pub fn new(tool_calls: Vec<ToolCall>, tool: Tool) -> Self {
        Self {
            tool_calls,
            tool,
            params: None,
            expected_count: None,
            min_count: None,
            max_count: None,
            after_tool: None,
            before_tool: None,
        }
    }

    // =========================================================================
    // Builder methods (chainable)
    // =========================================================================

    /// Set parameter expectations for matching.
    ///
    /// Parameters use regex matching. Use `.*` for wildcards, escape special chars with `\`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use aptitude::params;
    ///
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .with_params(params!{"file_path" => r".*\.txt"})
    ///     .to_be_called();
    /// ```
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.params = Some(params);
        self
    }

    /// Assert the tool was called exactly N times.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .times(3)
    ///     .to_be_called();
    /// ```
    pub fn times(mut self, n: usize) -> Self {
        self.expected_count = Some(n);
        self
    }

    /// Assert the tool was called at least N times.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .at_least(2)
    ///     .to_be_called();
    /// ```
    pub fn at_least(mut self, n: usize) -> Self {
        self.min_count = Some(n);
        self
    }

    /// Assert the tool was called at most N times.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .at_most(5)
    ///     .to_be_called();
    /// ```
    pub fn at_most(mut self, n: usize) -> Self {
        self.max_count = Some(n);
        self
    }

    /// Assert this tool was called after another tool.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Write)
    ///     .after(Tool::Read)
    ///     .to_be_called();
    /// ```
    pub fn after(mut self, tool: Tool) -> Self {
        self.after_tool = Some(tool);
        self
    }

    /// Assert this tool was called before another tool.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .before(Tool::Write)
    ///     .to_be_called();
    /// ```
    pub fn before(mut self, tool: Tool) -> Self {
        self.before_tool = Some(tool);
        self
    }

    // =========================================================================
    // Assertion methods (panic on failure)
    // =========================================================================

    /// Assert the tool was called.
    ///
    /// Panics with a detailed error message if the assertion fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .to_be_called();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the tool was not called (or doesn't match params/ordering).
    pub fn to_be_called(&self) {
        let result = self.evaluate_called(true);
        if !result.passed {
            self.panic_with_context(&result);
        }
    }

    /// Assert the tool was NOT called.
    ///
    /// Panics with a detailed error message if the tool was called.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Bash)
    ///     .not_to_be_called();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the tool was called (matching any specified params).
    pub fn not_to_be_called(&self) {
        let result = self.evaluate_called(false);
        if !result.passed {
            self.panic_with_context(&result);
        }
    }

    // =========================================================================
    // nth_call pattern
    // =========================================================================

    /// Get the nth call (1-indexed) of this tool for further assertions.
    ///
    /// Returns a `NthCallAssertion` builder for making assertions about
    /// that specific call.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use aptitude::params;
    ///
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .nth_call(1)
    ///     .has_params(params!{"file_path" => "/first.txt"});
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the nth call doesn't exist.
    pub fn nth_call(&self, n: usize) -> NthCallAssertion {
        let matching_calls: Vec<&ToolCall> = self
            .tool_calls
            .iter()
            .filter(|c| c.name == self.tool.as_str())
            .collect();

        if n == 0 || n > matching_calls.len() {
            panic!(
                "assertion failed: expected {} call #{} to exist\n\n  actual: {} calls made\n{}",
                self.tool,
                n,
                matching_calls.len(),
                self.format_tool_calls()
            );
        }

        let call = matching_calls[n - 1];
        NthCallAssertion::new(call.clone(), self.tool, n, self.tool_calls.clone())
    }

    /// Get the last call of this tool for further assertions.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .last_call()
    ///     .has_params(params!{"file_path" => "/last.txt"});
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the tool was never called.
    pub fn last_call(&self) -> NthCallAssertion {
        let matching_calls: Vec<&ToolCall> = self
            .tool_calls
            .iter()
            .filter(|c| c.name == self.tool.as_str())
            .collect();

        if matching_calls.is_empty() {
            panic!(
                "assertion failed: expected {} to have been called\n\n  actual: 0 calls made\n{}",
                self.tool,
                self.format_tool_calls()
            );
        }

        let n = matching_calls.len();
        let call = matching_calls[n - 1];
        NthCallAssertion::new(call.clone(), self.tool, n, self.tool_calls.clone())
    }

    // =========================================================================
    // Non-panicking evaluation
    // =========================================================================

    /// Evaluate the assertion without panicking (expects tool to be called).
    ///
    /// Returns an `AssertionResult` that can be inspected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .evaluate();
    ///
    /// if !result.passed {
    ///     println!("Failed: {}", result.reason.unwrap());
    /// }
    /// ```
    pub fn evaluate(&self) -> AssertionResult {
        self.evaluate_called(true)
    }

    /// Evaluate that the tool was NOT called, without panicking.
    ///
    /// Returns an `AssertionResult` that can be inspected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = expect(&tool_calls)
    ///     .tool(Tool::Bash)
    ///     .evaluate_not_called();
    ///
    /// assert!(result.passed);
    /// ```
    pub fn evaluate_not_called(&self) -> AssertionResult {
        self.evaluate_called(false)
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn get_matching_calls(&self) -> Vec<&ToolCall> {
        self.tool_calls
            .iter()
            .filter(|c| c.name == self.tool.as_str())
            .filter(|c| {
                if let Some(params) = &self.params {
                    params_match(params, &c.params)
                } else {
                    true
                }
            })
            .collect()
    }

    fn evaluate_called(&self, should_be_called: bool) -> AssertionResult {
        let matching_calls = self.get_matching_calls();
        let count = matching_calls.len();
        let was_called = count > 0;

        // Collect all failures - check ALL constraints
        let mut failures: Vec<String> = Vec::new();

        // Check called/not called first (fundamental constraint)
        if should_be_called && !was_called {
            let param_desc = self
                .params
                .as_ref()
                .map(|p| format!(" with params {:?}", p))
                .unwrap_or_default();
            failures.push(format!("tool '{}'{} was never called", self.tool, param_desc));
        } else if !should_be_called && was_called {
            let found = matching_calls.first().unwrap();
            failures.push(format!(
                "tool '{}' was called but should not have been. Found: {:?}",
                self.tool, found.params
            ));
        }

        // Check count constraints (always check if constraint is set)
        if should_be_called {
            if let Some(expected) = self.expected_count {
                if count != expected {
                    failures.push(format!("expected {} calls, got {}", expected, count));
                }
            }
            if let Some(min) = self.min_count {
                if count < min {
                    failures.push(format!("expected at least {} calls, got {}", min, count));
                }
            }
            if let Some(max) = self.max_count {
                if count > max {
                    failures.push(format!("expected at most {} calls, got {}", max, count));
                }
            }
        }

        // Check ordering constraints
        if let Some(after) = &self.after_tool {
            if let Some(err) = self.check_after(after) {
                failures.push(err);
            }
        }
        if let Some(before) = &self.before_tool {
            if let Some(err) = self.check_before(before) {
                failures.push(err);
            }
        }

        // Build description
        let description = self.build_description(should_be_called);

        if failures.is_empty() {
            AssertionResult::pass(description)
        } else {
            AssertionResult::fail(description, failures.join("; "))
        }
    }

    /// Build a human-readable description of what this assertion checks.
    fn build_description(&self, should_be_called: bool) -> String {
        let mut parts = vec![self.tool.to_string()];

        if let Some(params) = &self.params {
            let param_str: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}='{}'", k, v))
                .collect();
            parts.push(format!("with {}", param_str.join(", ")));
        }

        if should_be_called {
            parts.push("called".to_string());
        } else {
            parts.push("not called".to_string());
        }

        if let Some(after) = &self.after_tool {
            parts.push(format!("after {}", after));
        }
        if let Some(before) = &self.before_tool {
            parts.push(format!("before {}", before));
        }
        if let Some(n) = self.expected_count {
            parts.push(format!("{} times", n));
        }
        if let Some(n) = self.min_count {
            parts.push(format!("at least {} times", n));
        }
        if let Some(n) = self.max_count {
            parts.push(format!("at most {} times", n));
        }

        parts.join(" ")
    }

    /// Check if tool was called after another tool. Returns error message if failed.
    fn check_after(&self, after_tool: &Tool) -> Option<String> {
        let mut seen_after = false;

        for call in &self.tool_calls {
            if call.name == after_tool.as_str() {
                seen_after = true;
            }
            if call.name == self.tool.as_str() && seen_after {
                if let Some(params) = &self.params {
                    if params_match(params, &call.params) {
                        return None; // Success
                    }
                } else {
                    return None; // Success
                }
            }
        }

        if !seen_after {
            Some(format!("'{}' was never called", after_tool))
        } else {
            Some(format!("'{}' was not called after '{}'", self.tool, after_tool))
        }
    }

    /// Check if tool was called before another tool. Returns error message if failed.
    fn check_before(&self, before_tool: &Tool) -> Option<String> {
        let mut seen_this = false;

        for call in &self.tool_calls {
            if call.name == self.tool.as_str() {
                if let Some(params) = &self.params {
                    if params_match(params, &call.params) {
                        seen_this = true;
                    }
                } else {
                    seen_this = true;
                }
            }
            if call.name == before_tool.as_str() && seen_this {
                return None; // Success
            }
        }

        let this_called = self.tool_calls.iter().any(|c| c.name == self.tool.as_str());
        let before_called = self
            .tool_calls
            .iter()
            .any(|c| c.name == before_tool.as_str());

        if !this_called {
            Some(format!("'{}' was never called", self.tool))
        } else if !before_called {
            Some(format!("'{}' was never called", before_tool))
        } else {
            Some(format!("'{}' was not called before '{}'", self.tool, before_tool))
        }
    }

    fn panic_with_context(&self, result: &AssertionResult) -> ! {
        let reason = result.reason.as_deref().unwrap_or("unknown reason");
        panic!(
            "assertion failed: expected {}\n\n  reason: {}\n{}",
            result.description,
            reason,
            self.format_tool_calls()
        );
    }

    fn format_tool_calls(&self) -> String {
        if self.tool_calls.is_empty() {
            return "  tool calls made: (none)\n".to_string();
        }

        let mut output = format!("  tool calls made ({}):\n", self.tool_calls.len());
        for (i, call) in self.tool_calls.iter().enumerate() {
            let params_preview = call
                .params
                .get("file_path")
                .or_else(|| call.params.get("command"))
                .or_else(|| call.params.get("pattern"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    if s.len() > 50 {
                        format!("{}...", &s[..47])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| "...".to_string());
            output.push_str(&format!(
                "    {}. {} {{ {} }}\n",
                i + 1,
                call.name,
                params_preview
            ));
        }
        output
    }
}

/// Assertion builder for a specific call (used in nth_call/last_call).
///
/// Provides methods to assert on parameter values for a specific tool call.
#[derive(Debug, Clone)]
pub struct NthCallAssertion {
    call: ToolCall,
    tool: Tool,
    n: usize,
    all_calls: Vec<ToolCall>,
}

impl NthCallAssertion {
    fn new(call: ToolCall, tool: Tool, n: usize, all_calls: Vec<ToolCall>) -> Self {
        Self { call, tool, n, all_calls }
    }

    /// Assert this specific call has the given parameters (panics on mismatch).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .nth_call(1)
    ///     .has_params(params!{"file_path" => "*.txt"});
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the parameters don't match.
    pub fn has_params(self, params: HashMap<String, String>) -> Self {
        if !params_match(&params, &self.call.params) {
            panic!(
                "assertion failed: {} call #{} params did not match\n\n  expected: {:?}\n  actual: {:?}\n{}",
                self.tool, self.n, params, self.call.params, self.format_tool_calls()
            );
        }
        self
    }

    /// Evaluate parameter match without panicking.
    ///
    /// Returns an `AssertionResult` that can be inspected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = expect(&tool_calls)
    ///     .tool(Tool::Read)
    ///     .nth_call(1)
    ///     .evaluate_params(params!{"file_path" => "*.txt"});
    ///
    /// assert!(result.passed);
    /// ```
    pub fn evaluate_params(&self, params: HashMap<String, String>) -> AssertionResult {
        if params_match(&params, &self.call.params) {
            AssertionResult::pass(format!("{} call #{} params match", self.tool, self.n))
        } else {
            AssertionResult::fail(
                format!("{} call #{} params match", self.tool, self.n),
                format!("expected {:?}, got {:?}", params, self.call.params),
            )
        }
    }

    /// Get the actual parameters of this call.
    ///
    /// Useful for debugging or custom assertions.
    pub fn params(&self) -> &serde_json::Value {
        &self.call.params
    }

    /// Get the call index (1-indexed).
    pub fn index(&self) -> usize {
        self.n
    }

    fn format_tool_calls(&self) -> String {
        if self.all_calls.is_empty() {
            return "  tool calls made: (none)\n".to_string();
        }

        let mut output = format!("  tool calls made ({}):\n", self.all_calls.len());
        for (i, call) in self.all_calls.iter().enumerate() {
            let params_preview = call
                .params
                .get("file_path")
                .or_else(|| call.params.get("command"))
                .or_else(|| call.params.get("pattern"))
                .and_then(|v| v.as_str())
                .map(|s| {
                    if s.len() > 50 {
                        format!("{}...", &s[..47])
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or_else(|| "...".to_string());
            output.push_str(&format!(
                "    {}. {} {{ {} }}\n",
                i + 1,
                call.name,
                params_preview
            ));
        }
        output
    }
}

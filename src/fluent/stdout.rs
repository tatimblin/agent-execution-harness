//! Fluent assertion builder for stdout/output.
//!
//! This module provides the builder type for making assertions about stdout:
//! - `StdoutAssertion` - Builder for assertions on stdout content

use regex::Regex;
use super::builder::AssertionResult;

/// Builder for assertions on stdout.
///
/// Methods like `to_exist()` evaluate immediately and panic on failure.
/// Use `evaluate()` for non-panicking evaluation.
#[derive(Debug, Clone)]
pub struct StdoutAssertion {
    stdout: Option<String>,
    contains: Vec<String>,
    not_contains: Vec<String>,
    matches: Vec<String>,
    not_matches: Vec<String>,
}

impl StdoutAssertion {
    /// Create a new stdout assertion.
    pub fn new(stdout: Option<String>) -> Self {
        Self {
            stdout,
            contains: Vec::new(),
            not_contains: Vec::new(),
            matches: Vec::new(),
            not_matches: Vec::new(),
        }
    }

    // =========================================================================
    // Builder methods (chainable)
    // =========================================================================

    /// Assert stdout contains the given substring.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .contains("success")
    ///     .to_exist();
    /// ```
    pub fn contains(mut self, s: &str) -> Self {
        self.contains.push(s.to_string());
        self
    }

    /// Assert stdout does NOT contain the given substring.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .not_contains("error")
    ///     .to_exist();
    /// ```
    pub fn not_contains(mut self, s: &str) -> Self {
        self.not_contains.push(s.to_string());
        self
    }

    /// Assert stdout matches the given regex pattern.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .matches(r"Success: \d+ items")
    ///     .to_exist();
    /// ```
    pub fn matches(mut self, pattern: &str) -> Self {
        self.matches.push(pattern.to_string());
        self
    }

    /// Assert stdout does NOT match the given regex pattern.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .not_matches(r"error|fail")
    ///     .to_exist();
    /// ```
    pub fn not_matches(mut self, pattern: &str) -> Self {
        self.not_matches.push(pattern.to_string());
        self
    }

    // =========================================================================
    // Assertion methods (panic on failure)
    // =========================================================================

    /// Assert stdout exists and matches all constraints.
    ///
    /// Panics with a detailed error message if the assertion fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .contains("done")
    ///     .to_exist();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if stdout is empty/None or doesn't match constraints.
    pub fn to_exist(&self) {
        let result = self.evaluate_exists(true);
        if !result.passed {
            self.panic_with_context(&result);
        }
    }

    /// Assert stdout is empty or None.
    ///
    /// Panics with a detailed error message if stdout exists.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// expect(&output)
    ///     .stdout()
    ///     .to_be_empty();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if stdout exists and is not empty.
    pub fn to_be_empty(&self) {
        let result = self.evaluate_exists(false);
        if !result.passed {
            self.panic_with_context(&result);
        }
    }

    // =========================================================================
    // Non-panicking evaluation
    // =========================================================================

    /// Evaluate the assertion without panicking (expects stdout to exist).
    ///
    /// Returns an `AssertionResult` that can be inspected.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = expect(&output)
    ///     .stdout()
    ///     .contains("done")
    ///     .evaluate();
    ///
    /// if !result.passed {
    ///     println!("Failed: {}", result.reason.unwrap());
    /// }
    /// ```
    pub fn evaluate(&self) -> AssertionResult {
        self.evaluate_exists(true)
    }

    /// Evaluate that stdout is empty, without panicking.
    ///
    /// Returns an `AssertionResult` that can be inspected.
    pub fn evaluate_empty(&self) -> AssertionResult {
        self.evaluate_exists(false)
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    fn evaluate_exists(&self, should_exist: bool) -> AssertionResult {
        let mut failures: Vec<String> = Vec::new();

        let stdout_exists = self.stdout.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

        // Check existence constraint
        if should_exist && !stdout_exists {
            failures.push("stdout was empty or not captured".to_string());
        } else if !should_exist && stdout_exists {
            let preview = self.format_stdout_preview();
            failures.push(format!("stdout should be empty but was: {}", preview));
        }

        // Only check content constraints if stdout exists and we expect it to
        if should_exist && stdout_exists {
            let stdout = self.stdout.as_ref().unwrap();

            // Check contains constraints
            for pattern in &self.contains {
                if !stdout.contains(pattern) {
                    failures.push(format!("stdout does not contain '{}'", pattern));
                }
            }

            // Check not_contains constraints
            for pattern in &self.not_contains {
                if stdout.contains(pattern) {
                    failures.push(format!("stdout contains '{}' but should not", pattern));
                }
            }

            // Check matches constraints
            for pattern in &self.matches {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if !re.is_match(stdout) {
                            failures.push(format!("stdout does not match pattern '{}'", pattern));
                        }
                    }
                    Err(e) => {
                        failures.push(format!("invalid regex '{}': {}", pattern, e));
                    }
                }
            }

            // Check not_matches constraints
            for pattern in &self.not_matches {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if re.is_match(stdout) {
                            failures.push(format!(
                                "stdout matches pattern '{}' but should not",
                                pattern
                            ));
                        }
                    }
                    Err(e) => {
                        failures.push(format!("invalid regex '{}': {}", pattern, e));
                    }
                }
            }
        }

        // Build description
        let description = self.build_description(should_exist);

        if failures.is_empty() {
            AssertionResult::pass(description)
        } else {
            AssertionResult::fail(description, failures.join("; "))
        }
    }

    fn build_description(&self, should_exist: bool) -> String {
        let mut parts = vec!["stdout".to_string()];

        if should_exist {
            parts.push("exists".to_string());
        } else {
            parts.push("is empty".to_string());
        }

        for s in &self.contains {
            parts.push(format!("contains '{}'", s));
        }
        for s in &self.not_contains {
            parts.push(format!("not contains '{}'", s));
        }
        for s in &self.matches {
            parts.push(format!("matches '{}'", s));
        }
        for s in &self.not_matches {
            parts.push(format!("not matches '{}'", s));
        }

        parts.join(", ")
    }

    fn format_stdout_preview(&self) -> String {
        match &self.stdout {
            Some(s) if !s.is_empty() => {
                if s.len() > 100 {
                    format!("\"{}...\"", &s[..97])
                } else {
                    format!("\"{}\"", s)
                }
            }
            _ => "(empty)".to_string(),
        }
    }

    fn panic_with_context(&self, result: &AssertionResult) -> ! {
        let reason = result.reason.as_deref().unwrap_or("unknown reason");
        let preview = self.format_stdout_preview();
        panic!(
            "assertion failed: {}\n\n  reason: {}\n  stdout: {}\n",
            result.description, reason, preview
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_exists() {
        let stdout = Some("hello world".to_string());
        let assertion = StdoutAssertion::new(stdout);
        let result = assertion.evaluate();
        assert!(result.passed);
    }

    #[test]
    fn test_stdout_empty() {
        let stdout: Option<String> = None;
        let assertion = StdoutAssertion::new(stdout);
        let result = assertion.evaluate_empty();
        assert!(result.passed);
    }

    #[test]
    fn test_stdout_contains() {
        let stdout = Some("hello world".to_string());
        let assertion = StdoutAssertion::new(stdout).contains("world");
        let result = assertion.evaluate();
        assert!(result.passed);
    }

    #[test]
    fn test_stdout_contains_fails() {
        let stdout = Some("hello world".to_string());
        let assertion = StdoutAssertion::new(stdout).contains("foo");
        let result = assertion.evaluate();
        assert!(!result.passed);
        assert!(result.reason.unwrap().contains("does not contain"));
    }

    #[test]
    fn test_stdout_not_contains() {
        let stdout = Some("hello world".to_string());
        let assertion = StdoutAssertion::new(stdout).not_contains("error");
        let result = assertion.evaluate();
        assert!(result.passed);
    }

    #[test]
    fn test_stdout_matches() {
        let stdout = Some("Success: 42 items processed".to_string());
        let assertion = StdoutAssertion::new(stdout).matches(r"Success: \d+ items");
        let result = assertion.evaluate();
        assert!(result.passed);
    }

    #[test]
    fn test_stdout_not_matches() {
        let stdout = Some("all good".to_string());
        let assertion = StdoutAssertion::new(stdout).not_matches(r"error|fail");
        let result = assertion.evaluate();
        assert!(result.passed);
    }

    #[test]
    fn test_multiple_constraints() {
        let stdout = Some("Success: 10 items done".to_string());
        let assertion = StdoutAssertion::new(stdout)
            .contains("Success")
            .not_contains("error")
            .matches(r"\d+ items");
        let result = assertion.evaluate();
        assert!(result.passed);
    }
}

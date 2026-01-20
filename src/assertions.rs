use anyhow::{Context, Result};
use glob::Pattern;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::parser::ToolCall;

/// A test loaded from YAML
#[derive(Debug, Deserialize)]
pub struct Test {
    pub name: String,
    pub prompt: String,
    pub assertions: Vec<Assertion>,
}

/// A single assertion about tool usage
#[derive(Debug, Deserialize)]
pub struct Assertion {
    pub tool: String,
    #[serde(default = "default_true")]
    pub called: bool,
    pub params: Option<HashMap<String, String>>,
    pub called_after: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Result of evaluating an assertion
#[derive(Debug)]
pub enum AssertionResult {
    Pass,
    Fail { reason: String },
}

impl AssertionResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, AssertionResult::Pass)
    }
}

/// Load a test from a YAML file
pub fn load_test(path: &Path) -> Result<Test> {
    let content = fs::read_to_string(path).context("Failed to read test file")?;
    let test: Test = serde_yaml::from_str(&content).context("Failed to parse YAML")?;
    Ok(test)
}

/// Evaluate all assertions against collected tool calls
pub fn evaluate_assertions(
    assertions: &[Assertion],
    tool_calls: &[ToolCall],
) -> Vec<(String, AssertionResult)> {
    assertions
        .iter()
        .map(|assertion| {
            let description = format_assertion_description(assertion);
            let result = evaluate_single_assertion(assertion, tool_calls);
            (description, result)
        })
        .collect()
}

fn format_assertion_description(assertion: &Assertion) -> String {
    let mut desc = assertion.tool.clone();

    if let Some(params) = &assertion.params {
        let param_str: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}='{}'", k, v))
            .collect();
        desc = format!("{} with {}", desc, param_str.join(", "));
    }

    if assertion.called {
        if let Some(after) = &assertion.called_after {
            format!("{} called after {}", desc, after)
        } else {
            format!("{} called", desc)
        }
    } else {
        format!("{} not called", desc)
    }
}

fn evaluate_single_assertion(assertion: &Assertion, tool_calls: &[ToolCall]) -> AssertionResult {
    // Find all calls to this tool
    let matching_calls: Vec<&ToolCall> = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .collect();

    // Check params if specified
    let calls_with_matching_params: Vec<&ToolCall> = if let Some(params) = &assertion.params {
        matching_calls
            .into_iter()
            .filter(|call| params_match(params, &call.params))
            .collect()
    } else {
        matching_calls
    };

    let tool_was_called = !calls_with_matching_params.is_empty();

    // Handle called_after assertion
    if let Some(after_tool) = &assertion.called_after {
        return evaluate_called_after(assertion, after_tool, tool_calls);
    }

    // Check if called matches expectation
    if assertion.called && !tool_was_called {
        let param_desc = assertion
            .params
            .as_ref()
            .map(|p| format!(" with params {:?}", p))
            .unwrap_or_default();
        AssertionResult::Fail {
            reason: format!("Tool '{}'{} was never called", assertion.tool, param_desc),
        }
    } else if !assertion.called && tool_was_called {
        let found_call = calls_with_matching_params.first().unwrap();
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was called but should not have been. Found: {:?}",
                assertion.tool, found_call.params
            ),
        }
    } else {
        AssertionResult::Pass
    }
}

fn evaluate_called_after(
    assertion: &Assertion,
    after_tool: &str,
    tool_calls: &[ToolCall],
) -> AssertionResult {
    let mut seen_after = false;

    for call in tool_calls {
        if call.name == after_tool {
            seen_after = true;
        }
        if call.name == assertion.tool && seen_after {
            // Check params if specified
            if let Some(params) = &assertion.params {
                if params_match(params, &call.params) {
                    return AssertionResult::Pass;
                }
            } else {
                return AssertionResult::Pass;
            }
        }
    }

    if !seen_after {
        AssertionResult::Fail {
            reason: format!("Tool '{}' was never called", after_tool),
        }
    } else {
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was not called after '{}'",
                assertion.tool, after_tool
            ),
        }
    }
}

fn params_match(expected: &HashMap<String, String>, actual: &serde_json::Value) -> bool {
    for (key, pattern) in expected {
        let actual_value = actual.get(key);

        let actual_str = match actual_value {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(v) => v.to_string(),
            None => return false,
        };

        // Try glob pattern first
        if let Ok(glob) = Pattern::new(pattern) {
            if glob.matches(&actual_str) {
                continue;
            }
        }

        // Try regex
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&actual_str) {
                continue;
            }
        }

        // Exact match fallback
        if &actual_str != pattern {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn make_call(name: &str, params: serde_json::Value) -> ToolCall {
        ToolCall {
            name: name.to_string(),
            params,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_tool_called() {
        let assertion = Assertion {
            tool: "Read".to_string(),
            called: true,
            params: None,
            called_after: None,
        };
        let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];
        let result = evaluate_single_assertion(&assertion, &calls);
        assert!(result.is_pass());
    }

    #[test]
    fn test_tool_not_called() {
        let assertion = Assertion {
            tool: "Read".to_string(),
            called: false,
            params: Some(HashMap::from([("file_path".to_string(), "*.env".to_string())])),
            called_after: None,
        };
        let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];
        let result = evaluate_single_assertion(&assertion, &calls);
        assert!(result.is_pass());
    }

    #[test]
    fn test_glob_matching() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "*.env".to_string());

        assert!(params_match(&params, &json!({"file_path": ".env"})));
        assert!(params_match(&params, &json!({"file_path": "test.env"})));
        assert!(!params_match(&params, &json!({"file_path": "test.txt"})));
    }
}

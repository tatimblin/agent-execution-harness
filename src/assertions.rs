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
    /// Agent to use for this test (defaults to "claude").
    #[serde(default)]
    pub agent: Option<String>,
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
    /// Assert this tool is called before another tool
    pub called_before: Option<String>,
    /// Assert exact number of times the tool was called
    pub call_count: Option<u32>,
    /// Assert maximum number of times the tool can be called
    pub max_calls: Option<u32>,
    /// Assert minimum number of times the tool must be called
    pub min_calls: Option<u32>,
    /// Assert parameters for specific call indices (1-based)
    pub nth_call_params: Option<HashMap<u32, HashMap<String, String>>>,
    /// Assert parameters for the first call
    pub first_call_params: Option<HashMap<String, String>>,
    /// Assert parameters for the last call
    pub last_call_params: Option<HashMap<String, String>>,
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
    let mut results = Vec::new();

    for assertion in assertions {
        // 1. Validate assertion configuration
        if let Err(err) = validate_assertion(assertion) {
            results.push((
                format!("{} (invalid)", assertion.tool),
                AssertionResult::Fail { reason: err },
            ));
            continue;
        }

        // 2. Evaluate presence (called: true/false) - only if not using ordering assertions
        if assertion.called_after.is_none() && assertion.called_before.is_none() {
            let description = format_assertion_description(assertion, None);
            let result = evaluate_single_assertion(assertion, tool_calls);
            results.push((description, result));
        }

        // 3. Evaluate ordering: called_after
        if let Some(after_tool) = &assertion.called_after {
            let description = format_assertion_description(assertion, None);
            let result = evaluate_called_after(assertion, after_tool, tool_calls);
            results.push((description, result));
        }

        // 4. Evaluate ordering: called_before
        if let Some(before_tool) = &assertion.called_before {
            let description = format_assertion_description(assertion, None);
            let result = evaluate_called_before(assertion, before_tool, tool_calls);
            results.push((description, result));
        }

        // 5. Evaluate count constraints
        if let Some(count) = assertion.call_count {
            let description = format_count_description(&assertion.tool, "call_count ==", count);
            let result = evaluate_call_count(assertion, tool_calls, count);
            results.push((description, result));
        }

        if let Some(max) = assertion.max_calls {
            let description = format_count_description(&assertion.tool, "max_calls <=", max);
            let result = evaluate_max_calls(assertion, tool_calls, max);
            results.push((description, result));
        }

        if let Some(min) = assertion.min_calls {
            let description = format_count_description(&assertion.tool, "min_calls >=", min);
            let result = evaluate_min_calls(assertion, tool_calls, min);
            results.push((description, result));
        }

        // 6. Evaluate parameter assertions
        if let Some(nth_params) = &assertion.nth_call_params {
            for (n, params) in nth_params {
                let description = format!(
                    "{} nth_call_params[{}] matches {:?}",
                    assertion.tool, n, params
                );
                let nth_results = evaluate_nth_call_params(assertion, tool_calls, nth_params);
                // Get the result for this specific n
                let index = nth_params.keys().position(|k| k == n).unwrap_or(0);
                if let Some(result) = nth_results.into_iter().nth(index) {
                    results.push((description, result));
                }
            }
        }

        if let Some(first_params) = &assertion.first_call_params {
            let description = format_params_description(&assertion.tool, "first_call_params");
            let result = evaluate_first_call_params(assertion, tool_calls, first_params);
            results.push((description, result));
        }

        if let Some(last_params) = &assertion.last_call_params {
            let description = format_params_description(&assertion.tool, "last_call_params");
            let result = evaluate_last_call_params(assertion, tool_calls, last_params);
            results.push((description, result));
        }
    }

    results
}

fn format_assertion_description(assertion: &Assertion, suffix: Option<&str>) -> String {
    let mut desc = assertion.tool.clone();

    if let Some(params) = &assertion.params {
        let param_str: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}='{}'", k, v))
            .collect();
        desc = format!("{} with {}", desc, param_str.join(", "));
    }

    let base_desc = if assertion.called {
        if let Some(after) = &assertion.called_after {
            format!("{} called after {}", desc, after)
        } else if let Some(before) = &assertion.called_before {
            format!("{} called before {}", desc, before)
        } else {
            format!("{} called", desc)
        }
    } else {
        format!("{} not called", desc)
    };

    match suffix {
        Some(s) => format!("{} {}", base_desc, s),
        None => base_desc,
    }
}

fn format_count_description(tool: &str, assertion_type: &str, count: u32) -> String {
    format!("{} {} {}", tool, assertion_type, count)
}

fn format_params_description(tool: &str, assertion_type: &str) -> String {
    format!("{} {}", tool, assertion_type)
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

/// Validate an assertion for invalid field combinations
fn validate_assertion(assertion: &Assertion) -> Result<(), String> {
    // called: false is mutually exclusive with count assertions
    if !assertion.called {
        if assertion.call_count.is_some() {
            return Err("'called: false' cannot be combined with 'call_count'".to_string());
        }
        if assertion.min_calls.is_some() {
            return Err("'called: false' cannot be combined with 'min_calls'".to_string());
        }
        if assertion.max_calls.is_some() && assertion.max_calls != Some(0) {
            return Err(
                "'called: false' cannot be combined with 'max_calls' (except max_calls: 0)"
                    .to_string(),
            );
        }
    }
    Ok(())
}

/// Evaluate call_count assertion
fn evaluate_call_count(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    expected_count: u32,
) -> AssertionResult {
    let matching_calls: Vec<&ToolCall> = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .filter(|call| {
            if let Some(params) = &assertion.params {
                params_match(params, &call.params)
            } else {
                true
            }
        })
        .collect();

    let actual_count = matching_calls.len() as u32;
    if actual_count == expected_count {
        AssertionResult::Pass
    } else {
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was called {} times, expected exactly {}",
                assertion.tool, actual_count, expected_count
            ),
        }
    }
}

/// Evaluate max_calls assertion
fn evaluate_max_calls(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    max: u32,
) -> AssertionResult {
    let matching_calls: Vec<&ToolCall> = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .filter(|call| {
            if let Some(params) = &assertion.params {
                params_match(params, &call.params)
            } else {
                true
            }
        })
        .collect();

    let actual_count = matching_calls.len() as u32;
    if actual_count <= max {
        AssertionResult::Pass
    } else {
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was called {} times, expected at most {}",
                assertion.tool, actual_count, max
            ),
        }
    }
}

/// Evaluate min_calls assertion
fn evaluate_min_calls(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    min: u32,
) -> AssertionResult {
    let matching_calls: Vec<&ToolCall> = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .filter(|call| {
            if let Some(params) = &assertion.params {
                params_match(params, &call.params)
            } else {
                true
            }
        })
        .collect();

    let actual_count = matching_calls.len() as u32;
    if actual_count >= min {
        AssertionResult::Pass
    } else {
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was called {} times, expected at least {}",
                assertion.tool, actual_count, min
            ),
        }
    }
}

/// Evaluate called_before assertion (this tool must be called before another)
fn evaluate_called_before(
    assertion: &Assertion,
    before_tool: &str,
    tool_calls: &[ToolCall],
) -> AssertionResult {
    let mut seen_this_tool = false;

    for call in tool_calls {
        if call.name == assertion.tool {
            // Check params if specified
            if let Some(params) = &assertion.params {
                if params_match(params, &call.params) {
                    seen_this_tool = true;
                }
            } else {
                seen_this_tool = true;
            }
        }
        if call.name == before_tool && seen_this_tool {
            return AssertionResult::Pass;
        }
    }

    let this_tool_called = tool_calls.iter().any(|c| c.name == assertion.tool);
    let before_tool_called = tool_calls.iter().any(|c| c.name == before_tool);

    if !this_tool_called {
        AssertionResult::Fail {
            reason: format!("Tool '{}' was never called", assertion.tool),
        }
    } else if !before_tool_called {
        AssertionResult::Fail {
            reason: format!("Tool '{}' was never called", before_tool),
        }
    } else {
        AssertionResult::Fail {
            reason: format!(
                "Tool '{}' was not called before '{}'",
                assertion.tool, before_tool
            ),
        }
    }
}

/// Evaluate nth_call_params assertion (1-based indexing)
fn evaluate_nth_call_params(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    nth_params: &HashMap<u32, HashMap<String, String>>,
) -> Vec<AssertionResult> {
    let matching_calls: Vec<&ToolCall> = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .collect();

    let mut results = Vec::new();

    for (n, expected_params) in nth_params {
        // Convert 1-based to 0-based index
        let index = (*n as usize).saturating_sub(1);
        if let Some(call) = matching_calls.get(index) {
            if params_match(expected_params, &call.params) {
                results.push(AssertionResult::Pass);
            } else {
                results.push(AssertionResult::Fail {
                    reason: format!(
                        "Tool '{}' call #{} params did not match. Expected {:?}, got {:?}",
                        assertion.tool, n, expected_params, call.params
                    ),
                });
            }
        } else {
            results.push(AssertionResult::Fail {
                reason: format!(
                    "Tool '{}' call #{} does not exist (only {} calls made)",
                    assertion.tool,
                    n,
                    matching_calls.len()
                ),
            });
        }
    }

    results
}

/// Evaluate first_call_params assertion
fn evaluate_first_call_params(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    expected_params: &HashMap<String, String>,
) -> AssertionResult {
    let first_call = tool_calls.iter().find(|call| call.name == assertion.tool);

    match first_call {
        Some(call) => {
            if params_match(expected_params, &call.params) {
                AssertionResult::Pass
            } else {
                AssertionResult::Fail {
                    reason: format!(
                        "Tool '{}' first call params did not match. Expected {:?}, got {:?}",
                        assertion.tool, expected_params, call.params
                    ),
                }
            }
        }
        None => AssertionResult::Fail {
            reason: format!("Tool '{}' was never called", assertion.tool),
        },
    }
}

/// Evaluate last_call_params assertion
fn evaluate_last_call_params(
    assertion: &Assertion,
    tool_calls: &[ToolCall],
    expected_params: &HashMap<String, String>,
) -> AssertionResult {
    let last_call = tool_calls
        .iter()
        .filter(|call| call.name == assertion.tool)
        .last();

    match last_call {
        Some(call) => {
            if params_match(expected_params, &call.params) {
                AssertionResult::Pass
            } else {
                AssertionResult::Fail {
                    reason: format!(
                        "Tool '{}' last call params did not match. Expected {:?}, got {:?}",
                        assertion.tool, expected_params, call.params
                    ),
                }
            }
        }
        None => AssertionResult::Fail {
            reason: format!("Tool '{}' was never called", assertion.tool),
        },
    }
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

    fn default_assertion(tool: &str) -> Assertion {
        Assertion {
            tool: tool.to_string(),
            called: true,
            params: None,
            called_after: None,
            called_before: None,
            call_count: None,
            max_calls: None,
            min_calls: None,
            nth_call_params: None,
            first_call_params: None,
            last_call_params: None,
        }
    }

    #[test]
    fn test_tool_called() {
        let assertion = default_assertion("Read");
        let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];
        let result = evaluate_single_assertion(&assertion, &calls);
        assert!(matches!(result, AssertionResult::Pass));
    }

    #[test]
    fn test_tool_not_called() {
        let mut assertion = default_assertion("Read");
        assertion.called = false;
        assertion.params = Some(HashMap::from([("file_path".to_string(), "*.env".to_string())]));

        let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];
        let result = evaluate_single_assertion(&assertion, &calls);
        assert!(matches!(result, AssertionResult::Pass));
    }

    #[test]
    fn test_glob_matching() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "*.env".to_string());

        assert!(params_match(&params, &json!({"file_path": ".env"})));
        assert!(params_match(&params, &json!({"file_path": "test.env"})));
        assert!(!params_match(&params, &json!({"file_path": "test.txt"})));
    }

    #[test]
    fn test_call_count_exact() {
        let mut assertion = default_assertion("Read");
        assertion.call_count = Some(2);

        let calls = vec![
            make_call("Read", json!({"file_path": "/a.txt"})),
            make_call("Read", json!({"file_path": "/b.txt"})),
        ];
        let result = evaluate_call_count(&assertion, &calls, 2);
        assert!(matches!(result, AssertionResult::Pass));

        // Wrong count should fail
        let result = evaluate_call_count(&assertion, &calls, 3);
        assert!(matches!(result, AssertionResult::Fail { .. }));
    }

    #[test]
    fn test_max_calls() {
        let mut assertion = default_assertion("Read");
        assertion.max_calls = Some(2);

        let calls = vec![
            make_call("Read", json!({"file_path": "/a.txt"})),
            make_call("Read", json!({"file_path": "/b.txt"})),
        ];
        let result = evaluate_max_calls(&assertion, &calls, 2);
        assert!(matches!(result, AssertionResult::Pass));

        let result = evaluate_max_calls(&assertion, &calls, 3);
        assert!(matches!(result, AssertionResult::Pass));

        // Too many calls should fail
        let result = evaluate_max_calls(&assertion, &calls, 1);
        assert!(matches!(result, AssertionResult::Fail { .. }));
    }

    #[test]
    fn test_min_calls() {
        let mut assertion = default_assertion("Read");
        assertion.min_calls = Some(2);

        let calls = vec![
            make_call("Read", json!({"file_path": "/a.txt"})),
            make_call("Read", json!({"file_path": "/b.txt"})),
        ];
        let result = evaluate_min_calls(&assertion, &calls, 2);
        assert!(matches!(result, AssertionResult::Pass));

        let result = evaluate_min_calls(&assertion, &calls, 1);
        assert!(matches!(result, AssertionResult::Pass));

        // Too few calls should fail
        let result = evaluate_min_calls(&assertion, &calls, 3);
        assert!(matches!(result, AssertionResult::Fail { .. }));
    }

    #[test]
    fn test_called_before() {
        let mut assertion = default_assertion("Read");
        assertion.called_before = Some("Write".to_string());

        // Read before Write - should pass
        let calls = vec![
            make_call("Read", json!({"file_path": "/a.txt"})),
            make_call("Write", json!({"file_path": "/b.txt"})),
        ];
        let result = evaluate_called_before(&assertion, "Write", &calls);
        assert!(matches!(result, AssertionResult::Pass));

        // Write before Read - should fail
        let calls = vec![
            make_call("Write", json!({"file_path": "/b.txt"})),
            make_call("Read", json!({"file_path": "/a.txt"})),
        ];
        let result = evaluate_called_before(&assertion, "Write", &calls);
        assert!(matches!(result, AssertionResult::Fail { .. }));
    }

    #[test]
    fn test_first_call_params() {
        let assertion = default_assertion("Read");
        let expected = HashMap::from([("file_path".to_string(), "/first.txt".to_string())]);

        let calls = vec![
            make_call("Read", json!({"file_path": "/first.txt"})),
            make_call("Read", json!({"file_path": "/second.txt"})),
        ];
        let result = evaluate_first_call_params(&assertion, &calls, &expected);
        assert!(matches!(result, AssertionResult::Pass));

        // Wrong first call params should fail
        let expected_wrong = HashMap::from([("file_path".to_string(), "/second.txt".to_string())]);
        let result = evaluate_first_call_params(&assertion, &calls, &expected_wrong);
        assert!(matches!(result, AssertionResult::Fail { .. }));
    }

    #[test]
    fn test_last_call_params() {
        let assertion = default_assertion("Read");
        let expected = HashMap::from([("file_path".to_string(), "/last.txt".to_string())]);

        let calls = vec![
            make_call("Read", json!({"file_path": "/first.txt"})),
            make_call("Read", json!({"file_path": "/last.txt"})),
        ];
        let result = evaluate_last_call_params(&assertion, &calls, &expected);
        assert!(matches!(result, AssertionResult::Pass));
    }

    #[test]
    fn test_nth_call_params() {
        let assertion = default_assertion("Read");
        let mut nth_params = HashMap::new();
        nth_params.insert(1, HashMap::from([("file_path".to_string(), "/first.txt".to_string())]));
        nth_params.insert(2, HashMap::from([("file_path".to_string(), "/second.txt".to_string())]));

        let calls = vec![
            make_call("Read", json!({"file_path": "/first.txt"})),
            make_call("Read", json!({"file_path": "/second.txt"})),
        ];
        let results = evaluate_nth_call_params(&assertion, &calls, &nth_params);
        assert!(results.iter().all(|r| matches!(r, AssertionResult::Pass)));
    }

    #[test]
    fn test_validate_assertion_mutual_exclusivity() {
        // called: false with call_count should fail validation
        let mut assertion = default_assertion("Read");
        assertion.called = false;
        assertion.call_count = Some(2);
        assert!(validate_assertion(&assertion).is_err());

        // called: false with min_calls should fail validation
        let mut assertion = default_assertion("Read");
        assertion.called = false;
        assertion.min_calls = Some(1);
        assert!(validate_assertion(&assertion).is_err());

        // called: false with max_calls: 0 should pass (special case)
        let mut assertion = default_assertion("Read");
        assertion.called = false;
        assertion.max_calls = Some(0);
        assert!(validate_assertion(&assertion).is_ok());

        // called: false with max_calls > 0 should fail
        let mut assertion = default_assertion("Read");
        assertion.called = false;
        assertion.max_calls = Some(1);
        assert!(validate_assertion(&assertion).is_err());
    }
}

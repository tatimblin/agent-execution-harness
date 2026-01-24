//! Tests for the fluent assertion API.

use super::*;
use crate::params;
use crate::parser::ToolCall;
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
fn test_expect_tool_called() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    // Should not panic
    expect_tools(&calls).tool(Tool::Read).to_be_called();
}

#[test]
fn test_expect_tool_not_called() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    // Should not panic
    expect_tools(&calls).tool(Tool::Bash).not_to_be_called();
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_expect_tool_called_fails() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    // Should panic - Bash was never called
    expect_tools(&calls).tool(Tool::Bash).to_be_called();
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_expect_tool_not_called_fails() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    // Should panic - Read was called
    expect_tools(&calls).tool(Tool::Read).not_to_be_called();
}

#[test]
fn test_with_params_regex() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    expect_tools(&calls)
        .tool(Tool::Read)
        .with_params(params! {"file_path" => r".*\.txt"})
        .to_be_called();
}

#[test]
fn test_with_params_no_match() {
    let calls = vec![make_call("Read", json!({"file_path": "/tmp/test.txt"}))];

    // Read was called but not with .rs files
    expect_tools(&calls)
        .tool(Tool::Read)
        .with_params(params! {"file_path" => r".*\.rs"})
        .not_to_be_called();
}

#[test]
fn test_times_exact() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).times(2).to_be_called();
}

#[test]
#[should_panic(expected = "expected 3 calls, got 2")]
fn test_times_wrong_count() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).times(3).to_be_called();
}

#[test]
fn test_at_least() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
        make_call("Read", json!({"file_path": "/c.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).at_least(2).to_be_called();
    expect_tools(&calls).tool(Tool::Read).at_least(3).to_be_called();
}

#[test]
#[should_panic(expected = "expected at least 3 calls, got 2")]
fn test_at_least_fails() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).at_least(3).to_be_called();
}

#[test]
fn test_at_most() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).at_most(2).to_be_called();
    expect_tools(&calls).tool(Tool::Read).at_most(5).to_be_called();
}

#[test]
#[should_panic(expected = "expected at most 2 calls, got 3")]
fn test_at_most_fails() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Read", json!({"file_path": "/b.txt"})),
        make_call("Read", json!({"file_path": "/c.txt"})),
    ];

    expect_tools(&calls).tool(Tool::Read).at_most(2).to_be_called();
}

#[test]
fn test_after() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/input.txt"})),
        make_call("Write", json!({"file_path": "/output.txt"})),
    ];

    expect_tools(&calls)
        .tool(Tool::Write)
        .after(Tool::Read)
        .to_be_called();
}

#[test]
#[should_panic(expected = "was not called after")]
fn test_after_wrong_order() {
    let calls = vec![
        make_call("Write", json!({"file_path": "/output.txt"})),
        make_call("Read", json!({"file_path": "/input.txt"})),
    ];

    expect_tools(&calls)
        .tool(Tool::Write)
        .after(Tool::Read)
        .to_be_called();
}

#[test]
fn test_before() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/input.txt"})),
        make_call("Write", json!({"file_path": "/output.txt"})),
    ];

    expect_tools(&calls)
        .tool(Tool::Read)
        .before(Tool::Write)
        .to_be_called();
}

#[test]
#[should_panic(expected = "was not called before")]
fn test_before_wrong_order() {
    let calls = vec![
        make_call("Write", json!({"file_path": "/output.txt"})),
        make_call("Read", json!({"file_path": "/input.txt"})),
    ];

    expect_tools(&calls)
        .tool(Tool::Read)
        .before(Tool::Write)
        .to_be_called();
}

#[test]
fn test_nth_call() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/first.txt"})),
        make_call("Read", json!({"file_path": "/second.txt"})),
        make_call("Read", json!({"file_path": "/third.txt"})),
    ];

    // New builder-based API
    expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(1)
        .has_params(params! {"file_path" => "/first.txt"});

    expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(2)
        .has_params(params! {"file_path" => "/second.txt"});

    expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(3)
        .has_params(params! {"file_path" => "/third.txt"});
}

#[test]
#[should_panic(expected = "call #4 to exist")]
fn test_nth_call_out_of_bounds() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/first.txt"})),
        make_call("Read", json!({"file_path": "/second.txt"})),
    ];

    // Will panic when trying to get call #4
    let _ = expect_tools(&calls).tool(Tool::Read).nth_call(4);
}

#[test]
#[should_panic(expected = "params did not match")]
fn test_nth_call_wrong_params() {
    let calls = vec![make_call("Read", json!({"file_path": "/first.txt"}))];

    expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(1)
        .has_params(params! {"file_path" => "/wrong.txt"});
}

#[test]
fn test_last_call() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/first.txt"})),
        make_call("Read", json!({"file_path": "/second.txt"})),
        make_call("Read", json!({"file_path": "/last.txt"})),
    ];

    expect_tools(&calls)
        .tool(Tool::Read)
        .last_call()
        .has_params(params! {"file_path" => "/last.txt"});
}

#[test]
fn test_nth_call_evaluate_params() {
    let calls = vec![make_call("Read", json!({"file_path": "/test.txt"}))];

    // Non-panicking evaluation
    let result = expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(1)
        .evaluate_params(params! {"file_path" => r".*\.txt"});
    assert!(result.passed);

    let result = expect_tools(&calls)
        .tool(Tool::Read)
        .nth_call(1)
        .evaluate_params(params! {"file_path" => r".*\.rs"});
    assert!(!result.passed);
}

#[test]
fn test_evaluate_non_panicking() {
    let calls = vec![make_call("Read", json!({"file_path": "/test.txt"}))];

    let result = expect_tools(&calls).tool(Tool::Read).evaluate();
    assert!(result.passed);
    assert!(result.reason.is_none());

    let result = expect_tools(&calls).tool(Tool::Bash).evaluate();
    assert!(!result.passed);
    assert!(result.reason.is_some());
}

#[test]
fn test_empty_tool_calls() {
    let calls: Vec<ToolCall> = vec![];

    expect_tools(&calls).tool(Tool::Read).not_to_be_called();
    expect_tools(&calls).tool(Tool::Bash).not_to_be_called();
}

#[test]
fn test_multiple_assertions_same_tool() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/config.json"})),
        make_call("Read", json!({"file_path": "/settings.yaml"})),
    ];

    expect_tools(&calls).tool(Tool::Read).to_be_called();
    expect_tools(&calls).tool(Tool::Read).at_least(2).to_be_called();
    expect_tools(&calls)
        .tool(Tool::Read)
        .with_params(params! {"file_path" => r".*\.json"})
        .to_be_called();
    expect_tools(&calls)
        .tool(Tool::Read)
        .with_params(params! {"file_path" => r".*\.yaml"})
        .to_be_called();
}

#[test]
fn test_chained_constraints() {
    let calls = vec![
        make_call("Glob", json!({"pattern": "**/*.rs"})),
        make_call("Read", json!({"file_path": "/src/main.rs"})),
        make_call("Read", json!({"file_path": "/src/lib.rs"})),
        make_call("Write", json!({"file_path": "/output.txt"})),
    ];

    // Read was called at least twice, after Glob, and before Write
    expect_tools(&calls)
        .tool(Tool::Read)
        .at_least(2)
        .after(Tool::Glob)
        .to_be_called();
}

#[test]
fn test_evaluate_not_called() {
    let calls = vec![make_call("Read", json!({"file_path": "/test.txt"}))];

    // Bash was not called - should pass
    let result = expect_tools(&calls).tool(Tool::Bash).evaluate_not_called();
    assert!(result.passed);

    // Read was called - should fail
    let result = expect_tools(&calls).tool(Tool::Read).evaluate_not_called();
    assert!(!result.passed);
}

#[test]
fn test_all_constraints_checked() {
    // Test that ALL constraints are checked, not just the first one
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
        make_call("Glob", json!({"pattern": "**/*"})),
    ];

    // Read was called once but AFTER Glob (wrong order)
    // Both count (times(2)) and ordering (after Glob) should fail
    let result = expect_tools(&calls)
        .tool(Tool::Read)
        .times(2)
        .after(Tool::Glob)
        .evaluate();

    assert!(!result.passed);
    // Should contain both failures
    let reason = result.reason.unwrap();
    assert!(reason.contains("expected 2 calls"), "Should mention count failure: {}", reason);
    assert!(reason.contains("was not called after"), "Should mention ordering failure: {}", reason);
}

#[test]
fn test_multiple_count_constraints() {
    let calls = vec![
        make_call("Read", json!({"file_path": "/a.txt"})),
    ];

    // Check that min and max constraints work together
    let result = expect_tools(&calls)
        .tool(Tool::Read)
        .at_least(1)
        .at_most(5)
        .evaluate();
    assert!(result.passed);

    // Should fail when count is outside range
    let result = expect_tools(&calls)
        .tool(Tool::Read)
        .at_least(3)
        .evaluate();
    assert!(!result.passed);
}

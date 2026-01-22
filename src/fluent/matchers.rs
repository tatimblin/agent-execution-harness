//! Parameter matching utilities for tool call assertions.
//!
//! This module provides utilities for matching expected parameters against
//! actual tool call parameters, supporting glob patterns, regex, and exact matches.

use glob::Pattern;
use regex::Regex;
use std::collections::HashMap;

/// Match expected parameters against actual tool call parameters.
///
/// Supports three matching modes (tried in order):
/// 1. **Glob patterns**: e.g., `*.txt`, `**/config.json`
/// 2. **Regex**: e.g., `^/tmp/.*\.log$`
/// 3. **Exact match**: literal string comparison
///
/// # Arguments
///
/// * `expected` - Map of parameter names to expected patterns/values
/// * `actual` - The actual JSON value containing the tool call parameters
///
/// # Returns
///
/// `true` if all expected parameters match their actual values.
///
/// # Example
///
/// ```rust
/// use aptitude::params_match;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// let mut expected = HashMap::new();
/// expected.insert("file_path".to_string(), "*.txt".to_string());
///
/// assert!(params_match(&expected, &json!({"file_path": "test.txt"})));
/// assert!(!params_match(&expected, &json!({"file_path": "test.rs"})));
/// ```
pub fn params_match(expected: &HashMap<String, String>, actual: &serde_json::Value) -> bool {
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

/// Create a parameter map from key-value pairs.
///
/// This is a convenience macro for creating parameter expectations.
///
/// # Example
///
/// ```rust,ignore
/// use aptitude::params;
///
/// let params = params! {
///     "file_path" => "*.txt",
///     "content" => "hello"
/// };
/// ```
#[macro_export]
macro_rules! params {
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $value.to_string());
        )*
        map
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_glob_matching() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "*.env".to_string());

        assert!(params_match(&params, &json!({"file_path": ".env"})));
        assert!(params_match(&params, &json!({"file_path": "test.env"})));
        assert!(!params_match(&params, &json!({"file_path": "test.txt"})));
    }

    #[test]
    fn test_glob_path_matching() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "**/config.json".to_string());

        assert!(params_match(
            &params,
            &json!({"file_path": "src/config.json"})
        ));
        assert!(params_match(&params, &json!({"file_path": "config.json"})));
    }

    #[test]
    fn test_regex_matching() {
        let mut params = HashMap::new();
        params.insert("command".to_string(), r"^npm (install|i)$".to_string());

        assert!(params_match(&params, &json!({"command": "npm install"})));
        assert!(params_match(&params, &json!({"command": "npm i"})));
        assert!(!params_match(&params, &json!({"command": "npm run"})));
    }

    #[test]
    fn test_exact_matching() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "/tmp/test.txt".to_string());

        assert!(params_match(
            &params,
            &json!({"file_path": "/tmp/test.txt"})
        ));
        assert!(!params_match(
            &params,
            &json!({"file_path": "/tmp/other.txt"})
        ));
    }

    #[test]
    fn test_missing_key() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "test.txt".to_string());

        assert!(!params_match(&params, &json!({"other_key": "test.txt"})));
    }

    #[test]
    fn test_multiple_params() {
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), "*.txt".to_string());
        params.insert("content".to_string(), "hello.*".to_string());

        assert!(params_match(
            &params,
            &json!({"file_path": "test.txt", "content": "hello world"})
        ));
        assert!(!params_match(
            &params,
            &json!({"file_path": "test.txt", "content": "goodbye"})
        ));
    }

    #[test]
    fn test_non_string_values() {
        let mut params = HashMap::new();
        params.insert("count".to_string(), "42".to_string());

        assert!(params_match(&params, &json!({"count": 42})));
    }

    #[test]
    fn test_params_macro() {
        let params = params! {
            "file_path" => "test.txt",
            "content" => "hello"
        };

        assert_eq!(params.get("file_path"), Some(&"test.txt".to_string()));
        assert_eq!(params.get("content"), Some(&"hello".to_string()));
    }
}

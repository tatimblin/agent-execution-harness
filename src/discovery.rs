//! Test file discovery using glob patterns and walkdir.
//!
//! This module handles finding test files in directories based on
//! configurable patterns and exclusion rules.

use anyhow::Result;
use glob::Pattern;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

/// Discover test files in a directory according to config.
///
/// Returns a sorted list of paths to test files that match the pattern.
pub fn discover_tests(dir: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let patterns = parse_patterns(&config.test_pattern)?;

    let mut tests = Vec::new();

    let walker = if config.recursive {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).max_depth(1)
    };

    for entry in walker
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), &config.exclude))
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && matches_any_pattern(path, &patterns) {
            tests.push(path.to_path_buf());
        }
    }

    // Sort by path for consistent ordering
    tests.sort();

    Ok(tests)
}

/// Parse a pattern string that may contain brace expansion.
/// E.g., "*.{yaml,yml}" expands to ["*.yaml", "*.yml"]
fn parse_patterns(pattern: &str) -> Result<Vec<Pattern>> {
    let expanded = expand_braces(pattern);
    expanded
        .into_iter()
        .map(|p| {
            Pattern::new(&p)
                .map_err(|e| anyhow::anyhow!("Invalid test pattern '{}': {}", p, e))
        })
        .collect()
}

/// Expand brace expressions in a pattern.
/// E.g., "*.{yaml,yml}" -> ["*.yaml", "*.yml"]
fn expand_braces(pattern: &str) -> Vec<String> {
    if let Some(start) = pattern.find('{') {
        if let Some(end) = pattern[start..].find('}') {
            let prefix = &pattern[..start];
            let suffix = &pattern[start + end + 1..];
            let alternatives = &pattern[start + 1..start + end];

            return alternatives
                .split(',')
                .flat_map(|alt| {
                    let expanded = format!("{}{}{}", prefix, alt, suffix);
                    expand_braces(&expanded)
                })
                .collect();
        }
    }
    vec![pattern.to_string()]
}

/// Check if a file name matches any of the glob patterns.
fn matches_any_pattern(path: &Path, patterns: &[Pattern]) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| patterns.iter().any(|p| p.matches(name)))
        .unwrap_or(false)
}

/// Check if a path should be excluded based on directory names.
fn should_exclude(path: &Path, excludes: &[String]) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                if excludes.iter().any(|e| e == name_str) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_braces_simple() {
        let expanded = expand_braces("*.{yaml,yml}");
        assert_eq!(expanded, vec!["*.yaml", "*.yml"]);
    }

    #[test]
    fn test_expand_braces_no_braces() {
        let expanded = expand_braces("*.yaml");
        assert_eq!(expanded, vec!["*.yaml"]);
    }

    #[test]
    fn test_expand_braces_multiple_alternatives() {
        let expanded = expand_braces("*.{a,b,c}");
        assert_eq!(expanded, vec!["*.a", "*.b", "*.c"]);
    }

    #[test]
    fn test_matches_any_pattern_yaml() {
        let patterns = parse_patterns("*.{yaml,yml}").unwrap();
        assert!(matches_any_pattern(Path::new("/foo/test.yaml"), &patterns));
        assert!(matches_any_pattern(Path::new("/foo/test.yml"), &patterns));
        assert!(!matches_any_pattern(Path::new("/foo/test.json"), &patterns));
    }

    #[test]
    fn test_matches_any_pattern_suffix() {
        let patterns = parse_patterns("*.test.yaml").unwrap();
        assert!(matches_any_pattern(Path::new("/foo/my.test.yaml"), &patterns));
        assert!(!matches_any_pattern(Path::new("/foo/test.yaml"), &patterns));
    }

    #[test]
    fn test_should_exclude() {
        let excludes = vec!["target".to_string(), "node_modules".to_string()];

        assert!(should_exclude(Path::new("/project/target/debug"), &excludes));
        assert!(should_exclude(
            Path::new("/project/node_modules/foo"),
            &excludes
        ));
        assert!(!should_exclude(Path::new("/project/src/main.rs"), &excludes));
    }
}

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A tool call extracted from Claude Code logs
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub params: Value,
    pub timestamp: DateTime<Utc>,
}

/// Lightweight struct to check entry type before full parse
#[derive(Debug, Deserialize)]
struct EntryTypeCheck {
    #[serde(rename = "type")]
    entry_type: Option<String>,
}

/// Raw log entry from JSONL (only valid for assistant messages)
#[derive(Debug, Deserialize)]
struct LogEntry {
    timestamp: Option<String>,
    message: Option<MessageContent>,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: Option<Vec<ContentBlock>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: Option<Value>,
    },
    #[serde(other)]
    Other,
}

/// Parse a JSONL file and extract all tool calls
pub fn parse_jsonl_file(path: &Path) -> Result<Vec<ToolCall>> {
    let file = File::open(path).context("Failed to open JSONL file")?;
    let reader = BufReader::new(file);
    let mut tool_calls = Vec::new();

    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        if let Some(calls) = parse_line_internal(&line)? {
            tool_calls.extend(calls);
        }
    }

    Ok(tool_calls)
}

/// Internal parsing: check type first, then parse full entry only for assistant messages
fn parse_line_internal(line: &str) -> Result<Option<Vec<ToolCall>>> {
    if line.trim().is_empty() {
        return Ok(None);
    }

    // First, check if this is an assistant message (lightweight parse)
    let type_check: EntryTypeCheck =
        serde_json::from_str(line).context("Failed to parse JSON line")?;

    if type_check.entry_type.as_deref() != Some("assistant") {
        // Not an assistant message, skip without full parse
        return Ok(None);
    }

    // Now do the full parse for assistant messages
    let entry: LogEntry =
        serde_json::from_str(line).context("Failed to parse assistant message")?;

    Ok(extract_tool_calls(&entry))
}

fn extract_tool_calls(entry: &LogEntry) -> Option<Vec<ToolCall>> {
    let timestamp = entry
        .timestamp
        .as_ref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let message = entry.message.as_ref()?;
    let content = message.content.as_ref()?;

    let calls: Vec<ToolCall> = content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::ToolUse { name, input, .. } => Some(ToolCall {
                name: name.clone(),
                params: input.clone(),
                timestamp,
            }),
            _ => None,
        })
        .collect();

    if calls.is_empty() {
        None
    } else {
        Some(calls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_use() {
        let json = r#"{"type":"assistant","timestamp":"2024-01-19T12:00:00Z","message":{"content":[{"type":"tool_use","id":"123","name":"Read","input":{"file_path":"/tmp/test.txt"}}]}}"#;
        let calls = parse_line_internal(json).unwrap().unwrap_or_default();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "Read");
    }

    #[test]
    fn test_skip_user_messages() {
        // User messages are skipped early (before full parse) based on type field
        let json = r#"{"type":"user","timestamp":"2024-01-19T12:00:00Z","message":{"content":"hello"}}"#;
        let calls = parse_line_internal(json).unwrap();
        assert!(calls.is_none());
    }

    #[test]
    fn test_skip_system_messages() {
        // System/meta messages are also skipped
        let json = r#"{"type":"system","subtype":"turn_duration","durationMs":318950}"#;
        let calls = parse_line_internal(json).unwrap();
        assert!(calls.is_none());
    }
}

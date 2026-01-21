//! Claude Code agent adapter.
//!
//! This adapter integrates with Claude Code CLI (`claude --print`).

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::parser::{parse_jsonl_file, ToolCall};
use super::mapping::{canonical, ToolNameMapping};
use super::traits::{Agent, ExecutionConfig, RawExecutionResult};

/// Claude Code agent adapter.
pub struct ClaudeAdapter {
    mapping: ToolNameMapping,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let mut mapping = ToolNameMapping::new();

        // Map Claude's PascalCase tool names to canonical snake_case names
        mapping
            .add("Read", canonical::READ_FILE)
            .add("Write", canonical::WRITE_FILE)
            .add("Edit", canonical::EDIT_FILE)
            .add("Bash", canonical::EXECUTE_COMMAND)
            .add("Grep", canonical::SEARCH_FILES)
            .add("Glob", canonical::GLOB_FILES)
            .add("LS", canonical::LIST_DIRECTORY)
            .add("AskUserQuestion", canonical::ASK_USER)
            .add("Task", canonical::TASK)
            .add("WebFetch", canonical::WEB_FETCH)
            .add("WebSearch", canonical::WEB_SEARCH)
            .add("NotebookEdit", canonical::NOTEBOOK_EDIT);

        Self { mapping }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ClaudeAdapter {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn execute(&self, prompt: &str, config: &ExecutionConfig) -> Result<RawExecutionResult> {
        // Get the claude projects directory to watch for new sessions
        let claude_dir = get_claude_projects_dir()?;

        // Get list of existing sessions before running
        let existing_sessions = list_session_files(&claude_dir)?;

        // Run claude with the prompt
        let mut cmd = Command::new("claude");
        cmd.arg("--print").arg(prompt).stdin(Stdio::null());

        if let Some(dir) = &config.working_dir {
            cmd.current_dir(dir);
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let _output = cmd.output().context("Failed to execute claude command")?;

        // Find the new session log file
        let session_log_path = find_new_session(&claude_dir, &existing_sessions)?;

        Ok(RawExecutionResult {
            session_log_path: Some(session_log_path),
        })
    }

    fn parse_session(&self, result: &RawExecutionResult) -> Result<Vec<ToolCall>> {
        let path = result
            .session_log_path
            .as_ref()
            .context("Claude requires session log path")?;
        parse_jsonl_file(path)
    }

    fn tool_mapping(&self) -> &ToolNameMapping {
        &self.mapping
    }

    fn is_available(&self) -> bool {
        Command::new("claude")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Get the Claude projects directory.
fn get_claude_projects_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let claude_dir = home.join(".claude").join("projects");

    if !claude_dir.exists() {
        anyhow::bail!(
            "Claude projects directory not found at {:?}. Is Claude Code installed?",
            claude_dir
        );
    }

    Ok(claude_dir)
}

/// List all JSONL session files in the claude directory.
fn list_session_files(claude_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if claude_dir.exists() {
        for entry in walkdir::WalkDir::new(claude_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "jsonl") {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}

/// Find a new session log file that wasn't in the existing list.
fn find_new_session(claude_dir: &PathBuf, existing: &[PathBuf]) -> Result<PathBuf> {
    let current = list_session_files(claude_dir)?;

    // Find files that are new or modified
    for path in current {
        if !existing.contains(&path) {
            return Ok(path);
        }
    }

    // If no new file, find the most recently modified
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

    for entry in walkdir::WalkDir::new(claude_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "jsonl") {
            if let Ok(metadata) = path.metadata() {
                if let Ok(modified) = metadata.modified() {
                    match &newest {
                        None => newest = Some((path.to_path_buf(), modified)),
                        Some((_, newest_time)) if modified > *newest_time => {
                            newest = Some((path.to_path_buf(), modified));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    newest
        .map(|(path, _)| path)
        .context("Could not find session log file")
}

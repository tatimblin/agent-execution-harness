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

        // Claude's tool names are already canonical - map to same names
        // This preserves the actual tool names from JSONL output
        mapping
            .add("Read", canonical::READ)
            .add("Write", canonical::WRITE)
            .add("Edit", canonical::EDIT)
            .add("Bash", canonical::BASH)
            .add("Grep", canonical::GREP)
            .add("Glob", canonical::GLOB)
            .add("LS", canonical::LIST_DIRECTORY)
            .add("AskUserQuestion", canonical::ASK_USER)
            .add("Task", canonical::TASK)
            .add("WebFetch", canonical::WEB_FETCH)
            .add("WebSearch", canonical::WEB_SEARCH)
            .add("NotebookEdit", canonical::NOTEBOOK_EDIT)
            .add("TodoWrite", canonical::TODO_WRITE)
            .add("KillShell", canonical::KILL_SHELL)
            .add("TaskOutput", canonical::TASK_OUTPUT);

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

        // Determine the specific project directory for the working directory
        let project_dir = get_project_dir_for_workdir(&claude_dir, &config.working_dir)?;

        // Get list of existing sessions before running (only in this project)
        let existing_sessions = list_session_files(&project_dir)?;

        // Run claude with the prompt
        let mut cmd = Command::new("claude");
        cmd.arg("--print").arg(prompt).stdin(Stdio::null());

        if let Some(dir) = &config.working_dir {
            cmd.current_dir(dir);
        }

        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        let output = cmd.output().context("Failed to execute claude command")?;

        // Capture stdout
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stdout = if stdout.is_empty() { None } else { Some(stdout) };

        // Find the new session log file (only in this project)
        let session_log_path = find_new_session(&project_dir, &existing_sessions)?;

        Ok(RawExecutionResult {
            session_log_path: Some(session_log_path),
            stdout,
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

/// Get the specific project directory for a given working directory.
///
/// Claude Code stores sessions in directories named after the working directory path,
/// with slashes replaced by dashes. E.g., /Users/foo/bar becomes -Users-foo-bar
fn get_project_dir_for_workdir(
    claude_dir: &PathBuf,
    working_dir: &Option<PathBuf>,
) -> Result<PathBuf> {
    let workdir = match working_dir {
        Some(dir) => dir
            .canonicalize()
            .context("Failed to canonicalize working directory")?,
        None => std::env::current_dir().context("Failed to get current directory")?,
    };

    // Convert path to Claude's project directory naming convention
    // /Users/foo/bar -> -Users-foo-bar
    let path_str = workdir.to_string_lossy();
    let project_name = path_str.replace('/', "-");

    let project_dir = claude_dir.join(&project_name);

    // If the specific project dir doesn't exist, fall back to searching all projects
    if !project_dir.exists() {
        return Ok(claude_dir.clone());
    }

    Ok(project_dir)
}

/// List all JSONL session files in the claude directory.
/// Excludes subagent logs (files in /subagents/ directories).
fn list_session_files(claude_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if claude_dir.exists() {
        for entry in walkdir::WalkDir::new(claude_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            // Skip subagent logs - we only want main session logs
            if path.to_string_lossy().contains("/subagents/") {
                continue;
            }
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
    for path in &current {
        if !existing.contains(path) {
            return Ok(path.clone());
        }
    }

    // If no new file, find the most recently modified from the filtered list
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;

    for path in current {
        if let Ok(metadata) = path.metadata() {
            if let Ok(modified) = metadata.modified() {
                match &newest {
                    None => newest = Some((path, modified)),
                    Some((_, newest_time)) if modified > *newest_time => {
                        newest = Some((path, modified));
                    }
                    _ => {}
                }
            }
        }
    }

    newest
        .map(|(path, _)| path)
        .context("Could not find session log file")
}

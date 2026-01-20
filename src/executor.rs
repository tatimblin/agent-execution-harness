use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Result of executing Claude
pub struct ExecutionResult {
    pub session_log_path: PathBuf,
    pub exit_code: i32,
}

/// Execute Claude with a given prompt and return the session log path
pub fn execute_claude(prompt: &str, working_dir: Option<&PathBuf>) -> Result<ExecutionResult> {
    // Get the claude projects directory to watch for new sessions
    let claude_dir = get_claude_projects_dir()?;

    // Get list of existing sessions before running
    let existing_sessions = list_session_files(&claude_dir)?;

    // Run claude with the prompt
    let mut cmd = Command::new("claude");
    cmd.arg("--print").arg(prompt).stdin(Stdio::null());

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().context("Failed to execute claude command")?;

    let exit_code = output.status.code().unwrap_or(-1);

    // Find the new session log file
    let session_log_path = find_new_session(&claude_dir, &existing_sessions)?;

    Ok(ExecutionResult {
        session_log_path,
        exit_code,
    })
}

/// Get the Claude projects directory
pub fn get_claude_projects_dir() -> Result<PathBuf> {
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

/// List all JSONL session files in the claude directory
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

/// Find a new session log file that wasn't in the existing list
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

/// Find the most recent session log file
pub fn find_latest_session() -> Result<PathBuf> {
    let claude_dir = get_claude_projects_dir()?;
    find_new_session(&claude_dir, &[])
}

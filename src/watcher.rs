use anyhow::Result;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use crate::parser::{parse_jsonl_line, ToolCall};

/// Watch a log file for new tool calls
/// For the hackathon MVP, this is a simple tail-like implementation
pub struct LogWatcher {
    path: std::path::PathBuf,
    last_position: u64,
}

impl LogWatcher {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            last_position: 0,
        }
    }

    /// Read any new lines from the log file and parse tool calls
    pub fn poll(&mut self) -> Result<Vec<ToolCall>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Seek, SeekFrom};

        let mut file = File::open(&self.path)?;
        let current_size = file.metadata()?.len();

        if current_size <= self.last_position {
            return Ok(Vec::new());
        }

        file.seek(SeekFrom::Start(self.last_position))?;
        let reader = BufReader::new(file);

        let mut tool_calls = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(calls) = parse_jsonl_line(&line) {
                tool_calls.extend(calls);
            }
        }

        self.last_position = current_size;
        Ok(tool_calls)
    }

    /// Reset to read from the beginning
    pub fn reset(&mut self) {
        self.last_position = 0;
    }
}

/// Create a file system watcher that notifies on changes
pub fn create_fs_watcher(path: &Path) -> Result<(RecommendedWatcher, Receiver<notify::Result<notify::Event>>)> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        Config::default().with_poll_interval(Duration::from_millis(100)),
    )?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    Ok((watcher, rx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_watcher() {
        let mut file = NamedTempFile::new().unwrap();

        // Write initial content
        writeln!(file, r#"{{"type":"user","message":"hello"}}"#).unwrap();
        file.flush().unwrap();

        let mut watcher = LogWatcher::new(file.path());

        // First poll should get nothing (no tool calls in user message)
        let calls = watcher.poll().unwrap();
        assert!(calls.is_empty());

        // Add a tool call
        writeln!(
            file,
            r#"{{"type":"assistant","timestamp":"2024-01-19T12:00:00Z","message":{{"content":[{{"type":"tool_use","id":"1","name":"Read","input":{{"file_path":"/test"}}}}]}}}}"#
        )
        .unwrap();
        file.flush().unwrap();

        // Second poll should get the tool call
        let calls = watcher.poll().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "Read");
    }
}

//! Tool enum representing canonical tool names.
//!
//! These match the actual tool names emitted by Claude Code in JSONL logs.

/// Canonical tool names as an enum for type safety.
///
/// These match the actual tool names emitted by Claude Code in JSONL output.
/// Using an enum provides compile-time checking and prevents typos.
///
/// # Example
///
/// ```rust
/// use aptitude::Tool;
///
/// let tool = Tool::Read;
/// assert_eq!(tool.as_str(), "Read");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    /// Read a file
    Read,
    /// Write a file
    Write,
    /// Edit a file
    Edit,
    /// Execute a bash command
    Bash,
    /// Search for files using glob patterns
    Glob,
    /// Search file contents using grep/regex
    Grep,
    /// Launch a subagent task
    Task,
    /// Fetch content from a URL
    WebFetch,
    /// Search the web
    WebSearch,
    /// Edit a Jupyter notebook cell
    NotebookEdit,
    /// Ask the user a question
    AskUserQuestion,
    /// Write to the todo list
    TodoWrite,
    /// Kill a background shell
    KillShell,
    /// Get output from a background task
    TaskOutput,
}

impl Tool {
    /// Get the canonical string name (matches JSONL output).
    ///
    /// # Example
    ///
    /// ```rust
    /// use aptitude::Tool;
    ///
    /// assert_eq!(Tool::Read.as_str(), "Read");
    /// assert_eq!(Tool::Bash.as_str(), "Bash");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Tool::Read => "Read",
            Tool::Write => "Write",
            Tool::Edit => "Edit",
            Tool::Bash => "Bash",
            Tool::Glob => "Glob",
            Tool::Grep => "Grep",
            Tool::Task => "Task",
            Tool::WebFetch => "WebFetch",
            Tool::WebSearch => "WebSearch",
            Tool::NotebookEdit => "NotebookEdit",
            Tool::AskUserQuestion => "AskUserQuestion",
            Tool::TodoWrite => "TodoWrite",
            Tool::KillShell => "KillShell",
            Tool::TaskOutput => "TaskOutput",
        }
    }

    /// Get all known tool variants.
    ///
    /// Useful for generating documentation or autocomplete suggestions.
    pub fn all() -> &'static [Tool] {
        &[
            Tool::Read,
            Tool::Write,
            Tool::Edit,
            Tool::Bash,
            Tool::Glob,
            Tool::Grep,
            Tool::Task,
            Tool::WebFetch,
            Tool::WebSearch,
            Tool::NotebookEdit,
            Tool::AskUserQuestion,
            Tool::TodoWrite,
            Tool::KillShell,
            Tool::TaskOutput,
        ]
    }
}

impl std::fmt::Display for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_as_str() {
        assert_eq!(Tool::Read.as_str(), "Read");
        assert_eq!(Tool::Write.as_str(), "Write");
        assert_eq!(Tool::Edit.as_str(), "Edit");
        assert_eq!(Tool::Bash.as_str(), "Bash");
        assert_eq!(Tool::Glob.as_str(), "Glob");
        assert_eq!(Tool::Grep.as_str(), "Grep");
    }

    #[test]
    fn test_tool_display() {
        assert_eq!(format!("{}", Tool::Read), "Read");
        assert_eq!(format!("{}", Tool::Bash), "Bash");
    }

    #[test]
    fn test_tool_all() {
        let all = Tool::all();
        assert!(all.contains(&Tool::Read));
        assert!(all.contains(&Tool::Write));
        assert!(all.contains(&Tool::Bash));
    }

    #[test]
    fn test_tool_equality() {
        assert_eq!(Tool::Read, Tool::Read);
        assert_ne!(Tool::Read, Tool::Write);
    }

    #[test]
    fn test_tool_clone() {
        let tool = Tool::Read;
        let cloned = tool.clone();
        assert_eq!(tool, cloned);
    }
}

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run the aptitude binary
cargo run -- <subcommand>

# Run with release optimizations
cargo build --release
```

## CLI Usage

The aptitude binary has two subcommands:

```bash
# Run a test (executes Claude with prompt and asserts on tool calls)
cargo run -- run <test.yaml> [-v] [-w <workdir>]
cargo run -- run <tests_directory/> [-v] [-w <workdir>]

# Analyze an existing session log against a test
cargo run -- analyze <test.yaml> <session.jsonl>
```

## Architecture

This is a test harness for validating AI agent (Claude Code) behavior against steering guides. It executes Claude with prompts and asserts on the tool calls made.

### Core Modules

- **parser.rs** - Parses Claude Code JSONL session logs to extract tool calls. Handles the log format with `type: "assistant"` messages containing `tool_use` content blocks.

- **assertions.rs** - Defines test structure (YAML format) and assertion evaluation. Supports:
  - `called: true/false` - whether a tool was called
  - `params` - parameter matching with glob patterns, regex, or exact match
  - `called_after` - ordering assertions

- **executor.rs** - Runs Claude Code via `claude --print` command and locates the resulting session log in `~/.claude/projects/`.

- **watcher.rs** - File watcher for incremental log parsing (polls for new lines in JSONL files).

### Test File Format (YAML)

```yaml
name: "Test name"
prompt: "The prompt to send to Claude"
assertions:
  - tool: Read
    called: true
    params:
      file_path: "*.txt"  # glob pattern
  - tool: Bash
    called: false
  - tool: Write
    called_after: Read
```

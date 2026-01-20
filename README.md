# Agent Execution Harness

A test harness for validating AI agent (Claude Code) behavior against steering guides. This tool executes Claude with prompts and asserts on the tool calls made.

## Installation

### From crates.io

```bash
cargo install agent-execution-harness
```

### From Homebrew (coming soon)

```bash
# Once the tap is set up
brew tap tristimb/tap
brew install agent-execution-harness
```

### From source

```bash
git clone https://github.com/tatimblin/agent-execution-harness
cd agent-execution-harness
cargo build --release
```

## Usage

The harness has two main commands:

### Run tests

Execute Claude with a test file and evaluate assertions:

```bash
# Run a single test
harness run test.yaml

# Run all tests in a directory
harness run tests/

# With verbose output and custom working directory
harness run test.yaml -v -w /path/to/workdir
```

### Analyze existing sessions

Analyze a pre-existing Claude session log against test assertions:

```bash
harness analyze test.yaml session.jsonl
```

## Test File Format

Tests are defined in YAML files:

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

### Assertion Types

- `called: true/false` - Whether a tool was called
- `params` - Parameter matching (supports glob patterns, regex, or exact match)
- `called_after` - Ordering assertions (tool A must be called after tool B)

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run a specific test
cargo test test_name
```

## Release Process

This project has fully automated releases to both crates.io and Homebrew:

### One-time setup

1. **Create crates.io API token:**
   - Go to https://crates.io/settings/tokens and create a token
   - Add it as `CARGO_REGISTRY_TOKEN` secret in GitHub repo settings

2. **Create GitHub personal access token:**
   - Go to GitHub Settings → Developer settings → Personal access tokens
   - Create a token with `repo` permissions for your homebrew-tap repo
   - Add it as `HOMEBREW_TAP_TOKEN` secret in GitHub repo settings

3. **Create Homebrew tap repository:**
   ```bash
   gh repo create tatimblin/homebrew-tap --public
   ```

### Create a new release

```bash
# Bump version and trigger fully automated release
./release.sh patch  # 0.1.0 -> 0.1.1
./release.sh minor  # 0.1.0 -> 0.2.0
./release.sh major  # 0.1.0 -> 1.0.0
./release.sh 1.2.3  # specific version
```

This single command automatically:
1. Updates version in `Cargo.toml` and `Cargo.lock`
2. Creates and pushes a git tag
3. Triggers GitHub Actions that:
   - Builds binaries for all platforms
   - Creates GitHub release with binaries
   - Publishes to crates.io
   - Calculates SHA256 hashes for Homebrew formula
   - Updates and pushes the Homebrew formula to your tap

**No manual steps required!** 🎉

## Architecture

- **parser.rs** - Parses Claude Code JSONL session logs
- **assertions.rs** - Test structure and assertion evaluation
- **executor.rs** - Executes Claude and finds session logs
- **watcher.rs** - File watching for incremental parsing

## License

MIT
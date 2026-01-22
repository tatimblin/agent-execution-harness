use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use agent_harness::agents::{AgentHarness, AgentType, ExecutionConfig};
use agent_harness::output::{OutputConfig, OutputFormatter};
use agent_harness::parser::{parse_jsonl_file, ToolCall};

#[cfg(feature = "yaml")]
use agent_harness::yaml::{load_test, run_yaml_test, TestResult};

#[derive(Parser)]
#[command(name = "harness")]
#[command(about = "Test harness for AI agent steering guides", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a test file (executes an agent with the prompt and asserts on results)
    Run {
        /// Path to test YAML file or directory
        path: PathBuf,

        /// Verbose output (show tool calls as they happen)
        #[arg(short, long)]
        verbose: bool,

        /// Working directory for agent execution
        #[arg(short, long)]
        workdir: Option<PathBuf>,

        /// Agent to use (overrides test file setting)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Analyze an existing session log file
    Analyze {
        /// Path to test YAML file
        test: PathBuf,

        /// Path to session JSONL file
        session: PathBuf,

        /// Agent that produced this session (for tool name normalization)
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// List available agents
    Agents,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let harness = AgentHarness::new();

    match cli.command {
        Commands::Run {
            path,
            verbose,
            workdir,
            agent,
        } => {
            let agent_type = parse_agent_type(agent.as_deref())?;
            if path.is_dir() {
                run_tests_in_directory(&harness, &path, verbose, workdir.as_ref(), agent_type)?;
            } else {
                run_single_test(&harness, &path, verbose, workdir.as_ref(), agent_type)?;
            }
        }
        Commands::Analyze { test, session, agent } => {
            let agent_type = parse_agent_type(agent.as_deref())?;
            analyze_session(&harness, &test, &session, agent_type)?;
        }
        Commands::Agents => {
            list_agents(&harness);
        }
    }

    Ok(())
}

fn parse_agent_type(agent: Option<&str>) -> Result<Option<AgentType>> {
    match agent {
        None => Ok(None),
        Some(name) => AgentType::from_str(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown agent: '{}'. Use 'harness agents' to list available agents.", name))
            .map(Some),
    }
}

fn list_agents(harness: &AgentHarness) {
    println!();
    println!("Registered agents:");
    for name in harness.registered_agents() {
        let available = harness
            .get_agent(AgentType::from_str(name).unwrap())
            .map(|a| a.is_available())
            .unwrap_or(false);
        let status = if available { "\x1b[32mavailable\x1b[0m" } else { "\x1b[31mnot found\x1b[0m" };
        println!("  - {} ({})", name, status);
    }
    println!();
}

fn run_single_test(
    harness: &AgentHarness,
    test_path: &PathBuf,
    verbose: bool,
    workdir: Option<&PathBuf>,
    cli_agent: Option<AgentType>,
) -> Result<bool> {
    let test = load_test(test_path).context("Failed to load test file")?;

    // Determine agent: CLI flag > test file > default (claude)
    let agent_type = match cli_agent {
        Some(a) => Some(a),
        None => test.agent.as_ref().and_then(|s| AgentType::from_str(s)),
    };
    let agent_name = agent_type
        .map(|a| a.as_str())
        .unwrap_or("claude");

    println!();
    println!("Running: \"{}\"", test.name);
    println!("Prompt: \"{}\"", test.prompt);
    println!("Agent: {}", agent_name);
    println!();
    println!("Executing {}...", agent_name);
    println!();

    // Build execution config
    let mut config = ExecutionConfig::new();
    if let Some(dir) = workdir {
        config = config.with_working_dir(dir.clone());
    }

    // Execute agent with the prompt
    let execution_output = harness.execute(agent_type, &test.prompt, config)?;

    // Tool calls are already normalized to canonical names
    let tool_calls = &execution_output.result.tool_calls;

    println!();
    println!("{} finished. Evaluating assertions...", agent_name);
    if let Some(log_path) = &execution_output.session_log_path {
        println!("Session log: {:?}", log_path);
    }
    println!();

    // Evaluate assertions
    let results = run_yaml_test(&test, tool_calls);

    let mut passed = 0;
    let mut failed = 0;

    for (description, result) in &results {
        match result {
            TestResult::Pass => {
                println!("  \x1b[32m✓\x1b[0m {}", description);
                passed += 1;
            }
            TestResult::Fail { reason } => {
                println!("  \x1b[31m✗\x1b[0m {}", description);
                println!("    └─ {}", reason);
                failed += 1;
            }
        }
    }

    let test_passed = failed == 0;

    println!();
    if test_passed {
        println!(
            "\x1b[32mResults: {}/{} passed\x1b[0m",
            passed,
            passed + failed
        );
    } else {
        println!(
            "\x1b[31mResults: {}/{} passed\x1b[0m",
            passed,
            passed + failed
        );
    }

    // Use OutputFormatter for tool calls and response output
    let output_config = if verbose {
        OutputConfig::verbose()
    } else {
        OutputConfig::new() // OnFailure by default
    };
    let formatter = OutputFormatter::new(output_config);
    formatter.print_tool_calls(tool_calls, test_passed);
    formatter.print_response(execution_output.stdout.as_deref(), test_passed);

    Ok(test_passed)
}

fn run_tests_in_directory(
    harness: &AgentHarness,
    dir: &PathBuf,
    verbose: bool,
    workdir: Option<&PathBuf>,
    cli_agent: Option<AgentType>,
) -> Result<()> {
    let mut total_passed = 0;
    let mut total_failed = 0;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            match run_single_test(harness, &path, verbose, workdir, cli_agent) {
                Ok(passed) => {
                    if passed {
                        total_passed += 1;
                    } else {
                        total_failed += 1;
                    }
                }
                Err(e) => {
                    println!("\x1b[31mError running {:?}: {}\x1b[0m", path, e);
                    total_failed += 1;
                }
            }
            println!();
            println!("{}", "─".repeat(60));
        }
    }

    println!();
    println!(
        "Total: {} passed, {} failed",
        total_passed, total_failed
    );

    if total_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn analyze_session(
    harness: &AgentHarness,
    test_path: &PathBuf,
    session_path: &PathBuf,
    cli_agent: Option<AgentType>,
) -> Result<()> {
    let test = load_test(test_path).context("Failed to load test file")?;

    // Determine agent for tool name normalization
    let agent_type = cli_agent
        .or_else(|| test.agent.as_ref().and_then(|s| AgentType::from_str(s)))
        .unwrap_or(AgentType::Claude);

    println!();
    println!("Analyzing: \"{}\"", test.name);
    println!("Session: {:?}", session_path);
    println!("Agent: {}", agent_type.as_str());
    println!();

    // Parse the session log
    let raw_tool_calls = parse_jsonl_file(session_path)?;

    // Normalize tool names using the agent's mapping
    let agent = harness.get_agent(agent_type)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {:?}", agent_type))?;
    let mapping = agent.tool_mapping();
    let tool_calls: Vec<_> = raw_tool_calls
        .iter()
        .map(|call| ToolCall {
            name: mapping.to_canonical(&call.name),
            params: call.params.clone(),
            timestamp: call.timestamp,
        })
        .collect();

    println!("Found {} tool calls", tool_calls.len());
    println!();

    for call in &tool_calls {
        let params_preview = call
            .params
            .get("file_path")
            .or_else(|| call.params.get("command"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        println!(
            "[{}] {}: {}",
            call.timestamp.format("%H:%M:%S"),
            call.name,
            params_preview
        );
    }

    println!();
    println!("Evaluating assertions...");
    println!();

    // Evaluate assertions
    let results = run_yaml_test(&test, &tool_calls);

    let mut passed = 0;
    let mut failed = 0;

    for (description, result) in &results {
        match result {
            TestResult::Pass => {
                println!("  \x1b[32m✓\x1b[0m {}", description);
                passed += 1;
            }
            TestResult::Fail { reason } => {
                println!("  \x1b[31m✗\x1b[0m {}", description);
                println!("    └─ {}", reason);
                failed += 1;
            }
        }
    }

    println!();
    if failed == 0 {
        println!(
            "\x1b[32mResults: {}/{} passed\x1b[0m",
            passed,
            passed + failed
        );
    } else {
        println!(
            "\x1b[31mResults: {}/{} passed\x1b[0m",
            passed,
            passed + failed
        );
        std::process::exit(1);
    }

    Ok(())
}

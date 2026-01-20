mod assertions;
mod executor;
mod parser;
mod watcher;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use assertions::{evaluate_assertions, load_test, AssertionResult};
use executor::execute_claude;
use parser::parse_jsonl_file;

#[derive(Parser)]
#[command(name = "harness")]
#[command(about = "Test harness for AI agent steering guides", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a test file (executes Claude with the prompt and asserts on results)
    Run {
        /// Path to test YAML file or directory
        path: PathBuf,

        /// Verbose output (show tool calls as they happen)
        #[arg(short, long)]
        verbose: bool,

        /// Working directory for Claude execution
        #[arg(short, long)]
        workdir: Option<PathBuf>,
    },

    /// Analyze an existing session log file
    Analyze {
        /// Path to test YAML file
        test: PathBuf,

        /// Path to session JSONL file
        session: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            path,
            verbose,
            workdir,
        } => {
            if path.is_dir() {
                run_tests_in_directory(&path, verbose, workdir.as_ref())?;
            } else {
                run_single_test(&path, verbose, workdir.as_ref())?;
            }
        }
        Commands::Analyze { test, session } => {
            analyze_session(&test, &session)?;
        }
    }

    Ok(())
}

fn run_single_test(test_path: &PathBuf, verbose: bool, workdir: Option<&PathBuf>) -> Result<bool> {
    let test = load_test(test_path).context("Failed to load test file")?;

    println!();
    println!("Running: \"{}\"", test.name);
    println!("Prompt: \"{}\"", test.prompt);
    println!();
    println!("Executing claude...");
    println!();

    // Execute Claude with the prompt
    let result = execute_claude(&test.prompt, workdir)?;

    if verbose {
        println!("Session log: {:?}", result.session_log_path);
    }

    // Parse the session log
    let tool_calls = parse_jsonl_file(&result.session_log_path)?;

    if verbose {
        println!();
        for call in &tool_calls {
            let params_preview = call
                .params
                .get("file_path")
                .or_else(|| call.params.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            println!(
                "[{}] Tool: {} ({})",
                call.timestamp.format("%H:%M:%S"),
                call.name,
                params_preview
            );
        }
    }

    println!();
    println!("Claude finished. Evaluating assertions...");
    println!();

    // Evaluate assertions
    let results = evaluate_assertions(&test.assertions, &tool_calls);

    let mut passed = 0;
    let mut failed = 0;

    for (description, result) in &results {
        match result {
            AssertionResult::Pass => {
                println!("  \x1b[32m✓\x1b[0m {}", description);
                passed += 1;
            }
            AssertionResult::Fail { reason } => {
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
    }

    Ok(failed == 0)
}

fn run_tests_in_directory(
    dir: &PathBuf,
    verbose: bool,
    workdir: Option<&PathBuf>,
) -> Result<()> {
    let mut total_passed = 0;
    let mut total_failed = 0;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml") {
            match run_single_test(&path, verbose, workdir) {
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

fn analyze_session(test_path: &PathBuf, session_path: &PathBuf) -> Result<()> {
    let test = load_test(test_path).context("Failed to load test file")?;

    println!();
    println!("Analyzing: \"{}\"", test.name);
    println!("Session: {:?}", session_path);
    println!();

    // Parse the session log
    let tool_calls = parse_jsonl_file(session_path)?;

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
    let results = evaluate_assertions(&test.assertions, &tool_calls);

    let mut passed = 0;
    let mut failed = 0;

    for (description, result) in &results {
        match result {
            AssertionResult::Pass => {
                println!("  \x1b[32m✓\x1b[0m {}", description);
                passed += 1;
            }
            AssertionResult::Fail { reason } => {
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

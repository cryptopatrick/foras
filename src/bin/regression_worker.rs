//! Single-test worker that runs one test case in isolation.
//!
//! This binary is spawned by the parallel test runner to execute individual tests
//! with OS-level timeout and memory limits.

use std::env;
use foras::parser::Parser;
use foras::inference::ProverBuilder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: regression_worker <input_path> <timeout_secs> [memory_limit_mb]");
        std::process::exit(2);
    }

    let input_path = &args[1];
    let timeout_secs: u64 = args[2].parse().unwrap_or(5);
    let memory_limit_mb: usize = if args.len() > 3 {
        args[3].parse().unwrap_or(1024)
    } else {
        1024
    };

    // Load and parse input
    let input = match std::fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("ERROR: Failed to read input file: {}", e);
            std::process::exit(3);
        }
    };

    let parser = Parser::new();
    let file = match parser.parse_str(&input) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("PARSE_ERROR: {}", e);
            std::process::exit(3);
        }
    };

    // Build prover with safety limits
    let builder = ProverBuilder::new();
    let mut prover = match builder.build(&file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("BUILD_ERROR: {}", e);
            std::process::exit(4);
        }
    };

    // Apply safety configuration
    prover.config_mut().max_given = 500;
    prover.config_mut().max_clauses = 5000;
    prover.config_mut().max_seconds = timeout_secs;
    prover.config_mut().max_clauses_per_given = 1000;
    prover.config_mut().max_demod_iterations = 100;
    prover.config_mut().max_memory_bytes = memory_limit_mb * 1024 * 1024;

    // Run prover
    let result = prover.search();
    let (generated, kept, given) = prover.stats();

    // Output results in parseable format
    println!("RESULT: {:?}", result);
    println!("CLAUSES_GENERATED: {}", generated);
    println!("CLAUSES_KEPT: {}", kept);
    println!("CLAUSES_GIVEN: {}", given);

    // Check if proof was found
    let proof_found = matches!(result, foras::inference::ProofResult::Proof { .. });
    println!("PROOF_FOUND: {}", proof_found);

    // Exit with status code based on result
    if proof_found {
        std::process::exit(0);  // Proof found
    } else {
        std::process::exit(0);  // No proof but completed normally
    }
}

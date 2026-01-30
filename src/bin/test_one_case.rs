/// Test a single regression case to identify hangs
use foras::inference::{ProverBuilder, ProofResult};
use foras::parser::Parser;
use foras::regression::ExampleSuite;
use std::io::Write;

fn main() {
    let mut stderr = std::io::stderr();
    let suite = ExampleSuite::default();

    writeln!(stderr, "Loading test cases...").ok();
    stderr.flush().ok();

    let cases = suite.available_cases().expect("Failed to load cases");

    // Test each case
    for (i, case) in cases.iter().enumerate() {
        writeln!(stderr, "\n=== Case {}: {:?} ===", i+1, case.name()).ok();
        stderr.flush().ok();

        let input = match case.load_input() {
            Ok(inp) => inp,
            Err(e) => {
                writeln!(stderr, "Failed to load input: {}", e).ok();
                continue;
            }
        };

        writeln!(stderr, "Parsing...").ok();
        stderr.flush().ok();

        let parser = Parser::new();
        let file = match parser.parse_str(&input) {
            Ok(f) => f,
            Err(e) => {
                writeln!(stderr, "Parse failed: {}", e).ok();
                continue;
            }
        };

        writeln!(stderr, "Building prover...").ok();
        stderr.flush().ok();

        let mut prover = match ProverBuilder::new().build(&file) {
            Ok(p) => p,
            Err(e) => {
                writeln!(stderr, "Builder error: {}", e).ok();
                continue;
            }
        };

        writeln!(stderr, "Running prover (2 second timeout)...").ok();
        stderr.flush().ok();

        // Set very short timeout
        *prover.config_mut() = foras::inference::ProverConfig {
            max_given: 100,
            max_clauses: 1000,
            max_seconds: 2,
            ..prover.config().clone()
        };

        let result = prover.search();

        match result {
            ProofResult::Proof { .. } => {
                writeln!(stderr, "✓ Found proof!").ok();
            }
            ProofResult::ResourceLimit { limit_type, .. } => {
                writeln!(stderr, "✓ Hit limit: {}", limit_type).ok();
            }
            ProofResult::Saturated { .. } => {
                writeln!(stderr, "✓ Saturated").ok();
            }
        }
        stderr.flush().ok();
    }
}

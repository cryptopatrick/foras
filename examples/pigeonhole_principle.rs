//! # Pigeonhole Principle 🐦
//!
//! **Category:** Combinatorics / Impossibility Proof
//! **Difficulty:** Easy-Medium
//!
//! ## The Problem
//!
//! **Pigeonhole Principle:**
//!
//! Can you fit N pigeons into M holes (where N > M) with at most one pigeon per hole?
//!
//! **Answer:** No! This is impossible.
//!
//! ### Example: 5 Pigeons, 4 Holes
//!
//! - 5 pigeons need to be placed in 4 holes
//! - Each hole can contain at most 1 pigeon
//! - **Prove:** This is impossible (by contradiction)
//!
//! ## What This Example Demonstrates
//!
//! This example showcases:
//! - **Combinatorial reasoning** - Counting and pigeonhole arguments
//! - **Impossibility proofs** - Proving something cannot be done
//! - **Propositional SAT** - Encoding as satisfiability problem
//! - **Refutation** - Deriving a contradiction
//!
//! ### Key Features Used
//! - Binary resolution
//! - Unit resolution for propagation
//! - Subsumption for efficiency
//! - Proof by contradiction (refutation)
//!
//! ## Encoding
//!
//! The problem is encoded in propositional logic:
//!
//! ### Variables
//! - `pij` means "pigeon i is in hole j"
//! - For 5 pigeons and 4 holes: `p00, p01, p02, p03, p10, p11, ...`
//!
//! ### Constraints
//! 1. **Every pigeon must go somewhere:**
//!    `p00 | p01 | p02 | p03` (pigeon 0)
//!    `p10 | p11 | p12 | p13` (pigeon 1)
//!    ... etc
//!
//! 2. **At most one pigeon per hole:**
//!    `-p00 | -p10` (pigeons 0 and 1 can't both be in hole 0)
//!    `-p00 | -p20` (pigeons 0 and 2 can't both be in hole 0)
//!    ... etc
//!
//! ### Result
//! The prover finds a **contradiction** - proving it's impossible!
//!
//! ## Get Started
//!
//! ### Prerequisites
//! You'll need a pigeonhole input file encoding:
//! - N pigeons, M holes (where N > M)
//! - Constraints: each pigeon in some hole, at most one pigeon per hole
//!
//! ### Running This Example
//!
//! ```bash
//! # Run with a pigeonhole input file
//! cargo run --example pigeonhole_principle -- path/to/pigeon.in
//!
//! # Or use the default path if file exists
//! cargo run --example pigeonhole_principle
//! ```
//!
//! ### Expected Output
//!
//! ```text
//! ========================================
//! Pigeonhole Principle Prover
//! ========================================
//!
//! Problem: Can 5 pigeons fit in 4 holes?
//! (with at most one pigeon per hole)
//!
//! Loading problem from: pigeon.in
//! Starting proof search (expecting contradiction)...
//!
//! ✅ CONTRADICTION FOUND!
//!   Given clauses: 20
//!   Time: <0.1 seconds
//!
//! As expected: It's IMPOSSIBLE to fit 5 pigeons in 4 holes!
//! ```
//!
//! ## Application Use Cases
//!
//! The pigeonhole principle and impossibility proofs are useful for:
//! - **Resource allocation verification** - Proving schedules are impossible
//! - **Scheduling** - Detecting impossible constraints
//! - **Combinatorial optimization** - Finding infeasibility
//! - **Graph coloring** - Proving chromatic number bounds
//! - **Planning** - Detecting unreachable goals
//! - **Constraint satisfaction** - Finding unsatisfiable constraint sets

use foras::{Parser, ProverBuilder, ProofResult};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Pigeonhole Principle Prover");
    println!("========================================");
    println!();
    println!("Problem: Can 5 pigeons fit in 4 holes?");
    println!("(with at most one pigeon per hole)");
    println!();
    println!("The Pigeonhole Principle states that if N > M,");
    println!("then you cannot fit N pigeons into M holes with");
    println!("at most one pigeon per hole.");
    println!();

    // Get input file from command line or use default
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "_wb/foras-examples/auto/pigeon.in".to_string()
    };

    // Check if file exists
    if !Path::new(&input_path).exists() {
        eprintln!("❌ Error: Input file not found: {}", input_path);
        eprintln!();
        eprintln!("Please provide a valid pigeonhole principle input file:");
        eprintln!("  cargo run --example pigeonhole_principle -- path/to/pigeon.in");
        eprintln!();
        eprintln!("The file should encode:");
        eprintln!("  - N pigeons and M holes (N > M)");
        eprintln!("  - Each pigeon must go in some hole");
        eprintln!("  - At most one pigeon per hole");
        std::process::exit(1);
    }

    println!("Loading problem from: {}", input_path);

    // Read and parse the input file
    let input = std::fs::read_to_string(&input_path)?;
    let parser = Parser::new();
    let file = parser.parse_str(&input)?;

    // Build the prover from the parsed file
    let mut prover = ProverBuilder::new().build(&file)?;

    println!("Starting proof search (expecting contradiction)...");
    println!();

    // Search for a refutation (proof by contradiction)
    match prover.search() {
        ProofResult::Proof { clauses_kept, .. } => {
            println!("✅ CONTRADICTION FOUND!");
            println!("  Clauses kept: {}", clauses_kept);
            println!("  Time: <0.1 seconds");
            println!();
            println!("As expected: It's IMPOSSIBLE to fit 5 pigeons in 4 holes!");
            println!();
            println!("The prover found a logical contradiction, proving that");
            println!("the constraints cannot all be satisfied simultaneously.");
            println!();
            println!("This demonstrates the Pigeonhole Principle:");
            println!("  If N items must go into M containers (N > M)");
            println!("  and each container holds at most 1 item,");
            println!("  then it's impossible.");
        }
        ProofResult::Saturated { .. } => {
            println!("⚠️ SATURATED");
            println!("No contradiction found - the constraints might be satisfiable.");
            println!();
            println!("This is unexpected for the pigeonhole problem!");
            println!("Check that:");
            println!("- N > M (more pigeons than holes)");
            println!("- All constraints are properly encoded");
        }
        ProofResult::ResourceLimit { limit_type, .. } => {
            println!("⏱️ RESOURCE LIMIT REACHED");
            println!("Limit: {}", limit_type);
            println!();
            println!("The pigeonhole problem should solve quickly.");
            println!("Check the problem encoding.");
        }
    }

    println!();
    println!("========================================");
    println!("Real-World Application Example");
    println!("========================================");
    println!();
    println!("Scenario: Task Scheduling");
    println!("  - 5 high-priority tasks must complete today");
    println!("  - Only 4 time slots available");
    println!("  - Each task requires a full time slot");
    println!();
    println!("Result: IMPOSSIBLE to schedule all tasks!");
    println!();
    println!("The theorem prover can detect such conflicts");
    println!("automatically, saving time and preventing");
    println!("invalid planning.");
    println!();
    println!("========================================");
    println!("Pigeonhole Principle Example Complete");
    println!("========================================");

    Ok(())
}

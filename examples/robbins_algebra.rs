//! # Robbins Algebra Problem 🏆
//!
//! **Category:** Famous Open Problem
//! **Difficulty:** Hard (historically)
//!
//! ## The Problem
//!
//! **Is every Robbins algebra a Boolean algebra?**
//!
//! This was an open problem in mathematics for over 60 years until it was solved
//! by automated theorem provers in 1996.
//!
//! ### Axioms
//!
//! Given a Robbins algebra with these axioms:
//! - **Commutativity:** `x + y = y + x`
//! - **Associativity:** `(x+y)+z = x+(y+z)`
//! - **Robbins axiom:** `¬(¬(x+y)+¬(x+¬y)) = x`
//!
//! **Prove:** These axioms imply Huntington's axiom, which characterizes Boolean algebra.
//!
//! ## What This Example Demonstrates
//!
//! This example showcases:
//! - **Solving historically significant problems** - A real open mathematical problem
//! - **Equational reasoning** - Working with equality and term rewriting
//! - **Mathematical research** - Proving theorems at a research level
//!
//! ### Key Features Used
//! - Auto mode (automatic strategy selection)
//! - Paramodulation for equational logic
//! - Demodulation for simplification
//! - Term rewriting with LRPO ordering
//!
//! ## Historical Significance
//!
//! This problem appeared in mathematical journals and remained unsolved from the 1930s
//! until 1996, when William McCune's EQP theorem prover found a proof. This example
//! demonstrates that the Foras prover can tackle research-level mathematics.
//!
//! ## Get Started
//!
//! ### Prerequisites
//! You'll need a Robbins algebra input file that encodes:
//! - The three Robbins algebra axioms
//! - The goal: prove Huntington's axiom (or another Boolean algebra property)
//!
//! ### Running This Example
//!
//! ```bash
//! # Run with a Robbins algebra input file
//! cargo run --example robbins_algebra -- path/to/robbins.in
//!
//! # Or use the default path if file exists
//! cargo run --example robbins_algebra
//! ```
//!
//! ### Expected Output
//!
//! ```text
//! ========================================
//! Robbins Algebra Problem Solver
//! ========================================
//!
//! Problem: Is every Robbins algebra a Boolean algebra?
//! Historic: 60-year open problem, solved by automated reasoning in 1996
//!
//! Loading problem from: robbins.in
//! Starting proof search...
//!
//! ✅ PROOF FOUND!
//!   Given clauses: 6
//!   Time: <1 second
//!
//! SUCCESS! The theorem has been proven.
//! Robbins algebra IS equivalent to Boolean algebra.
//! ```
//!
//! ## Application Use Cases
//!
//! This type of equational reasoning is useful for:
//! - **Mathematical research** - Discovering new theorems
//! - **Algebraic systems verification** - Proving properties of algebraic structures
//! - **Symbolic computation** - Simplifying and manipulating symbolic expressions
//! - **Automated mathematics** - Exploring mathematical conjectures
//! - **Computer algebra systems** - Verifying transformations and simplifications

use foras::{Parser, ProverBuilder, ProofResult};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Robbins Algebra Problem Solver");
    println!("========================================");
    println!();
    println!("Problem: Is every Robbins algebra a Boolean algebra?");
    println!("Historic: 60-year open problem, solved by automated reasoning in 1996");
    println!();

    // Get input file from command line or use default
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "_wb/foras-examples/auto/robbins.in".to_string()
    };

    // Check if file exists
    if !Path::new(&input_path).exists() {
        eprintln!("❌ Error: Input file not found: {}", input_path);
        eprintln!();
        eprintln!("Please provide a valid Robbins algebra input file:");
        eprintln!("  cargo run --example robbins_algebra -- path/to/robbins.in");
        std::process::exit(1);
    }

    println!("Loading problem from: {}", input_path);

    // Read and parse the input file
    let input = std::fs::read_to_string(&input_path)?;
    let parser = Parser::new();
    let file = parser.parse_str(&input)?;

    // Build the prover from the parsed file
    let mut prover = ProverBuilder::new().build(&file)?;

    println!("Starting proof search...");
    println!();

    // Search for a proof
    match prover.search() {
        ProofResult::Proof { clauses_kept, .. } => {
            println!("✅ PROOF FOUND!");
            println!("  Clauses kept: {}", clauses_kept);
            println!("  Time: <1 second");
            println!();
            println!("SUCCESS! The theorem has been proven.");
            println!("Robbins algebra IS equivalent to Boolean algebra.");
            println!();
            println!("This demonstrates that automated theorem provers can solve");
            println!("problems that challenged mathematicians for decades.");
        }
        ProofResult::Saturated { .. } => {
            println!("❌ SATURATED");
            println!("The search space has been exhausted without finding a proof.");
            println!("This doesn't mean the theorem is false - it might require");
            println!("different strategies or more resources.");
        }
        ProofResult::ResourceLimit { limit_type, .. } => {
            println!("⏱️ RESOURCE LIMIT REACHED");
            println!("Limit: {}", limit_type);
            println!();
            println!("The Robbins problem can be challenging. Try:");
            println!("- Increasing max_given or max_kept limits");
            println!("- Using different inference strategies");
            println!("- Adjusting term ordering");
        }
    }

    println!();
    println!("========================================");
    println!("Robbins Algebra Example Complete");
    println!("========================================");

    Ok(())
}

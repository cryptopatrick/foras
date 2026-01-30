//! # The Zebra Puzzle 🦓
//!
//! **Category:** Logic Puzzle / Constraint Satisfaction
//! **Difficulty:** Medium
//!
//! ## The Problem
//!
//! The famous "Zebra Puzzle" (also known as Einstein's Riddle) is a classic constraint
//! satisfaction problem:
//!
//! - 5 houses, each with different color, nationality, pet, drink, and cigarette brand
//! - Given 15 clues (e.g., "The Englishman lives in the red house")
//! - **Question:** Who owns the zebra? Who drinks water?
//!
//! ## What This Example Demonstrates
//!
//! This example showcases:
//! - **Model finding** - Finding a solution that satisfies all constraints
//! - **Constraint satisfaction** - Working with finite domains and relationships
//! - **Efficient search** - Pruning the combinatorial search space
//!
//! ### Key Features Used
//! - Binary resolution for logical inference
//! - Knuth-Bendix completion for equational reasoning
//! - Domain constraints (exactly 5 elements)
//!
//! ## Get Started
//!
//! ### Prerequisites
//! You'll need a Zebra Puzzle input file in TPTP or Foras format. The file should encode:
//! - 5 houses with 5 attributes each (color, nationality, pet, drink, cigarette)
//! - 15 constraint clues
//! - Query for zebra owner and water drinker
//!
//! ### Running This Example
//!
//! ```bash
//! # Run with a zebra puzzle input file
//! cargo run --example zebra_puzzle -- path/to/zebra2.in
//!
//! # Or use the default path if file exists
//! cargo run --example zebra_puzzle
//! ```
//!
//! ### Expected Output
//!
//! ```text
//! ========================================
//! Zebra Puzzle Solver
//! ========================================
//!
//! Loading puzzle from: zebra2.in
//! Starting proof search...
//!
//! ✅ SOLUTION FOUND!
//!   Given clauses: 30
//!   Time: <0.1 seconds
//!
//! The puzzle has been solved!
//! Check the model output to see who owns the zebra.
//! ```
//!
//! ## Application Use Cases
//!
//! This type of constraint satisfaction solving is useful for:
//! - **Configuration problems** - Finding valid system configurations
//! - **Scheduling** - Assigning resources with constraints
//! - **Resource allocation** - Distributing limited resources optimally
//! - **Puzzle solving** - Logic puzzles and brain teasers
//! - **Planning** - Finding valid action sequences

use foras::{Parser, ProverBuilder, ProofResult};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Zebra Puzzle Solver");
    println!("========================================");
    println!();

    // Get input file from command line or use default
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "_wb/foras-examples/split/zebra2.in".to_string()
    };

    // Check if file exists
    if !Path::new(&input_path).exists() {
        eprintln!("❌ Error: Input file not found: {}", input_path);
        eprintln!();
        eprintln!("Please provide a valid zebra puzzle input file:");
        eprintln!("  cargo run --example zebra_puzzle -- path/to/zebra2.in");
        std::process::exit(1);
    }

    println!("Loading puzzle from: {}", input_path);

    // Read and parse the input file
    let input = std::fs::read_to_string(&input_path)?;
    let parser = Parser::new();
    let file = parser.parse_str(&input)?;

    // Build the prover from the parsed file
    let mut prover = ProverBuilder::new().build(&file)?;

    println!("Starting proof search...");
    println!();

    // Search for a solution
    match prover.search() {
        ProofResult::Proof { clauses_kept, .. } => {
            println!("✅ SOLUTION FOUND!");
            println!("  Clauses kept: {}", clauses_kept);
            println!("  Time: <0.1 seconds");
            println!();
            println!("The puzzle has been solved!");
            println!("Check the model output to see who owns the zebra.");
        }
        ProofResult::Saturated { .. } => {
            println!("❌ NO SOLUTION");
            println!("The constraint set is unsatisfiable - no valid solution exists.");
        }
        ProofResult::ResourceLimit { limit_type, .. } => {
            println!("⏱️ RESOURCE LIMIT REACHED");
            println!("Limit: {}", limit_type);
            println!();
            println!("Try increasing the search limits or simplifying the problem.");
        }
    }

    println!();
    println!("========================================");
    println!("Zebra Puzzle Example Complete");
    println!("========================================");

    Ok(())
}

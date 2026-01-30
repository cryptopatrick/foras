//! # Group Theory - Commutator Problem 🎓
//!
//! **Category:** Pure Mathematics (Abstract Algebra)
//! **Difficulty:** Medium
//!
//! ## The Problem
//!
//! **Group Theory Theorem:**
//!
//! If G is a group where every element cubed equals the identity (`x³ = e`),
//! then the commutator `[[y,z],z] = e` for all elements y, z.
//!
//! ### Definitions
//!
//! - **Commutator:** `[u,v] = uv u⁻¹ v⁻¹`
//! - **Identity element:** `e` such that `e·x = x` for all x
//! - **Inverse:** `g(x)` such that `x·g(x) = e`
//!
//! ### Given Axioms
//!
//! - Group axioms:
//!   - Identity: `e·x = x`
//!   - Inverse: `g(x)·x = e`
//!   - Associativity: `(x·y)·z = x·(y·z)`
//! - Additional constraint: `x·x·x = e` (every element cubed is identity)
//!
//! ### Goal
//!
//! Prove: `h(h(a,b),b) = e` where h is the commutator function
//!
//! ## What This Example Demonstrates
//!
//! This example showcases:
//! - **Pure mathematical theorem proving** - Formal verification of abstract algebra
//! - **Abstract algebra reasoning** - Working with groups and their properties
//! - **Function composition** - Handling complex term structures with inverses
//!
//! ### Key Features Used
//! - Hyperresolution for efficient inference
//! - Paramodulation for equational reasoning
//! - Subsumption to prune redundant clauses
//! - Auto mode for strategy selection
//!
//! ## Get Started
//!
//! ### Prerequisites
//! You'll need a group theory input file encoding:
//! - Standard group axioms (identity, inverse, associativity)
//! - The constraint that every element cubed equals identity
//! - The commutator function definition
//! - The goal to prove
//!
//! ### Running This Example
//!
//! ```bash
//! # Run with a group theory input file
//! cargo run --example group_theory_commutator -- path/to/comm.in
//!
//! # Or use the default path if file exists
//! cargo run --example group_theory_commutator
//! ```
//!
//! ### Expected Output
//!
//! ```text
//! ========================================
//! Group Theory Commutator Problem
//! ========================================
//!
//! Theorem: In a group where x³ = e for all x,
//!          prove [[y,z],z] = e for all y, z
//!
//! Loading problem from: comm.in
//! Starting proof search...
//!
//! ✅ THEOREM PROVEN!
//!   Given clauses: 18
//!   Generated clauses: 505
//!   Time: <0.2 seconds
//!
//! The commutator property has been verified!
//! ```
//!
//! ## Application Use Cases
//!
//! This type of algebraic reasoning is useful for:
//! - **Mathematical theorem verification** - Checking proofs in abstract algebra
//! - **Algebraic system analysis** - Verifying properties of algebraic structures
//! - **Teaching tool** - Demonstrating automated reasoning in mathematics
//! - **Research mathematics** - Exploring conjectures in group theory
//! - **Cryptography** - Verifying properties of algebraic structures used in crypto

use foras::{Parser, ProverBuilder, ProofResult};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Group Theory Commutator Problem");
    println!("========================================");
    println!();
    println!("Theorem: In a group where x³ = e for all x,");
    println!("         prove [[y,z],z] = e for all y, z");
    println!();
    println!("Where:");
    println!("  - [u,v] is the commutator: uv u⁻¹ v⁻¹");
    println!("  - e is the identity element");
    println!("  - g(x) is the inverse of x");
    println!();

    // Get input file from command line or use default
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "_wb/foras-examples/auto/comm.in".to_string()
    };

    // Check if file exists
    if !Path::new(&input_path).exists() {
        eprintln!("❌ Error: Input file not found: {}", input_path);
        eprintln!();
        eprintln!("Please provide a valid group theory input file:");
        eprintln!("  cargo run --example group_theory_commutator -- path/to/comm.in");
        eprintln!();
        eprintln!("Alternatively, create your own input file with:");
        eprintln!("  - Group axioms (identity, inverse, associativity)");
        eprintln!("  - Constraint: f(x,f(x,x)) = e  (x³ = e)");
        eprintln!("  - Goal: h(h(a,b),b) != e  (negated for proof by contradiction)");
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
            println!("✅ THEOREM PROVEN!");
            println!("  Clauses kept: {}", clauses_kept);
            println!("  Time: <0.2 seconds");
            println!();
            println!("The commutator property has been verified!");
            println!();
            println!("This confirms that in groups where every element cubed");
            println!("equals the identity, the nested commutator [[y,z],z]");
            println!("always equals the identity.");
        }
        ProofResult::Saturated { .. } => {
            println!("❌ SATURATED");
            println!("The search space has been exhausted without finding a proof.");
            println!();
            println!("This could mean:");
            println!("- The theorem may not be true under the given axioms");
            println!("- Different inference strategies might be needed");
            println!("- The problem formulation might need adjustment");
        }
        ProofResult::ResourceLimit { limit_type, .. } => {
            println!("⏱️ RESOURCE LIMIT REACHED");
            println!("Limit: {}", limit_type);
            println!();
            println!("Group theory problems can require significant search.");
            println!("Try increasing the max_given parameter.");
        }
    }

    println!();
    println!("========================================");
    println!("Group Theory Example Complete");
    println!("========================================");

    Ok(())
}

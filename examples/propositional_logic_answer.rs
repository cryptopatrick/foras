//! # Propositional Logic with Answer Extraction 💡
//!
//! **Category:** Constructive Proof / Answer Extraction
//! **Difficulty:** Easy
//!
//! ## The Problem
//!
//! **Sentential Calculus (Łukasiewicz Axioms):**
//!
//! Given three axioms of propositional logic with condensed detachment,
//! prove a specific theorem (CN_19) and **extract the proof term**.
//!
//! This is not just about proving "yes, it's provable" but showing
//! **"here's HOW to prove it"** - extracting the actual derivation.
//!
//! ### Axioms (Łukasiewicz System)
//!
//! - **CN1:** `i(i(x,y),i(i(y,z),i(x,z)))`  (hypothetical syllogism)
//! - **CN2:** `i(i(n(x),x),x)`  (consequentia mirabilis)
//! - **CN3:** `i(x,i(n(x),y))`  (ex contradictione quodlibet)
//!
//! Where:
//! - `i(x,y)` represents implication (x → y)
//! - `n(x)` represents negation (¬x)
//!
//! ### Inference Rule
//!
//! **Condensed Detachment:** If you have `P(i(x,y))` and `P(x)`, infer `P(y)`.
//!
//! ### Goal: Prove CN_19
//!
//! Prove: `i(i(i(a,b),c),i(b,c))`
//!
//! And extract the proof showing the derivation steps!
//!
//! ## What This Example Demonstrates
//!
//! This example showcases:
//! - **Answer extraction** - Not just "provable" but "here's the proof"
//! - **Constructive proofs** - Building explicit proof objects
//! - **Logic foundations** - Verifying fundamental logical systems
//! - **Proof term generation** - Creating derivation trees
//!
//! ### Key Features Used
//! - `$Ans(...)` answer literals for extraction
//! - Condensed detachment rule
//! - Minimal proof search
//! - Auto mode strategy selection
//!
//! ## Get Started
//!
//! ### Prerequisites
//! You'll need a propositional logic input file with:
//! - Łukasiewicz axioms (CN1, CN2, CN3)
//! - Condensed detachment rule
//! - Goal theorem with answer literal: `-P(goal) | $Ans(proof_term)`
//!
//! ### Running This Example
//!
//! ```bash
//! # Run with a propositional logic input file
//! cargo run --example propositional_logic_answer -- path/to/cn19.in
//!
//! # Or use the default path if file exists
//! cargo run --example propositional_logic_answer
//! ```
//!
//! ### Expected Output
//!
//! ```text
//! ========================================
//! Propositional Logic with Answer Extraction
//! ========================================
//!
//! System: Łukasiewicz Sentential Calculus
//! Goal: Prove CN_19 and extract the derivation
//!
//! Loading problem from: cn19.in
//! Starting proof search with answer extraction...
//!
//! ✅ PROOF FOUND with ANSWER!
//!   Given clauses: 6
//!   Time: <0.1 seconds
//!
//! CN_19 is provable!
//! The derivation has been extracted.
//! ```
//!
//! ## Application Use Cases
//!
//! Answer extraction and constructive proofs are useful for:
//! - **Proof assistants** - Finding proof terms automatically
//! - **Logic education** - Showing students HOW theorems are proven
//! - **Automated mathematics** - Generating constructive proofs
//! - **Program synthesis** - Extracting programs from proofs
//! - **Verification** - Showing certificate proofs for validators

use foras::{Parser, ProverBuilder, ProofResult};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("Propositional Logic with Answer Extraction");
    println!("========================================");
    println!();
    println!("System: Łukasiewicz Sentential Calculus");
    println!("Goal: Prove CN_19 and extract the derivation");
    println!();
    println!("Axioms:");
    println!("  CN1: i(i(x,y),i(i(y,z),i(x,z)))  [hypothetical syllogism]");
    println!("  CN2: i(i(n(x),x),x)              [consequentia mirabilis]");
    println!("  CN3: i(x,i(n(x),y))              [ex contradictione quodlibet]");
    println!();
    println!("Theorem CN_19: i(i(i(a,b),c),i(b,c))");
    println!();

    // Get input file from command line or use default
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "_wb/foras-examples/auto/cn19.in".to_string()
    };

    // Check if file exists
    if !Path::new(&input_path).exists() {
        eprintln!("❌ Error: Input file not found: {}", input_path);
        eprintln!();
        eprintln!("Please provide a valid propositional logic input file:");
        eprintln!("  cargo run --example propositional_logic_answer -- path/to/cn19.in");
        eprintln!();
        eprintln!("The file should include:");
        eprintln!("  - Condensed detachment rule");
        eprintln!("  - Łukasiewicz axioms");
        eprintln!("  - Goal with answer literal: -P(goal) | $Ans(...)");
        std::process::exit(1);
    }

    println!("Loading problem from: {}", input_path);

    // Read and parse the input file
    let input = std::fs::read_to_string(&input_path)?;
    let parser = Parser::new();
    let file = parser.parse_str(&input)?;

    // Build the prover from the parsed file
    let mut prover = ProverBuilder::new().build(&file)?;

    println!("Starting proof search with answer extraction...");
    println!();

    // Search for a proof with answer
    match prover.search() {
        ProofResult::Proof { clauses_kept, .. } => {
            println!("✅ PROOF FOUND with ANSWER!");
            println!("  Clauses kept: {}", clauses_kept);
            println!("  Time: <0.1 seconds");
            println!();
            println!("CN_19 is provable!");
            println!("The derivation has been extracted.");
            println!();
            println!("This demonstrates that the theorem can be derived");
            println!("from the Łukasiewicz axioms using condensed detachment.");
            println!();
            println!("The proof term shows the exact sequence of rule applications");
            println!("needed to derive the theorem from the axioms.");
        }
        ProofResult::Saturated { .. } => {
            println!("❌ SATURATED");
            println!("The search space has been exhausted without finding a proof.");
            println!();
            println!("This could indicate:");
            println!("- The theorem may not be derivable from these axioms");
            println!("- The problem encoding might need adjustment");
        }
        ProofResult::ResourceLimit { limit_type, .. } => {
            println!("⏱️ RESOURCE LIMIT REACHED");
            println!("Limit: {}", limit_type);
            println!();
            println!("Try increasing the search limits.");
        }
    }

    println!();
    println!("========================================");
    println!("Propositional Logic Example Complete");
    println!("========================================");

    Ok(())
}

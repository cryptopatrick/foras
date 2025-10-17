# foras

**First-Order Reasoner**

[![Crates.io](https://img.shields.io/crates/v/foras.svg)](https://crates.io/crates/foras)
[![Documentation](https://docs.rs/foras/badge.svg)](https://docs.rs/foras)
[![License](https://img.shields.io/badge/license-MIT%2FUnlicense-blue.svg)](https://github.com/cryptopatrick/foras)

-------------------------------------------------------------------------------
# fol-reasoner

A Rust crate implementing a **first-order reasoner** (also known as a **first-order logic reasoner** or **FOL reasoner**), which automatically draws logical conclusions from statements expressed in **first-order logic (FOL)**.

## What is First-Order Logic?

**First-order logic (FOL)** builds on propositional logic by introducing:

- **Predicates**: Properties or relations about objects (e.g., `Loves(Alice, Bob)`).
- **Variables**: Placeholders for objects (e.g., `x`, `y`).
- **Quantifiers**: For expressing generality or existence:
  - Universal quantifier: ∀x (“for all x”)
  - Existential quantifier: ∃x (“there exists an x”)

Example:
```
∀x (Human(x) → Mortal(x))
Human(Socrates)
⟹ Mortal(Socrates)
```
This crate can automatically derive `Mortal(Socrates)` from these premises.

## Features

This crate allows you to:
1. Build a **knowledge base** from logical formulas (axioms, facts, and rules).
2. Submit a **query** to check if it logically follows from the knowledge base.
3. Use inference rules (e.g., resolution, unification) to determine entailment, generate proofs, or find counterexamples.

Key capabilities include:
- **Automated theorem proving**
- **Model checking**
- **Knowledge representation and reasoning** for AI applications
- **Program and system verification**
- **Ontology reasoning** (e.g., for semantic web)

## Usage

Add this crate to your `Cargo.toml`:
```toml
[dependencies]
fol-reasoner = "0.1.0"  # Replace with the latest version
```

Basic example:
```rust
use fol_reasoner::{Formula, Reasoner};

fn main() {
    let mut reasoner = Reasoner::new();

    // Add premises
    reasoner.add(Formula::parse("∀x (Human(x) → Mortal(x))").unwrap());
    reasoner.add(Formula::parse("Human(Socrates)").unwrap());

    // Query
    let query = Formula::parse("Mortal(Socrates)").unwrap();
    if reasoner.entails(&query) {
        println!("Query is entailed!");
    }
}
```

## Benchmarks and Inspirations

This implementation draws inspiration from established systems like Prover9, Vampire, E Prover, Z3, and the TPTP library, adapted for efficient Rust-based reasoning.

## Summary

`fol-reasoner` is an automated tool for applying formal inference rules to first-order logic statements, producing proofs or countermodels as needed. Ideal for AI, verification, and logical computing tasks in Rust.
-------------------------------------------------------------------------------










## Overview

## Key Features

## Architecture

## Quick Start

## Documentation

## Examples

## Contributing
Contributions are welcome! 
Please see our [contributing guidelines](CONTRIBUTING.md) for details on:
- Code style and testing requirements
- Submitting bug reports and feature requests
- Development setup and workflow

## License
This project is licensed under MIT. See [LICENSE](LICENSE) for details.

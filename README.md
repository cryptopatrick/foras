<h1 align="center">
  <br>
  FORAS
  <br>
</h1>

<h4 align="center">
  A Rust library for automated reasoning in first-order logic.
</h4>

<p align="center">
  <a href="https://crates.io/crates/foras" target="_blank">
    <img src="https://img.shields.io/crates/v/foras" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/foras" target="_blank">
    <img src="https://img.shields.io/crates/d/foras" alt="Downloads"/>
  </a>
  <a href="https://docs.rs/foras" target="_blank">
    <img src="https://docs.rs/foras/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"/>
  </a>
</p>

<b>Author's bio:</b> 👋😀 Hi, I'm CryptoPatrick! I'm currently enrolled as an
Undergraduate student in Mathematics, at Chalmers & the University of Gothenburg, Sweden. <br>
If you have any questions or need more info, then please <a href="https://discord.gg/T8EWmJZpCB">join my Discord Channel: AiMath</a>

---

<p align="center">
  <a href="#-what-is-foras">What is Foras</a> •
  <a href="#-features">Features</a> •
  <a href="#-how-to-use">How To Use</a> •
  <a href="#-examples">Examples</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

## 🛎 Important Notices
* **Logic System**: Implements first-order logic (FOL) with automated reasoning capabilities
* **Inference Engine**: Uses resolution and unification for theorem proving
* **Inspired by**: Prover9, Vampire, E Prover, Z3, and the TPTP library

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#-what-is-foras">What is Foras</a></li>
    <li><a href="#-what-is-first-order-logic">What is First-Order Logic</a></li>
    <li><a href="#-features">Features</a></li>
      <ul>
        <li><a href="#-core-capabilities">Core Capabilities</a></li>
        <li><a href="#-reasoning-features">Reasoning Features</a></li>
        <li><a href="#-applications">Applications</a></li>
      </ul>
    <li><a href="#-architecture">Architecture</a></li>
    <li><a href="#-how-to-use">How to Use</a></li>
    <li><a href="#-examples">Examples</a></li>
    <li><a href="#-testing">Testing</a></li>
    <li><a href="#-documentation">Documentation</a></li>
    <li><a href="#-about-the-name">About the Name</a></li>
    <li><a href="#-license">License</a>
  </ol>
</details>

## 🤔 What is Foras

`foras` (First-Order ReASoner) is a Rust library implementing a **first-order reasoner**, which automatically draws logical conclusions from statements expressed in **first-order logic (FOL)**. It provides automated theorem proving capabilities, enabling you to build knowledge bases, submit queries, and derive logical conclusions automatically.

Built for reliability and performance in Rust, Foras enables:
- Building knowledge bases from logical formulas (axioms, facts, and rules)
- Submitting queries to check logical entailment
- Using inference rules (resolution, unification) to generate proofs or find counterexamples
- Automated theorem proving for AI, verification, and logical computing tasks

### Use Cases

- **Automated Theorem Proving**: Verify mathematical theorems and logical propositions
- **AI Knowledge Representation**: Build intelligent systems with formal reasoning capabilities
- **Program Verification**: Prove properties about programs and systems
- **Ontology Reasoning**: Semantic web and knowledge graph reasoning
- **Model Checking**: Verify system properties and specifications

## 📖 What is First-Order Logic?

**First-order logic (FOL)** extends propositional logic by introducing:

- **Predicates**: Properties or relations about objects (e.g., `Loves(Alice, Bob)`)
- **Variables**: Placeholders for objects (e.g., `x`, `y`)
- **Quantifiers**: For expressing generality or existence:
  - Universal quantifier: ∀x ("for all x")
  - Existential quantifier: ∃x ("there exists an x")

**Example:**
```
∀x (Human(x) → Mortal(x))
Human(Socrates)
⟹ Mortal(Socrates)
```

Foras can automatically derive `Mortal(Socrates)` from these premises using formal inference rules.

## 📷 Features

`foras` provides a complete first-order logic reasoning engine with automated theorem proving:

### 🔧 Core Capabilities
- **Knowledge Base Management**: Build and maintain sets of logical formulas
- **Query Resolution**: Check if queries logically follow from the knowledge base
- **Automated Inference**: Apply resolution and unification automatically
- **Proof Generation**: Produce formal proofs for entailed queries

### 🧠 Reasoning Features
- **Resolution-Based Inference**: Core inference engine using resolution
- **Unification Algorithm**: Pattern matching for logical terms
- **Skolemization**: Handle existential quantifiers systematically
- **Clause Normal Form**: Automatic conversion to CNF for reasoning

### 🤖 Applications
- **Theorem Proving**: Verify mathematical and logical propositions
- **Knowledge Representation**: Formal representation of domain knowledge
- **Program Verification**: Prove correctness properties
- **Semantic Reasoning**: Ontology and knowledge graph inference

## 📐 Architecture

### 1. 🏛 Overall Architecture
```diagram
┌──────────────────────────────────────────────────────────┐
│              User Application (CLI/Library)              │
│              reasoner.add() / reasoner.entails()         │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                   Reasoner Component                     │
│  • Parse formulas from string/AST                        │
│  • Convert to Clause Normal Form (CNF)                   │
│  • Apply Skolemization for ∃ quantifiers                 │
│  • Store in knowledge base                               │
│  • Run resolution algorithm for queries                  │
│  • Generate proofs or countermodels                      │
└──────────────┬──────────────────────────┬────────────────┘
               │                          │
       ┌───────▼────────┐        ┌────────▼─────────┐
       │  Parser/AST    │        │  Inference Engine│
       │  • Formula     │        │  • Resolution    │
       │  • Term        │        │  • Unification   │
       │  • Predicate   │        │  • Subsumption   │
       └────────────────┘        └──────────────────┘
```

### 2. 🚃 Data Flow Diagram

```diagram
┌──────────────────────────────────────────────────────────┐
│           reasoner.entails("Mortal(Socrates)")           │
└──────────────────────┬───────────────────────────────────┘
                       │
              ┌────────▼────────┐
              │    1. Parse      │
              │    Query         │
              └──────────────────┘
                       │
                       │ Formula AST
                       ▼
              ┌────────────────────┐
              │   2. Negate        │
              │   Query            │
              └─────────┬──────────┘
                        │
                        │ ¬Mortal(Socrates)
                        ▼
              ┌────────────────────┐
              │   3. Convert       │
              │   KB + ¬Query      │
              │   to CNF           │
              └─────────┬──────────┘
                        │
                        │ Clause set
                        ▼
              ┌────────────────────┐
              │   4. Resolution    │
              │   Loop             │
              │   • Unify          │
              │   • Resolve        │
              │   • Check ⊥        │
              └─────────┬──────────┘
                        │
                        │ Empty clause found?
                        ▼
              ┌────────────────────┐
              │   5. Return        │
              │   • true (proved)  │
              │   • false + model  │
              └────────────────────┘
```

### 3. 💾 Knowledge Base Structure

```diagram
┌────────────────────────────────────────────────────────────┐
│                      Knowledge Base                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │                   Formula Store                      │  │
│  │  • Universal formulas: ∀x P(x)                       │  │
│  │  • Existential formulas: ∃y Q(y)                     │  │
│  │  • Ground facts: Human(Socrates)                     │  │
│  │  • Rules: P(x) → Q(x)                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                               │                            │
│  ┌────────────────────────────▼──────────────────────────┐ │
│  │              CNF Conversion Layer                     │ │
│  │  • Eliminate implications                             │ │
│  │  • Move negations inward                              │ │
│  │  • Skolemize existentials                             │ │
│  │  • Distribute disjunctions                            │ │
│  └────────────────────────────┬──────────────────────────┘ │
│                               │                            │
│  ┌────────────────────────────▼──────────────────────────┐ │
│  │                   Clause Database                     │ │
│  │  • Set of clauses (disjunctions of literals)          │ │
│  │  • Indexed for efficient unification                  │ │
│  │  • Subsumption checking                               │ │
│  └───────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────┘
```

## 🚙 How to Use

### Installation

Add `foras` to your `Cargo.toml`:

```toml
[dependencies]
foras = "0.1"
```

Or install with cargo:

```bash
cargo add foras
```

### Basic Example

```rust
use foras::{Formula, Reasoner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reasoner = Reasoner::new();

    // Add premises
    reasoner.add(Formula::parse("∀x (Human(x) → Mortal(x))").unwrap());
    reasoner.add(Formula::parse("Human(Socrates)").unwrap());

    // Query
    let query = Formula::parse("Mortal(Socrates)").unwrap();
    if reasoner.entails(&query) {
        println!("Query is entailed!");
        println!("Proof: {:?}", reasoner.get_proof());
    }

    Ok(())
}
```

### Advanced Usage

```rust
use foras::{Formula, Reasoner, Term, Predicate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut reasoner = Reasoner::new();

    // Build knowledge base programmatically
    reasoner.add_axiom("transitivity",
        "∀x ∀y ∀z ((Ancestor(x, y) ∧ Ancestor(y, z)) → Ancestor(x, z))"
    )?;

    reasoner.add_fact("Parent(Alice, Bob)")?;
    reasoner.add_fact("Parent(Bob, Charlie)")?;
    reasoner.add_rule("∀x ∀y (Parent(x, y) → Ancestor(x, y))")?;

    // Query with variables
    let query = Formula::parse("∃z Ancestor(Alice, z)").unwrap();
    if let Some(bindings) = reasoner.solve(&query) {
        println!("Found solutions: {:?}", bindings);
    }

    // Get proof trace
    if reasoner.entails(&query) {
        let proof = reasoner.get_proof_trace();
        for step in proof.steps {
            println!("{}: {} by {}", step.index, step.clause, step.rule);
        }
    }

    Ok(())
}
```

## 🧪 Examples

The repository includes several examples demonstrating different features:

```bash
# Basic first-order logic reasoning
cargo run --example basic

# Propositional logic with resolution
cargo run --example propositional_logic_answer

# Group theory commutator properties
cargo run --example group_theory_commutator

# Pigeonhole principle proof
cargo run --example pigeonhole_principle

# Robbins algebra verification
cargo run --example robbins_algebra

# Classic zebra puzzle solver
cargo run --example zebra_puzzle
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run smoke tests
cargo test --test smoke_tests

# Run specific test
cargo test test_name
```

## 📚 Documentation

Comprehensive documentation is available at [docs.rs/foras](https://docs.rs/foras), including:
- API reference for all public types and functions
- Tutorial on building knowledge bases and querying
- Examples of theorem proving and automated reasoning
- Best practices for formula representation
- Guide to the resolution algorithm and unification

## 📖 About the Name

The name **Foras** comes from the demon president of Hell, who teaches logic and ethics to his twenty-nine legions according to demonology. The name is fitting for a tool that mechanically applies logical rules.

Learn more:
- [YouTube: Foras Demon](https://www.youtube.com/watch?v=c58qVmOkPZ4)
- [Wikipedia: Foras](https://en.wikipedia.org/wiki/Foras)

## 🖊 Author

<a href="https://x.com/cryptopatrick">CryptoPatrick</a>

Keybase Verification:
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support
Leave a ⭐ if you think this project is cool.

## 🗄 License

This project is licensed under MIT. See [LICENSE](LICENSE) for details.

This project is based on the Otter project. For questions regarding Otter and its license, please read: https://www.mcs.anl.gov/research/projects/AR/otter/legal.html

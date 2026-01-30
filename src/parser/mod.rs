//! Simplified parser for Foras input files.
//!
//! This module currently handles a subset of the Foras syntax—list declarations,
//! commands, and atomic clauses—providing a scaffolding that can be expanded as
//! more functionality is ported from the original `io.c` implementation.

mod syntax;
mod formula;
mod operator;

pub use syntax::{
    ListKind, ListSection, ForasCommand, ForasFile, ParseError, Parser,
    WeightEntry,
};
pub use formula::{Formula, parse_formula};
pub use operator::{Fixity, Operator, OperatorTable};

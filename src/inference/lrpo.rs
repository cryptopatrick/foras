//! Lexicographic Recursive Path Ordering (LRPO) for term comparison.
//!
//! This module implements a simplified version of LRPO used to orient
//! equations for demodulation and guide paramodulation.

use crate::data::Term;

/// Maximum recursion depth to prevent stack overflow
const MAX_LRPO_DEPTH: usize = 500;

/// Comparison result for term ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Greater,
    Less,
    Equal,
    Incomparable,
}

/// Check if t1 > t2 in the LRPO ordering.
///
/// Returns true if t1 is strictly greater than t2.
pub fn lrpo_greater(t1: &Term, t2: &Term) -> bool {
    lrpo_with_depth(t1, t2, 0) == Ordering::Greater
}

/// Compare two terms using LRPO (entry point).
fn lrpo(t1: &Term, t2: &Term) -> Ordering {
    lrpo_with_depth(t1, t2, 0)
}

/// Compare two terms using LRPO with depth tracking.
///
/// This is a simplified implementation that:
/// - Variables are never greater than anything
/// - A non-variable term is greater than a variable if it contains that variable
/// - Terms are compared by their structure and weight
fn lrpo_with_depth(t1: &Term, t2: &Term, depth: usize) -> Ordering {
    // Prevent stack overflow
    if depth > MAX_LRPO_DEPTH {
        return Ordering::Incomparable;
    }
    match (t1, t2) {
        // Variable is never greater than anything
        (Term::Variable { .. }, _) => {
            if terms_equal(t1, t2) {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }

        // Non-variable > variable if it contains the variable
        (Term::Application { .. }, Term::Variable { .. }) => {
            if occurs_in(t2, t1) {
                Ordering::Greater
            } else {
                Ordering::Incomparable
            }
        }

        // Both are applications
        (
            Term::Application { symbol: s1, args: args1 },
            Term::Application { symbol: s2, args: args2 },
        ) => {
            // If same symbol, compare arguments
            if s1 == s2 {
                // Simplified: compare by structure
                compare_same_symbol_with_depth(args1, args2, t1, t2, depth)
            } else {
                // Different symbols: use weight as precedence
                let w1 = term_weight_with_depth(t1, 0);
                let w2 = term_weight_with_depth(t2, 0);

                if w1 > w2 {
                    // t1 > t2 if t1 > all args of t2
                    if args2.iter().all(|arg| lrpo_with_depth(t1, arg, depth + 1) == Ordering::Greater) {
                        Ordering::Greater
                    } else {
                        Ordering::Incomparable
                    }
                } else if w1 < w2 {
                    // Check if some arg of t1 equals or is greater than t2
                    for arg in args1 {
                        match lrpo_with_depth(arg, t2, depth + 1) {
                            Ordering::Greater | Ordering::Equal => return Ordering::Greater,
                            _ => {}
                        }
                    }
                    Ordering::Less
                } else {
                    // Equal weight: incomparable
                    Ordering::Incomparable
                }
            }
        }
    }
}

/// Compare arguments when both terms have the same root symbol (with depth tracking).
fn compare_same_symbol_with_depth(
    args1: &[Term],
    args2: &[Term],
    t1: &Term,
    t2: &Term,
    depth: usize,
) -> Ordering {
    // Prevent stack overflow
    if depth > MAX_LRPO_DEPTH {
        return Ordering::Incomparable;
    }

    // If same number of args, compare lexicographically
    if args1.len() == args2.len() {
        // Skip identical prefix
        let mut i = 0;
        while i < args1.len() && terms_equal_with_depth(&args1[i], &args2[i], depth) {
            i += 1;
        }

        if i == args1.len() {
            // All args identical
            Ordering::Equal
        } else {
            // Compare first differing argument
            let cmp = lrpo_with_depth(&args1[i], &args2[i], depth + 1);
            match cmp {
                Ordering::Greater => {
                    // t1 > t2 if t1 > all remaining args of t2
                    if args2
                        .iter()
                        .skip(i + 1)
                        .all(|arg| lrpo_with_depth(t1, arg, depth + 1) == Ordering::Greater)
                    {
                        Ordering::Greater
                    } else {
                        Ordering::Incomparable
                    }
                }
                Ordering::Less => {
                    // Check if some remaining arg of t1 equals or is greater than t2
                    for arg in args1.iter().skip(i + 1) {
                        match lrpo_with_depth(arg, t2, depth + 1) {
                            Ordering::Greater | Ordering::Equal => return Ordering::Greater,
                            _ => {}
                        }
                    }
                    Ordering::Less
                }
                _ => Ordering::Incomparable,
            }
        }
    } else {
        // Different number of args: incomparable
        Ordering::Incomparable
    }
}

/// Check if two terms are structurally equal (with depth tracking).
fn terms_equal(t1: &Term, t2: &Term) -> bool {
    terms_equal_with_depth(t1, t2, 0)
}

fn terms_equal_with_depth(t1: &Term, t2: &Term, depth: usize) -> bool {
    if depth > MAX_LRPO_DEPTH {
        return false;
    }
    match (t1, t2) {
        (Term::Variable { id: id1, .. }, Term::Variable { id: id2, .. }) => id1 == id2,
        (
            Term::Application { symbol: s1, args: args1 },
            Term::Application { symbol: s2, args: args2 },
        ) => {
            s1 == s2
                && args1.len() == args2.len()
                && args1
                    .iter()
                    .zip(args2.iter())
                    .all(|(a, b)| terms_equal_with_depth(a, b, depth + 1))
        }
        _ => false,
    }
}

/// Check if a variable occurs in a term (with depth tracking).
fn occurs_in(var: &Term, term: &Term) -> bool {
    occurs_in_with_depth(var, term, 0)
}

fn occurs_in_with_depth(var: &Term, term: &Term, depth: usize) -> bool {
    if depth > MAX_LRPO_DEPTH {
        return false;
    }
    if terms_equal_with_depth(var, term, depth) {
        return true;
    }

    match term {
        Term::Variable { .. } => false,
        Term::Application { args, .. } => {
            args.iter().any(|arg| occurs_in_with_depth(var, arg, depth + 1))
        }
    }
}

/// Compute term weight (number of symbols) with depth tracking.
fn term_weight_with_depth(term: &Term, depth: usize) -> usize {
    if depth > MAX_LRPO_DEPTH {
        return 1; // Default weight if too deep
    }
    match term {
        Term::Variable { .. } => 1,
        Term::Application { args, .. } => {
            1 + args
                .iter()
                .map(|arg| term_weight_with_depth(arg, depth + 1))
                .sum::<usize>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::symbol::{SymbolKind, SymbolTable};
    use crate::data::VariableId;

    fn make_var(id: u16) -> Term {
        Term::variable(VariableId::new(id))
    }

    fn make_const(table: &SymbolTable, name: &str) -> Term {
        let sym = table.intern(name, 0, SymbolKind::Constant);
        Term::application(sym, vec![])
    }

    fn make_app(table: &SymbolTable, name: &str, args: Vec<Term>) -> Term {
        let sym = table.intern(name, args.len() as u8, SymbolKind::Function);
        Term::application(sym, args)
    }

    #[test]
    fn test_variable_not_greater() {
        let x = make_var(0);
        let y = make_var(1);

        assert!(!lrpo_greater(&x, &y));
        assert!(!lrpo_greater(&x, &x));
    }

    #[test]
    fn test_term_contains_variable() {
        let table = SymbolTable::new();
        let x = make_var(0);
        let f_x = make_app(&table, "f", vec![x.clone()]);

        assert!(lrpo_greater(&f_x, &x));
        assert!(!lrpo_greater(&x, &f_x));
    }

    #[test]
    fn test_weight_comparison() {
        let table = SymbolTable::new();
        let a = make_const(&table, "a");
        let f_a = make_app(&table, "f", vec![a.clone()]);
        let g_f_a = make_app(&table, "g", vec![f_a.clone()]);

        // g(f(a)) has weight 3, f(a) has weight 2, a has weight 1
        assert!(lrpo_greater(&g_f_a, &f_a));
        assert!(lrpo_greater(&f_a, &a));
        assert!(lrpo_greater(&g_f_a, &a));
    }

    #[test]
    fn test_same_symbol_comparison() {
        let table = SymbolTable::new();
        let a = make_const(&table, "a");
        let b = make_const(&table, "b");
        let f_a = make_app(&table, "f", vec![a.clone()]);
        let f_b = make_app(&table, "f", vec![b.clone()]);

        // f(a) vs f(b): incomparable (same structure, different constants)
        assert!(!lrpo_greater(&f_a, &f_b));
        assert!(!lrpo_greater(&f_b, &f_a));
    }
}

//! Term ordering for orienting equalities.
//!
//! Implements LRPO (Lexicographic Recursive Path Ordering), which is used to:
//! - Orient equalities (determine if l=r should be rewritten as l→r or r→l)
//! - Prevent infinite loops in paramodulation and demodulation
//! - Guide the search by choosing "simpler" terms
//!
//! This implementation follows C Foras's lrpo.c closely to ensure compatibility.

use crate::data::{SymbolId, Term};
use std::cmp::Ordering;

/// Maximum recursion depth for LRPO to prevent stack overflow
const MAX_LRPO_DEPTH: usize = 100;

/// Precedence comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Precedence {
    SameAs,
    GreaterThan,
    LessThan,
    NotComparable,
}

/// Lexicographic Recursive Path Ordering (LRPO) for terms.
///
/// LRPO is a simplification ordering that satisfies:
/// - Well-founded: No infinite descending chains
/// - Total on ground terms: Any two ground terms can be compared
/// - Compatible with term structure: If s > t then f(...s...) > f(...t...)
///
/// The ordering is based on:
/// 1. Symbol precedence (set via lex() directive, lower value = higher precedence)
/// 2. Lexicographic comparison of arguments for same function symbol
/// 3. Variables are smaller than non-variable terms that contain them
#[derive(Debug, Clone)]
pub struct LRPO {
    /// Symbol precedence (lower value = higher precedence)
    /// If not specified, symbols are NOT_COMPARABLE (like C Foras)
    precedence: Vec<(SymbolId, u32)>,
}

impl LRPO {
    /// Create a new LRPO ordering with default precedence.
    pub fn new() -> Self {
        Self {
            precedence: Vec::new(),
        }
    }

    /// Set explicit precedence for a symbol (lower value = higher precedence).
    pub fn set_precedence(&mut self, symbol: SymbolId, prec: u32) {
        // Remove old precedence if exists
        self.precedence.retain(|(s, _)| *s != symbol);
        self.precedence.push((symbol, prec));
    }

    /// Get precedence for a symbol.
    /// Returns None if the symbol has no explicit precedence (incomparable).
    fn get_precedence(&self, symbol: SymbolId) -> Option<u32> {
        self.precedence
            .iter()
            .find(|(s, _)| *s == symbol)
            .map(|(_, p)| *p)
    }

    /// Compare symbol precedence (following C Foras's sym_precedence).
    /// In C Foras, LOWER lex_val means HIGHER precedence.
    fn sym_precedence(&self, sym1: SymbolId, sym2: SymbolId) -> Precedence {
        if sym1 == sym2 {
            return Precedence::SameAs;
        }

        match (self.get_precedence(sym1), self.get_precedence(sym2)) {
            (Some(p1), Some(p2)) => {
                // Lower value = higher precedence (like C Foras)
                if p1 < p2 {
                    Precedence::GreaterThan // sym1 has higher precedence
                } else if p1 > p2 {
                    Precedence::LessThan // sym1 has lower precedence
                } else {
                    Precedence::SameAs
                }
            }
            // If either symbol has no explicit precedence, they're not comparable
            // (This matches C Foras's behavior with lex_val == MAX_INT)
            _ => Precedence::NotComparable,
        }
    }

    /// Check if t1 > t2 in LRPO (matches C Foras's lrpo function).
    ///
    /// This is the core LRPO algorithm following C Foras's lrpo.c exactly.
    /// Returns true if t1 is strictly greater than t2.
    fn lrpo_gt(&self, t1: &Term, t2: &Term, depth: usize) -> bool {
        // Prevent stack overflow
        if depth > MAX_LRPO_DEPTH {
            return false;
        }

        match (t1, t2) {
            // Variable is NEVER greater than anything (C Foras line 228-230)
            (Term::Variable { .. }, _) => false,

            // t1 > variable iff t1 properly contains that variable (C Foras line 231-233)
            (Term::Application { .. }, Term::Variable { .. }) => self.occurs_in(t2, t1),

            // Both are applications
            (
                Term::Application {
                    symbol: f,
                    args: s_args,
                },
                Term::Application {
                    symbol: g,
                    args: t_args,
                },
            ) => {
                // Same symbol with lexicographic status: use lrpo_lex (C Foras line 234-236)
                if f == g && s_args.len() == t_args.len() {
                    return self.lrpo_lex(t1, t2, s_args, t_args, depth);
                }

                // Different symbols: compare precedence (C Foras line 238-255)
                let prec = self.sym_precedence(*f, *g);

                match prec {
                    Precedence::SameAs => {
                        // Same precedence but different symbols: use multiset comparison
                        // For simplicity, we treat this as incomparable (return false)
                        // Full multiset comparison could be added later
                        false
                    }
                    Precedence::GreaterThan => {
                        // f > g: return (t1 > each arg of t2) (C Foras line 241-245)
                        t_args.iter().all(|t_arg| self.lrpo_gt(t1, t_arg, depth + 1))
                    }
                    Precedence::LessThan | Precedence::NotComparable => {
                        // f < g or incomparable: return (some arg of t1 >= t2) (C Foras line 247-254)
                        s_args.iter().any(|s_arg| {
                            self.terms_identical(s_arg, t2) || self.lrpo_gt(s_arg, t2, depth + 1)
                        })
                    }
                }
            }
        }
    }

    /// Lexicographic comparison for same-symbol terms (C Foras's lrpo_lex).
    fn lrpo_lex(
        &self,
        t1: &Term,
        t2: &Term,
        s_args: &[Term],
        t_args: &[Term],
        depth: usize,
    ) -> bool {
        // Skip over identical arguments (C Foras line 77-79)
        let mut i = 0;
        while i < s_args.len() && self.terms_identical(&s_args[i], &t_args[i]) {
            i += 1;
        }

        if i >= s_args.len() {
            // All arguments identical, t1 == t2, so t1 > t2 is false
            false
        } else if self.lrpo_gt(&s_args[i], &t_args[i], depth + 1) {
            // First differing arg: s_args[i] > t_args[i]
            // Return (t1 > each remaining arg of t2) (C Foras line 83-87)
            t_args
                .iter()
                .skip(i + 1)
                .all(|t_arg| self.lrpo_gt(t1, t_arg, depth + 1))
        } else {
            // First differing arg: s_args[i] <= t_args[i]
            // Return (some remaining arg of t1 >= t2) (C Foras line 90-96)
            s_args.iter().skip(i + 1).any(|s_arg| {
                self.terms_identical(s_arg, t2) || self.lrpo_gt(s_arg, t2, depth + 1)
            })
        }
    }

    /// Check if two terms are structurally identical.
    fn terms_identical(&self, t1: &Term, t2: &Term) -> bool {
        match (t1, t2) {
            (Term::Variable { id: x, .. }, Term::Variable { id: y, .. }) => x == y,
            (
                Term::Application {
                    symbol: s1,
                    args: args1,
                },
                Term::Application {
                    symbol: s2,
                    args: args2,
                },
            ) => {
                s1 == s2
                    && args1.len() == args2.len()
                    && args1
                        .iter()
                        .zip(args2.iter())
                        .all(|(a, b)| self.terms_identical(a, b))
            }
            _ => false,
        }
    }

    /// Check if variable v occurs in term t.
    fn occurs_in(&self, v: &Term, t: &Term) -> bool {
        match v {
            Term::Variable { .. } => self.occurs_in_impl(v, t),
            _ => false,
        }
    }

    fn occurs_in_impl(&self, v: &Term, t: &Term) -> bool {
        if self.terms_identical(v, t) {
            return true;
        }
        match t {
            Term::Variable { .. } => false,
            Term::Application { args, .. } => args.iter().any(|arg| self.occurs_in_impl(v, arg)),
        }
    }

    /// Compare two terms using LRPO.
    ///
    /// Returns:
    /// - `Ordering::Greater` if `s > t`
    /// - `Ordering::Less` if `s < t`
    /// - `Ordering::Equal` if `s = t` syntactically or if incomparable
    ///
    /// Note: When terms are incomparable, we return Equal rather than having
    /// a separate Incomparable variant. Callers should use `greater()` for
    /// definitive ordering checks.
    pub fn compare(&self, s: &Term, t: &Term) -> Ordering {
        if self.terms_identical(s, t) {
            Ordering::Equal
        } else if self.lrpo_gt(s, t, 0) {
            Ordering::Greater
        } else if self.lrpo_gt(t, s, 0) {
            Ordering::Less
        } else {
            // Incomparable - return Equal as a neutral choice
            // The demod code uses weight as primary criterion anyway
            Ordering::Equal
        }
    }

    /// Check if s > t in LRPO ordering.
    pub fn greater(&self, s: &Term, t: &Term) -> bool {
        self.lrpo_gt(s, t, 0)
    }

    /// Check if s >= t in LRPO ordering.
    pub fn greater_or_equal(&self, s: &Term, t: &Term) -> bool {
        self.terms_identical(s, t) || self.lrpo_gt(s, t, 0)
    }
}

impl Default for LRPO {
    fn default() -> Self {
        Self::new()
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

    fn make_fun(table: &SymbolTable, name: &str, args: Vec<Term>) -> Term {
        let sym = table.intern(name, args.len() as u8, SymbolKind::Function);
        Term::application(sym, args)
    }

    #[test]
    fn lrpo_variable_never_greater() {
        let table = SymbolTable::new();
        let lrpo = LRPO::new();

        let x = make_var(0);
        let y = make_var(1);
        let a = make_const(&table, "a");

        // Variable is NEVER greater than anything
        assert!(!lrpo.greater(&x, &y));
        assert!(!lrpo.greater(&x, &a));
        assert!(!lrpo.greater(&y, &x));
    }

    #[test]
    fn lrpo_term_contains_variable() {
        let table = SymbolTable::new();
        let lrpo = LRPO::new();

        let x = make_var(0);
        let y = make_var(1);
        let f_x = make_fun(&table, "f", vec![x.clone()]);

        // f(x) > x because x occurs in f(x)
        assert!(lrpo.greater(&f_x, &x));
        assert!(!lrpo.greater(&x, &f_x));

        // f(x) NOT > y because y does NOT occur in f(x)
        assert!(!lrpo.greater(&f_x, &y));
    }

    #[test]
    fn lrpo_same_symbol_lexicographic() {
        let table = SymbolTable::new();
        let mut lrpo = LRPO::new();

        let a = make_const(&table, "a");
        let b = make_const(&table, "b");

        // Set precedence: a > b (lower value = higher precedence)
        let a_sym = if let Term::Application { symbol, .. } = &a {
            *symbol
        } else {
            panic!()
        };
        let b_sym = if let Term::Application { symbol, .. } = &b {
            *symbol
        } else {
            panic!()
        };
        lrpo.set_precedence(a_sym, 1);
        lrpo.set_precedence(b_sym, 2);

        let f_a = make_fun(&table, "f", vec![a.clone()]);
        let f_b = make_fun(&table, "f", vec![b.clone()]);

        // f(a) > f(b) because a > b in precedence
        assert!(lrpo.greater(&f_a, &f_b));
        assert!(!lrpo.greater(&f_b, &f_a));
    }

    #[test]
    fn lrpo_nested_terms_same_symbol() {
        let table = SymbolTable::new();
        let lrpo = LRPO::new();

        let x = make_var(0);
        let f_x = make_fun(&table, "f", vec![x.clone()]);
        let f_f_x = make_fun(&table, "f", vec![f_x.clone()]);

        // f(f(x)) > f(x) > x (lexicographic: first arg f(x) > x)
        assert!(lrpo.greater(&f_f_x, &f_x));
        assert!(lrpo.greater(&f_x, &x));
        assert!(lrpo.greater(&f_f_x, &x));
    }

    #[test]
    fn lrpo_precedence_comparison() {
        let table = SymbolTable::new();
        let mut lrpo = LRPO::new();

        let x = make_var(0);
        let f_x = make_fun(&table, "f", vec![x.clone()]);
        let g_x = make_fun(&table, "g", vec![x.clone()]);

        // Get symbols
        let f_sym = if let Term::Application { symbol, .. } = &f_x {
            *symbol
        } else {
            panic!()
        };
        let g_sym = if let Term::Application { symbol, .. } = &g_x {
            *symbol
        } else {
            panic!()
        };

        // Set f > g (lower prec value = higher precedence)
        lrpo.set_precedence(f_sym, 1);
        lrpo.set_precedence(g_sym, 2);

        // f(x) > g(x) because f > g and f(x) > x (which is g(x)'s arg)
        assert!(lrpo.greater(&f_x, &g_x));
        assert!(!lrpo.greater(&g_x, &f_x));
    }

    #[test]
    fn lrpo_incomparable_without_precedence() {
        let table = SymbolTable::new();
        let lrpo = LRPO::new(); // No precedence set

        let x = make_var(0);
        let f_x = make_fun(&table, "f", vec![x.clone()]);
        let g_x = make_fun(&table, "g", vec![x.clone()]);

        // Without precedence, f(x) and g(x) are incomparable
        // (neither contains the other as subterm)
        // Actually, check "some arg of f(x) >= g(x)": x is not >= g(x), so false
        // And "some arg of g(x) >= f(x)": x is not >= f(x), so false
        // So neither is greater
        assert!(!lrpo.greater(&f_x, &g_x));
        assert!(!lrpo.greater(&g_x, &f_x));
    }
}

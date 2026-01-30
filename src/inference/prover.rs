//! Basic resolution-based theorem prover skeleton.
//!
//! This module provides a simple saturation-based prover that uses binary
//! resolution to search for contradictions (empty clauses).

use crate::data::{Clause, ClauseArena, ClauseId, ClauseList, LRPO, SymbolId, SymbolTable, WeightTable};
use crate::inference::{
    all_resolvents, back_subsumed, demodulate_clause_with_limit, extract_demodulator, factor_clause, forward_subsumed, forward_subsumed_ancestor,
    forward_unit_deletion, hyperresolve_units, linked_ur_resolve, paramodulate_into, ur_resolve,
    Demodulator, LinkedURConfig,
};

/// Result of a proof search.
#[derive(Clone, Debug)]
pub enum ProofResult {
    /// A proof was found (empty clause derived)
    Proof {
        /// The empty clause that was derived
        empty_clause_id: ClauseId,
        /// Total clauses generated during search
        clauses_generated: usize,
        /// Clauses kept after filtering
        clauses_kept: usize,
    },
    /// Search exhausted without finding proof
    Saturated {
        clauses_generated: usize,
        clauses_kept: usize,
    },
    /// Search exceeded resource limits
    ResourceLimit {
        clauses_generated: usize,
        clauses_kept: usize,
        limit_type: String,
    },
}

/// Configuration for the prover.
#[derive(Clone, Debug)]
pub struct ProverConfig {
    /// Maximum number of clauses to generate
    pub max_clauses: usize,
    /// Maximum number of given clauses to process
    pub max_given: usize,
    /// Ratio of clauses selected by weight vs FIFO (pick_given_ratio)
    /// For every N clauses selected by weight, select 1 by FIFO
    pub pick_given_ratio: usize,
    /// Maximum proof search time in seconds (0 = unlimited)
    pub max_seconds: u64,
    /// Auto mode enabled
    pub auto_mode: bool,
    /// Use hyperresolution
    pub use_hyper_res: bool,
    /// Use binary resolution
    pub use_binary_res: bool,
    /// Use paramodulation (para_into)
    pub use_para_into: bool,
    /// Use paramodulation (para_from)
    pub use_para_from: bool,
    /// Paramodulate from left side of equality (para_from_left)
    pub para_from_left: bool,
    /// Paramodulate from right side of equality (para_from_right)
    pub para_from_right: bool,
    /// Paramodulate into left side of equality (para_into_left)
    pub para_into_left: bool,
    /// Paramodulate into right side of equality (para_into_right)
    pub para_into_right: bool,
    /// Use demodulation for term rewriting
    pub use_demod: bool,
    /// Use back-demodulation to simplify existing clauses
    pub use_back_demod: bool,
    /// Use factoring to simplify clauses
    pub use_factor: bool,
    /// Use UR-resolution (unit-resulting resolution)
    pub use_ur_res: bool,
    /// Use Linked UR-resolution
    pub use_linked_ur_res: bool,
    /// Use subsumption to eliminate redundant clauses
    pub use_subsumption: bool,
    /// Use ancestor subsumption (bidirectional subsumption with proof length preference)
    pub use_ancestor_subsume: bool,
    /// Use unit deletion to simplify clauses
    pub use_unit_deletion: bool,
    /// Maximum weight for clauses (higher weight clauses are discarded)
    pub max_weight: i32,
    /// Weight for forward subsumption hint matching
    pub fsub_hint_wt: i32,
    /// Additive weight for forward subsumption hint matching
    pub fsub_hint_add_wt: i32,
    /// Weight for back subsumption hint matching
    pub bsub_hint_wt: i32,
    /// Additive weight for back subsumption hint matching
    pub bsub_hint_add_wt: i32,
    /// Weight for equivalence hint matching
    pub equiv_hint_wt: i32,
    /// Additive weight for equivalence hint matching
    pub equiv_hint_add_wt: i32,
    /// Keep clauses that subsume hints despite exceeding max weight
    pub keep_hint_subsumers: bool,
    /// Keep clauses that are equivalent to hints despite exceeding max weight
    pub keep_hint_equivalents: bool,
    /// Maximum clauses generated per given clause iteration (0 = unlimited)
    pub max_clauses_per_given: usize,
    /// Maximum iterations for demodulation rewrites (prevent infinite rewrite loops)
    pub max_demod_iterations: usize,
    /// Memory limit in bytes (0 = unlimited)
    pub max_memory_bytes: usize,
}

impl Default for ProverConfig {
    fn default() -> Self {
        Self {
            max_clauses: 10000,
            max_given: 1000,
            pick_given_ratio: 4,
            max_seconds: 0,
            auto_mode: false,
            use_hyper_res: false,
            use_binary_res: true,
            use_para_into: false,
            use_para_from: false,
            para_from_left: true,
            para_from_right: true,
            para_into_left: true,
            para_into_right: true,
            use_demod: false,
            use_back_demod: false,
            use_factor: false,
            use_ur_res: false,
            use_linked_ur_res: false,
            use_subsumption: false,
            use_ancestor_subsume: false,
            use_unit_deletion: false,
            max_weight: i32::MAX,
            fsub_hint_wt: crate::inference::MAX_WEIGHT,
            fsub_hint_add_wt: 0,
            bsub_hint_wt: crate::inference::MAX_WEIGHT,
            bsub_hint_add_wt: -1000,
            equiv_hint_wt: crate::inference::MAX_WEIGHT,
            equiv_hint_add_wt: 0,
            keep_hint_subsumers: false,
            keep_hint_equivalents: false,
            max_clauses_per_given: 0,  // 0 = unlimited (like original Foras)
            max_demod_iterations: 100,
            max_memory_bytes: 0,
        }
    }
}

/// Simple resolution-based theorem prover.
pub struct Prover {
    /// Configuration settings
    config: ProverConfig,
    /// Symbol table for looking up symbol names (needed for answer literals)
    symbols: SymbolTable,
    /// Storage for all clauses
    arena: ClauseArena,
    /// Set of support (clauses to be selected as given)
    sos: ClauseList,
    /// Usable clauses (clauses to resolve against)
    usable: ClauseList,
    /// Equality symbol (if set)
    eq_symbol: Option<SymbolId>,
    /// Demodulators for term rewriting
    demodulators: Vec<Demodulator>,
    /// Symbol weight table for clause selection
    weight_table: WeightTable,
    /// Term ordering for demodulation and paramodulation
    lrpo: LRPO,
    /// Hints for guiding the search
    hints: crate::inference::HintsList,
    /// Statistics
    clauses_generated: usize,
    clauses_kept: usize,
    given_count: usize,
    /// Counter for pick_given_ratio (tracks when to select by FIFO vs weight)
    pick_count: usize,
    /// Proof found during back-demodulation (t != t contradiction)
    proof_from_back_demod: Option<ClauseId>,
}

impl Prover {
    /// Create a new prover with default configuration.
    pub fn new() -> Self {
        Self::with_config(ProverConfig::default(), SymbolTable::new())
    }

    /// Create a new prover with custom configuration.
    pub fn with_config(config: ProverConfig, symbols: SymbolTable) -> Self {
        Self {
            config,
            symbols,
            arena: ClauseArena::new(),
            sos: ClauseList::new("sos"),
            usable: ClauseList::new("usable"),
            eq_symbol: None,
            demodulators: Vec::new(),
            weight_table: WeightTable::new(),
            lrpo: LRPO::new(),
            hints: crate::inference::HintsList::new(),
            clauses_generated: 0,
            clauses_kept: 0,
            given_count: 0,
            pick_count: 0,
            proof_from_back_demod: None,
        }
    }

    /// Set symbol weight for clause selection.
    pub fn set_symbol_weight(&mut self, symbol: SymbolId, weight: i32) {
        self.weight_table.set_weight(symbol, weight);
    }

    /// Set default weight for unlisted symbols.
    pub fn set_default_weight(&mut self, weight: i32) {
        self.weight_table.set_default(weight);
    }

    /// Set the equality symbol for paramodulation.
    pub fn set_eq_symbol(&mut self, sym: SymbolId) {
        self.eq_symbol = Some(sym);
    }

    /// Set symbol precedence for term ordering (lower value = higher precedence).
    pub fn set_symbol_precedence(&mut self, sym: SymbolId, prec: u32) {
        self.lrpo.set_precedence(sym, prec);
    }

    /// Add a hint clause to guide the search.
    pub fn add_hint(&mut self, clause: Clause) {
        let hint_data = crate::inference::HintData::new(
            self.config.fsub_hint_wt,
            self.config.fsub_hint_add_wt,
            self.config.bsub_hint_wt,
            self.config.bsub_hint_add_wt,
            self.config.equiv_hint_wt,
            self.config.equiv_hint_add_wt,
        );
        self.hints.add_hint(clause, hint_data);
    }

    /// Check if a clause is a proof (empty or contains only answer literals).
    fn is_proof(&self, clause: &Clause) -> bool {
        if clause.literals.is_empty() {
            return true;
        }
        // Check for answer literals (literals starting with $Ans)
        // A clause containing ONLY answer literals is a valid answer/proof
        clause.literals.iter().all(|lit| {
            if let crate::data::Term::Application { symbol, .. } = &lit.atom {
                if let Some(sym_data) = self.symbols.get(*symbol) {
                    return sym_data.name.starts_with("$Ans");
                }
            }
            false
        })
    }

    /// Process a new clause: apply factoring, demodulation, and check if it's a demodulator.
    ///
    /// Returns None if the clause should be discarded, or Some(processed_clause).
    fn process_new_clause(&mut self, mut clause: Clause) -> Option<Clause> {
        // Debug: trace parents
        let input_parents = clause.parents.len();
        // eprintln!("PROCESS: Input clause parents: {}", input_parents);

        // Check for tautology: clause contains both P(t) and ~P(t) for some terms
        // This is a trivially true clause that should be discarded
        for i in 0..clause.literals.len() {
            for j in (i + 1)..clause.literals.len() {
                let lit1 = &clause.literals[i];
                let lit2 = &clause.literals[j];

                // Check if signs are opposite and atoms are same
                if lit1.sign != lit2.sign && lit1.atom == lit2.atom {
                    // Tautology: P(t) | ~P(t)
                    eprintln!("TAUTOLOGY DETECTED: {:?}", clause);
                    return None;
                }
            }
        }

        // Apply factoring if enabled (simplify before other processing)
        if self.config.use_factor {
            // Try to factor the clause once
            let factors = factor_clause(&clause, None);
            if !factors.is_empty() {
                // Use the first factor (in full Foras, might generate all factors)
                clause = factors[0].clause.clone();
            }
        }

        // Apply forward demodulation if enabled
        if self.config.use_demod && !self.demodulators.is_empty() {
            clause = demodulate_clause_with_limit(&clause, &self.demodulators, self.config.max_demod_iterations);
        }

        // Check for xx_res: negated reflexive equality (t != t) is immediately false
        // This gives an empty clause (proof found)
        if let Some(eq_sym) = self.eq_symbol {
            if clause.literals.len() == 1 {
                let lit = &clause.literals[0];
                if !lit.sign {
                    // Negated literal
                    if let crate::data::Term::Application { symbol, args } = &lit.atom {
                        if *symbol == eq_sym && args.len() == 2 {
                            // Check if both sides are syntactically equal
                            if args[0] == args[1] {
                                // t != t is a contradiction - return empty clause
                                return Some(Clause::new(vec![]));
                            }
                        }
                    }
                }
            }
        }

        // Check if this clause is a demodulator
        if self.config.use_demod {
            if let Some(eq_sym) = self.eq_symbol {
                if let Some(demod) = extract_demodulator(&clause, eq_sym, Some(&self.lrpo)) {
                    // Apply back-demodulation: rewrite existing clauses with new demodulator
                    if self.config.use_back_demod {
                        self.back_demodulate(&demod);
                    }
                    self.demodulators.push(demod);
                }
            }
        }

        if clause.parents.is_empty() && input_parents > 0 {
             eprintln!("PROCESS: Parents lost during processing! Input: {}, Output: 0", input_parents);
        }

        Some(clause)
    }

    /// Add an input/initial clause to the set of support.
    ///
    /// Input clauses are not subject to max_weight filtering.
    pub fn add_sos(&mut self, mut clause: Clause) -> ClauseId {
        // Cache the weight for efficient clause selection
        clause.pick_weight = self.weight_table.weight_clause(&clause);

        // Adjust weight based on hints if they match (for priority)
        if !self.hints.is_empty() {
            crate::inference::adjust_weight_with_hints(&mut clause, &self.hints);
        }

        // Input clauses bypass max_weight check
        let id = self.arena.insert(clause);
        self.sos.push(id);
        self.clauses_kept += 1;
        id
    }

    /// Add an inferred clause to the set of support.
    ///
    /// Returns Some(id) if the clause was added, None if it was discarded (e.g., exceeds max_weight).
    pub fn add_inferred_sos(&mut self, mut clause: Clause) -> Option<ClauseId> {
        // Cache the weight for efficient clause selection
        clause.pick_weight = self.weight_table.weight_clause(&clause);

        // Adjust weight based on hints if they match
        if !self.hints.is_empty() {
            crate::inference::adjust_weight_with_hints(&mut clause, &self.hints);
        }

        // Check max_weight constraint for inferred clauses
        if self.config.max_weight < i32::MAX && clause.pick_weight > self.config.max_weight {
            // Clause exceeds max_weight - check if it should be kept due to hints
            let keep = crate::inference::hint_keep_test(
                &clause,
                &self.hints,
                self.config.keep_hint_subsumers,
                self.config.keep_hint_equivalents,
            );
            if !keep {
                return None; // Discard clause
            }
        }

        let id = self.arena.insert(clause);
        self.sos.push(id);
        self.clauses_kept += 1;
        Some(id)
    }

    /// Try to add a processed inferred clause to SOS.
    ///
    /// This applies max_weight filtering and hint checks. Returns true if added, false if discarded.
    fn try_keep_clause(&mut self, mut clause: Clause) -> bool {
        // Debug: catch clauses with empty parents (should only be initial axioms)
        if clause.parents.is_empty() && !clause.attributes.iter().any(|attr| attr.name == "list") {
            eprintln!("WARNING: Inferred clause with empty parents: {:?}", clause);
        }

        // Cache the weight for efficient clause selection
        clause.pick_weight = self.weight_table.weight_clause(&clause);

        // Adjust weight based on hints if they match
        if !self.hints.is_empty() {
            crate::inference::adjust_weight_with_hints(&mut clause, &self.hints);
        }

        // Check max_weight constraint for inferred clauses
        if self.config.max_weight < i32::MAX && clause.pick_weight > self.config.max_weight {
            // Clause exceeds max_weight - check if it should be kept due to hints
            let keep = crate::inference::hint_keep_test(
                &clause,
                &self.hints,
                self.config.keep_hint_subsumers,
                self.config.keep_hint_equivalents,
            );
            if !keep {
                return false; // Discard clause
            }
        }

        let id = self.arena.insert(clause);
        self.sos.push(id);
        self.clauses_kept += 1;
        true
    }

    /// Add a clause to the usable set.
    pub fn add_usable(&mut self, mut clause: Clause) -> ClauseId {
        // Cache the weight (may be used if clause moves to SOS later)
        clause.pick_weight = self.weight_table.weight_clause(&clause);

        let id = self.arena.insert(clause);
        self.usable.push(id);
        self.clauses_kept += 1;
        id
    }

    /// Select and remove the lightest clause from SOS based on weight.
    ///
    /// This scans all clauses in SOS, finds the one with minimum weight,
    /// removes it from the list, and returns it.
    ///
    /// Note: Uses cached pick_weight from clause for O(n) selection instead of O(n*m).
    fn select_lightest_clause(&mut self) -> Option<ClauseId> {
        if self.sos.is_empty() {
            return None;
        }

        // Find the clause with minimum weight
        let mut min_weight = i32::MAX;
        let mut min_index = 0;

        for (index, clause_id) in self.sos.iter().enumerate() {
            if let Some(clause) = self.arena.get(*clause_id) {
                // Use cached weight instead of recalculating!
                // This changes complexity from O(n*m) to O(n) where m = avg clause size
                if clause.pick_weight < min_weight {
                    min_weight = clause.pick_weight;
                    min_index = index;
                }
            }
        }

        // Remove and return the lightest clause
        self.sos.remove(min_index)
    }

    /// Pre-process initial clauses to extract demodulators.
    fn preprocess_initial_clauses(&mut self) {
        if !self.config.use_demod {
            return;
        }

        let eq_sym = match self.eq_symbol {
            Some(sym) => sym,
            None => return,
        };

        // Extract demodulators from usable clauses
        for clause_id in self.usable.iter() {
            if let Some(clause) = self.arena.get(*clause_id) {
                if let Some(demod) = extract_demodulator(clause, eq_sym, Some(&self.lrpo)) {
                    self.demodulators.push(demod);
                }
            }
        }

        // Also extract demodulators from SOS clauses
        for clause_id in self.sos.iter() {
            if let Some(clause) = self.arena.get(*clause_id) {
                if let Some(demod) = extract_demodulator(clause, eq_sym, Some(&self.lrpo)) {
                    self.demodulators.push(demod);
                }
            }
        }
    }

    /// Check if a clause is forward subsumed, using ancestor subsumption if enabled.
    fn is_forward_subsumed(&self, clause: &Clause, usable_refs: &[&Clause], sos_refs: &[&Clause]) -> bool {
        if self.config.use_ancestor_subsume {
            forward_subsumed_ancestor(clause, usable_refs) || forward_subsumed_ancestor(clause, sos_refs)
        } else {
            forward_subsumed(clause, usable_refs) || forward_subsumed(clause, sos_refs)
        }
    }

    /// Perform backward subsumption: remove clauses subsumed by the new clause.
    ///
    /// When a new more general clause is derived, it can subsume (make redundant)
    /// existing more specific clauses. This reduces the search space.
    fn perform_back_subsumption(&mut self, new_clause: &Clause) {
        if !self.config.use_subsumption {
            return;
        }

        // Collect clause IDs and references from usable
        let usable_pairs: Vec<(ClauseId, &Clause)> = self.usable.iter()
            .filter_map(|id| self.arena.get(*id).map(|c| (*id, c)))
            .collect();

        // Collect clause IDs and references from SOS
        let sos_pairs: Vec<(ClauseId, &Clause)> = self.sos.iter()
            .filter_map(|id| self.arena.get(*id).map(|c| (*id, c)))
            .collect();

        // Find clause IDs subsumed by the new clause
        let usable_refs: Vec<&Clause> = usable_pairs.iter().map(|(_, c)| *c).collect();
        let sos_refs: Vec<&Clause> = sos_pairs.iter().map(|(_, c)| *c).collect();

        let subsumed_in_usable = back_subsumed(new_clause, &usable_refs);
        let subsumed_in_sos = back_subsumed(new_clause, &sos_refs);

        // Collect IDs to remove from usable
        let mut usable_ids_to_remove: Vec<ClauseId> = Vec::new();
        for subsumed_clause in subsumed_in_usable {
            for (id, clause) in &usable_pairs {
                if std::ptr::eq(*clause, subsumed_clause) {
                    usable_ids_to_remove.push(*id);
                    break;
                }
            }
        }

        // Collect IDs to remove from SOS
        let mut sos_ids_to_remove: Vec<ClauseId> = Vec::new();
        for subsumed_clause in subsumed_in_sos {
            for (id, clause) in &sos_pairs {
                if std::ptr::eq(*clause, subsumed_clause) {
                    sos_ids_to_remove.push(*id);
                    break;
                }
            }
        }

        // Remove clauses from usable by finding and removing each ID
        for id_to_remove in usable_ids_to_remove {
            let mut index_to_remove = None;
            for (idx, id) in self.usable.iter().enumerate() {
                if *id == id_to_remove {
                    index_to_remove = Some(idx);
                    break;
                }
            }
            if let Some(idx) = index_to_remove {
                self.usable.remove(idx);
            }
        }

        // Remove clauses from SOS by finding and removing each ID
        for id_to_remove in sos_ids_to_remove {
            let mut index_to_remove = None;
            for (idx, id) in self.sos.iter().enumerate() {
                if *id == id_to_remove {
                    index_to_remove = Some(idx);
                    break;
                }
            }
            if let Some(idx) = index_to_remove {
                self.sos.remove(idx);
            }
        }
    }

    /// Apply a new demodulator to all existing clauses (back-demodulation).
    ///
    /// This rewrites clauses in both usable and SOS with the new demodulator,
    /// which can simplify the clause set and help find proofs faster.
    /// If a t != t contradiction is found, sets proof_from_back_demod.
    fn back_demodulate(&mut self, new_demod: &Demodulator) {
        let eq_sym = self.eq_symbol;

        // Apply to usable clauses
        for clause_id in self.usable.iter() {
            if let Some(clause) = self.arena.get(*clause_id).cloned() {
                let simplified = demodulate_clause_with_limit(&clause, &[new_demod.clone()], self.config.max_demod_iterations);

                // Only update if the clause actually changed
                if clause.literals != simplified.literals {
                    // Check for xx_res: t != t becomes empty clause
                    if let Some(eq_s) = eq_sym {
                        if simplified.literals.len() == 1 {
                            let lit = &simplified.literals[0];
                            if !lit.sign {
                                if let crate::data::Term::Application { symbol, args } = &lit.atom {
                                    if *symbol == eq_s && args.len() == 2 && args[0] == args[1] {
                                        // Found t != t - create empty clause
                                        let empty = Clause::new(vec![]);
                                        let empty_id = self.arena.insert(empty);
                                        self.proof_from_back_demod = Some(empty_id);
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(mut_clause) = self.arena.get_mut(*clause_id) {
                        *mut_clause = simplified;
                    }
                }
            }
        }

        // Apply to SOS clauses
        for clause_id in self.sos.iter() {
            if let Some(clause) = self.arena.get(*clause_id).cloned() {
                let simplified = demodulate_clause_with_limit(&clause, &[new_demod.clone()], self.config.max_demod_iterations);

                // Only update if the clause actually changed
                if clause.literals != simplified.literals {
                    // Check for xx_res: t != t becomes empty clause
                    if let Some(eq_s) = eq_sym {
                        if simplified.literals.len() == 1 {
                            let lit = &simplified.literals[0];
                            if !lit.sign {
                                if let crate::data::Term::Application { symbol, args } = &lit.atom {
                                    if *symbol == eq_s && args.len() == 2 && args[0] == args[1] {
                                        // Found t != t - create empty clause
                                        let empty = Clause::new(vec![]);
                                        let empty_id = self.arena.insert(empty);
                                        self.proof_from_back_demod = Some(empty_id);
                                        return;
                                    }
                                }
                            }
                        }
                    }

                    // Need to recalculate weight for SOS clauses since they may be selected
                    if let Some(mut_clause) = self.arena.get_mut(*clause_id) {
                        *mut_clause = simplified;
                        mut_clause.pick_weight = self.weight_table.weight_clause(mut_clause);
                    }
                }
            }
        }
    }

    /// Run the proof search.
    pub fn search(&mut self) -> ProofResult {
        eprintln!("DEBUG: Starting search, SOS={}, usable={}", self.sos.len(), self.usable.len());
        // Pre-process to extract initial demodulators
        self.preprocess_initial_clauses();
        eprintln!("DEBUG: After preprocess, demodulators={}", self.demodulators.len());

        // Start timer for max_seconds check
        let start_time = std::time::Instant::now();

        while !self.sos.is_empty() {
            // Check if back-demodulation found a proof (t != t contradiction)
            if let Some(empty_id) = self.proof_from_back_demod.take() {
                self.clauses_kept += 1;
                return ProofResult::Proof {
                    empty_clause_id: empty_id,
                    clauses_generated: self.clauses_generated,
                    clauses_kept: self.clauses_kept,
                };
            }

            // Check resource limits
            if self.given_count >= self.config.max_given {
                return ProofResult::ResourceLimit {
                    clauses_generated: self.clauses_generated,
                    clauses_kept: self.clauses_kept,
                    limit_type: "max_given".to_string(),
                };
            }

            if self.clauses_kept >= self.config.max_clauses {
                return ProofResult::ResourceLimit {
                    clauses_generated: self.clauses_generated,
                    clauses_kept: self.clauses_kept,
                    limit_type: "max_clauses".to_string(),
                };
            }

            // Check time limit
            if self.config.max_seconds > 0 {
                let elapsed = start_time.elapsed().as_secs();
                if elapsed >= self.config.max_seconds {
                    return ProofResult::ResourceLimit {
                        clauses_generated: self.clauses_generated,
                        clauses_kept: self.clauses_kept,
                        limit_type: "max_seconds".to_string(),
                    };
                }
            }

            // Check memory limit (every 10 iterations to reduce overhead)
            if self.config.max_memory_bytes > 0 && self.given_count % 10 == 0 {
                use sysinfo::{ProcessRefreshKind, System};
                let mut system = System::new();
                if let Ok(pid) = sysinfo::get_current_pid() {
                    system.refresh_process_specifics(pid, ProcessRefreshKind::new().with_memory());
                    if let Some(process) = system.process(pid) {
                        let memory_bytes = process.memory() * 1024; // Convert KB to bytes
                        if memory_bytes as usize > self.config.max_memory_bytes {
                            return ProofResult::ResourceLimit {
                                clauses_generated: self.clauses_generated,
                                clauses_kept: self.clauses_kept,
                                limit_type: format!("max_memory ({} MB used)", memory_bytes / 1024 / 1024),
                            };
                        }
                    }
                }
            }

            // Select given clause using pick_given_ratio strategy
            // Pick by weight for first N clauses, then by FIFO for 1 clause, repeat
            let select_by_weight = self.pick_count < self.config.pick_given_ratio;
            self.pick_count = (self.pick_count + 1) % (self.config.pick_given_ratio + 1);

            let given_id = if select_by_weight {
                self.select_lightest_clause()
            } else {
                self.sos.pop() // FIFO
            };

            let given_id = match given_id {
                Some(id) => id,
                None => break,
            };

            self.given_count += 1;

            // Track clauses generated in this iteration for per-iteration limits
            let clauses_at_iteration_start = self.clauses_generated;
            let iteration_clause_limit = self.config.max_clauses_per_given;

            // Get the given clause
            let given_clause = match self.arena.get(given_id) {
                Some(c) => c.clone(),
                None => continue,
            };

            // Given clause selected (debug output removed for performance)

            // Extract demodulator from given clause if it's a unit equality
            // This is critical for Knuth-Bendix completion
            if self.config.use_demod {
                if let Some(eq_sym) = self.eq_symbol {
                    if let Some(demod) = extract_demodulator(&given_clause, eq_sym, Some(&self.lrpo)) {
                        // Apply back-demodulation with new demodulator
                        if self.config.use_back_demod {
                            self.back_demodulate(&demod);
                            // Check if back-demod found a proof (t != t contradiction)
                            if let Some(empty_id) = self.proof_from_back_demod.take() {
                                self.clauses_kept += 1;
                                return ProofResult::Proof {
                                    empty_clause_id: empty_id,
                                    clauses_generated: self.clauses_generated,
                                    clauses_kept: self.clauses_kept,
                                };
                            }
                        }
                        self.demodulators.push(demod);
                    }
                }
            }

            // Collect usable clauses for inference
            // Build paired list to ensure IDs and clauses stay in sync
            let usable_pairs: Vec<(ClauseId, Clause)> = self.usable
                .iter()
                .filter_map(|id| self.arena.get(*id).cloned().map(|c| (*id, c)))
                .collect();
            let usable_ids: Vec<ClauseId> = usable_pairs.iter().map(|(id, _)| *id).collect();
            let usable_clauses: Vec<Clause> = usable_pairs.into_iter().map(|(_, c)| c).collect();
            let usable_id_opts: Vec<Option<ClauseId>> = usable_ids.iter().map(|id| Some(*id)).collect();

            // Collect SOS clauses snapshot for forward subsumption (C Foras checks both usable AND SOS)
            let sos_clauses_snapshot: Vec<Clause> = self.sos
                .iter()
                .filter_map(|id| self.arena.get(*id).cloned())
                .collect();

            // Perform inference rules within a labeled block to allow early exit on limits
            'given_clause_iteration: {
            // Perform hyperresolution if enabled
            // In hyperresolution, the given clause (positive satellite) is resolved
            // against usable clauses (nuclei with negative literals)
            if self.config.use_hyper_res {
                // Try hyperresolving each usable clause (nucleus) with given + other usable (satellites)
                for (nucleus_idx, nucleus_id) in usable_ids.iter().enumerate() {
                    let nucleus = &usable_clauses[nucleus_idx];

                    // Check if nucleus has negative literals
                    if !nucleus.literals.iter().any(|lit| !lit.sign) {
                        continue;
                    }

                    // Build list of satellites: given clause + other positive units from usable
                    let mut satellites = vec![];
                    let mut satellite_ids = vec![];

                    // Add given clause if it's a positive unit
                    if given_clause.literals.len() == 1 && given_clause.literals[0].sign {
                        satellites.push(given_clause.clone());
                        satellite_ids.push(Some(given_id));
                    }

                    for (sat_idx, sat_id) in usable_ids.iter().enumerate() {
                        if sat_idx != nucleus_idx {
                            let sat = &usable_clauses[sat_idx];
                            // Add positive units as potential satellites
                            if sat.literals.len() == 1 && sat.literals[0].sign {
                                satellites.push(sat.clone());
                                satellite_ids.push(Some(*sat_id));
                            }
                        }
                    }

                    if satellites.is_empty() {
                        continue;
                    }

                    let hyper_resolvents = hyperresolve_units(
                        nucleus,
                        Some(*nucleus_id),
                        &satellites,
                        &satellite_ids,
                    );

                    for resolvent in hyper_resolvents {
                        self.clauses_generated += 1;

                        // Debug: check if resolvent has parents before processing
                        if resolvent.clause.parents.is_empty() {
                            eprintln!("BUG: Hyperresolution produced clause with empty parents!");
                            eprintln!("  Nucleus ID: {:?}", resolvent.nucleus_id);
                            eprintln!("  Satellite IDs: {:?}", resolvent.satellite_ids);
                            eprintln!("  Resolvent: {:?}", resolvent.clause);
                        }

                        // Process the clause (demodulate, extract demodulators)
                        let processed = match self.process_new_clause(resolvent.clause) {
                            Some(c) => c,
                            None => continue,
                        };

                        // Check for proof (empty clause or answer clause)
                        if self.is_proof(&processed) {
                            let empty_id = self.arena.insert(processed);
                            self.clauses_kept += 1;
                            return ProofResult::Proof {
                                empty_clause_id: empty_id,
                                clauses_generated: self.clauses_generated,
                                clauses_kept: self.clauses_kept,
                            };
                        }

                        // Unit deletion: simplify clause using unit clauses
                        let mut final_clause = processed;
                        if self.config.use_unit_deletion {
                            if let Some(unit_deleted) = forward_unit_deletion(
                                &final_clause,
                                None,
                                &usable_clauses,
                                &usable_ids.iter().map(|id| Some(*id)).collect::<Vec<_>>(),
                            ) {
                                final_clause = unit_deleted.clause;
                                // Restore parents
                                for parent in unit_deleted.parents {
                                    final_clause.add_parent(parent);
                                }
                                // Check again for proof after unit deletion
                                if self.is_proof(&final_clause) {
                                    let empty_id = self.arena.insert(final_clause);
                                    self.clauses_kept += 1;
                                    return ProofResult::Proof {
                                        empty_clause_id: empty_id,
                                        clauses_generated: self.clauses_generated,
                                        clauses_kept: self.clauses_kept,
                                    };
                                }
                            }
                        }

                        // Check for tautology after unit deletion (it may have created one)
                        let is_tautology = (0..final_clause.literals.len()).any(|i| {
                            ((i + 1)..final_clause.literals.len()).any(|j| {
                                let lit1 = &final_clause.literals[i];
                                let lit2 = &final_clause.literals[j];
                                lit1.sign != lit2.sign && lit1.atom == lit2.atom
                            })
                        });
                        if is_tautology {
                            eprintln!("TAUTOLOGY after unit deletion: {:?}", final_clause);
                            continue;
                        }

                        // Forward subsumption: check if new clause is subsumed by existing clauses
                        // Check against both usable and SOS (like C Foras)
                        if self.config.use_subsumption {
                            let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                            let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                            if self.is_forward_subsumed(&final_clause, &usable_refs, &sos_refs) {
                                continue; // Skip this clause, it's subsumed
                            }
                        }

                        // Backward subsumption: remove clauses subsumed by this new clause
                        self.perform_back_subsumption(&final_clause);

                        // Add to sos for further processing (with max_weight filtering)
                        self.try_keep_clause(final_clause);
                    }
                }
            }
            // Check if we've exceeded per-iteration clause limit after hyperresolution
            if iteration_clause_limit > 0 && (self.clauses_generated - clauses_at_iteration_start) >= iteration_clause_limit {
                break 'given_clause_iteration;
            }

            // Perform binary resolution if enabled
            if self.config.use_binary_res {
                for (i, usable_id) in usable_ids.iter().enumerate() {
                    let usable_clause = &usable_clauses[i];

                    let resolvents = all_resolvents(
                        &given_clause,
                        usable_clause,
                        Some(given_id),
                        Some(*usable_id),
                    );

                    for resolvent in resolvents {
                        self.clauses_generated += 1;

                        if resolvent.clause.parents.is_empty() {
                            eprintln!("BUG: Binary resolution produced clause with empty parents!");
                            eprintln!("  Clause 1 ID: {:?}", given_id);
                            eprintln!("  Clause 2 ID: {:?}", usable_id);
                            eprintln!("  Resolvent: {:?}", resolvent.clause);
                        }

                        // Process the clause (demodulate, extract demodulators)
                        let processed = match self.process_new_clause(resolvent.clause) {
                            Some(c) => c,
                            None => continue,
                        };

                        // Check for proof (empty clause or answer clause)
                        if self.is_proof(&processed) {
                            let empty_id = self.arena.insert(processed);
                            self.clauses_kept += 1;
                            return ProofResult::Proof {
                                empty_clause_id: empty_id,
                                clauses_generated: self.clauses_generated,
                                clauses_kept: self.clauses_kept,
                            };
                        }

                        // Unit deletion: simplify clause using unit clauses
                        let mut final_clause = processed;
                        if self.config.use_unit_deletion {
                            if let Some(unit_deleted) = forward_unit_deletion(
                                &final_clause,
                                None,
                                &usable_clauses,
                                &usable_ids.iter().map(|id| Some(*id)).collect::<Vec<_>>(),
                            ) {
                                final_clause = unit_deleted.clause;
                                // Restore parents
                                for parent in unit_deleted.parents {
                                    final_clause.add_parent(parent);
                                }
                                // Check again for proof after unit deletion
                                if self.is_proof(&final_clause) {
                                    let empty_id = self.arena.insert(final_clause);
                                    self.clauses_kept += 1;
                                    return ProofResult::Proof {
                                        empty_clause_id: empty_id,
                                        clauses_generated: self.clauses_generated,
                                        clauses_kept: self.clauses_kept,
                                    };
                                }
                            }
                        }

                        // Check for tautology after unit deletion (it may have created one)
                        let is_tautology = (0..final_clause.literals.len()).any(|i| {
                            ((i + 1)..final_clause.literals.len()).any(|j| {
                                let lit1 = &final_clause.literals[i];
                                let lit2 = &final_clause.literals[j];
                                lit1.sign != lit2.sign && lit1.atom == lit2.atom
                            })
                        });
                        if is_tautology {
                            eprintln!("TAUTOLOGY after unit deletion: {:?}", final_clause);
                            continue;
                        }

                        // Forward subsumption: check if new clause is subsumed by existing clauses
                        // Check against both usable and SOS (like C Foras)
                        if self.config.use_subsumption {
                            let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                            let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                            if self.is_forward_subsumed(&final_clause, &usable_refs, &sos_refs) {
                                continue; // Skip this clause, it's subsumed
                            }
                        }

                        // Backward subsumption: remove clauses subsumed by this new clause
                        self.perform_back_subsumption(&final_clause);

                        // Add to sos for further processing (with max_weight filtering)
                        self.try_keep_clause(final_clause);
                    }
                }
            }
            // Check if we've exceeded per-iteration clause limit after binary resolution
            if iteration_clause_limit > 0 && (self.clauses_generated - clauses_at_iteration_start) >= iteration_clause_limit {
                break 'given_clause_iteration;
            }

            // Perform UR-resolution if enabled
            if self.config.use_ur_res {
                // Collect usable clauses for UR-resolution
                let ur_resolvents = ur_resolve(
                    &given_clause,
                    Some(given_id),
                    &usable_clauses,
                    &usable_id_opts,
                );

                for resolvent in ur_resolvents {
                    self.clauses_generated += 1;

                    // Process the clause
                    let processed = match self.process_new_clause(resolvent.clause) {
                        Some(c) => c,
                        None => continue,
                    };

                    // Check for proof (empty clause or answer clause)
                    if self.is_proof(&processed) {
                        let empty_id = self.arena.insert(processed);
                        self.clauses_kept += 1;
                        return ProofResult::Proof {
                            empty_clause_id: empty_id,
                            clauses_generated: self.clauses_generated,
                            clauses_kept: self.clauses_kept,
                        };
                    }

                    // Forward subsumption: check against both usable and SOS
                    if self.config.use_subsumption {
                        let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                        let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                        if self.is_forward_subsumed(&processed, &usable_refs, &sos_refs) {
                            continue;
                        }
                    }

                    // Backward subsumption
                    self.perform_back_subsumption(&processed);

                    // Add to sos (with max_weight filtering)
                    self.try_keep_clause(processed);
                }
            }
            // Check if we've exceeded per-iteration clause limit after UR-resolution
            if iteration_clause_limit > 0 && (self.clauses_generated - clauses_at_iteration_start) >= iteration_clause_limit {
                break 'given_clause_iteration;
            }

            // Perform Linked UR-resolution if enabled
            if self.config.use_linked_ur_res {
                let linked_ur_config = LinkedURConfig::default();
                let linked_ur_resolvents = linked_ur_resolve(
                    &given_clause,
                    Some(given_id),
                    &usable_clauses,
                    &linked_ur_config,
                );

                for resolvent in linked_ur_resolvents {
                    self.clauses_generated += 1;

                    // Process the clause
                    let processed = match self.process_new_clause(resolvent.clause) {
                        Some(c) => c,
                        None => continue,
                    };

                    // Check for proof (empty clause or answer clause)
                    if self.is_proof(&processed) {
                        let empty_id = self.arena.insert(processed);
                        self.clauses_kept += 1;
                        return ProofResult::Proof {
                            empty_clause_id: empty_id,
                            clauses_generated: self.clauses_generated,
                            clauses_kept: self.clauses_kept,
                        };
                    }

                    // Forward subsumption: check against both usable and SOS
                    if self.config.use_subsumption {
                        let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                        let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                        if self.is_forward_subsumed(&processed, &usable_refs, &sos_refs) {
                            continue;
                        }
                    }

                    // Backward subsumption
                    self.perform_back_subsumption(&processed);

                    // Add to sos (with max_weight filtering)
                    self.try_keep_clause(processed);
                }
            }
            // Check if we've exceeded per-iteration clause limit after Linked UR-resolution
            if iteration_clause_limit > 0 && (self.clauses_generated - clauses_at_iteration_start) >= iteration_clause_limit {
                break 'given_clause_iteration;
            }

            // Perform paramodulation if enabled and we have an equality symbol
            if (self.config.use_para_into || self.config.use_para_from) && self.eq_symbol.is_some() {
                let eq_sym = self.eq_symbol.unwrap();

                for (i, usable_id) in usable_ids.iter().enumerate() {
                    let usable_clause = &usable_clauses[i];

                    // Para into: given contains equality, paramodulate into usable
                    if self.config.use_para_into {
                        let paramodulants = paramodulate_into(
                            &given_clause,
                            Some(given_id),
                            usable_clause,
                            Some(*usable_id),
                            eq_sym,
                            self.config.para_from_left,
                            self.config.para_from_right,
                            self.config.para_into_left,
                            self.config.para_into_right,
                        );

                        for paramodulant in paramodulants {
                            self.clauses_generated += 1;

                            // Process the clause (demodulate, extract demodulators)
                            let processed = match self.process_new_clause(paramodulant.clause) {
                                Some(c) => c,
                                None => continue,
                            };

                            if self.is_proof(&processed) {
                                let empty_id = self.arena.insert(processed);
                                self.clauses_kept += 1;
                                return ProofResult::Proof {
                                    empty_clause_id: empty_id,
                                    clauses_generated: self.clauses_generated,
                                    clauses_kept: self.clauses_kept,
                                };
                            }

                            // Forward subsumption: check if new clause is subsumed by existing clauses
                            // Check against both usable and SOS (like C Foras)
                            if self.config.use_subsumption {
                                let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                                let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                                if self.is_forward_subsumed(&processed, &usable_refs, &sos_refs) {
                                    continue; // Skip this clause, it's subsumed
                                }
                            }

                            // Backward subsumption: remove clauses subsumed by this new clause
                            self.perform_back_subsumption(&processed);

                            // Add to sos (with max_weight filtering)
                            self.try_keep_clause(processed);
                        }
                    }

                    // Para from: usable contains equality, paramodulate into given
                    if self.config.use_para_from {
                        let paramodulants = paramodulate_into(
                            usable_clause,
                            Some(*usable_id),
                            &given_clause,
                            Some(given_id),
                            eq_sym,
                            self.config.para_from_left,
                            self.config.para_from_right,
                            self.config.para_into_left,
                            self.config.para_into_right,
                        );

                        for paramodulant in paramodulants {
                            self.clauses_generated += 1;

                            // Process the clause (demodulate, extract demodulators)
                            let processed = match self.process_new_clause(paramodulant.clause) {
                                Some(c) => c,
                                None => continue,
                            };

                            if self.is_proof(&processed) {
                                let empty_id = self.arena.insert(processed);
                                self.clauses_kept += 1;
                                return ProofResult::Proof {
                                    empty_clause_id: empty_id,
                                    clauses_generated: self.clauses_generated,
                                    clauses_kept: self.clauses_kept,
                                };
                            }

                            // Forward subsumption: check if new clause is subsumed by existing clauses
                            // Check against both usable and SOS (like C Foras)
                            if self.config.use_subsumption {
                                let usable_refs: Vec<&Clause> = usable_clauses.iter().collect();
                                let sos_refs: Vec<&Clause> = sos_clauses_snapshot.iter().collect();
                                if self.is_forward_subsumed(&processed, &usable_refs, &sos_refs) {
                                    continue; // Skip this clause, it's subsumed
                                }
                            }

                            // Backward subsumption: remove clauses subsumed by this new clause
                            self.perform_back_subsumption(&processed);

                            // Add to sos (with max_weight filtering)
                            self.try_keep_clause(processed);
                        }
                    }
                }
            }
            // Check if we've exceeded per-iteration clause limit after paramodulation
            if iteration_clause_limit > 0 && (self.clauses_generated - clauses_at_iteration_start) >= iteration_clause_limit {
                break 'given_clause_iteration;
            }
            } // End of 'given_clause_iteration labeled block

            // Move given clause to usable
            self.usable.push(given_id);
        }

        ProofResult::Saturated {
            clauses_generated: self.clauses_generated,
            clauses_kept: self.clauses_kept,
        }
    }

    /// Get the clause arena for inspection.
    pub fn arena(&self) -> &ClauseArena {
        &self.arena
    }

    /// Get statistics about the search.
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.clauses_generated, self.clauses_kept, self.given_count)
    }

    /// Get the prover configuration.
    pub fn config(&self) -> &ProverConfig {
        &self.config
    }

    /// Get a mutable reference to the prover configuration.
    pub fn config_mut(&mut self) -> &mut ProverConfig {
        &mut self.config
    }
}

impl Default for Prover {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::symbol::{SymbolKind, SymbolTable};
    use crate::data::{Literal, Term, VariableId};

    fn make_var(id: u16) -> Term {
        Term::variable(VariableId::new(id))
    }

    fn make_const(table: &SymbolTable, name: &str) -> Term {
        let sym = table.intern(name, 0, SymbolKind::Constant);
        Term::application(sym, vec![])
    }

    fn make_pred(table: &SymbolTable, name: &str, args: Vec<Term>) -> Term {
        let sym = table.intern(name, args.len() as u8, SymbolKind::Predicate);
        Term::application(sym, args)
    }

    #[test]
    fn prove_simple_contradiction() {
        // P(a) and -P(x) should derive empty clause
        let table = SymbolTable::new();
        let a = make_const(&table, "a");
        let x = make_var(0);

        let p_a = make_pred(&table, "P", vec![a]);
        let p_x = make_pred(&table, "P", vec![x]);

        let clause1 = Clause::new(vec![Literal::new(true, p_a)]);
        let clause2 = Clause::new(vec![Literal::new(false, p_x)]);

        // Need to pass table to Prover now
        let mut prover = Prover::with_config(ProverConfig::default(), table);
        prover.add_sos(clause1);
        prover.add_usable(clause2);

        let result = prover.search();
        assert!(matches!(result, ProofResult::Proof { .. }));
    }

    #[test]
    fn saturates_without_proof() {
        // P(a) and Q(b) cannot derive contradiction
        let table = SymbolTable::new();
        let a = make_const(&table, "a");
        let b = make_const(&table, "b");

        let p_a = make_pred(&table, "P", vec![a]);
        let q_b = make_pred(&table, "Q", vec![b]);

        let clause1 = Clause::new(vec![Literal::new(true, p_a)]);
        let clause2 = Clause::new(vec![Literal::new(true, q_b)]);

        let mut prover = Prover::with_config(ProverConfig::default(), table);
        prover.add_sos(clause1);
        prover.add_usable(clause2);

        let result = prover.search();
        assert!(matches!(result, ProofResult::Saturated { .. }));
    }

    #[test]
    fn prove_chain_resolution() {
        // P(a), P(x) -> Q(x), -Q(a) should derive empty clause
        let table = SymbolTable::new();
        let a = make_const(&table, "a");
        let x = make_var(0);
        let y = make_var(1);

        let p_a = make_pred(&table, "P", vec![a.clone()]);
        let _p_x = make_pred(&table, "P", vec![x.clone()]);
        let _q_x = make_pred(&table, "Q", vec![x]);
        let q_a = make_pred(&table, "Q", vec![a]);

        // P(a)
        let clause1 = Clause::new(vec![Literal::new(true, p_a)]);
        // -P(y) | Q(y)
        let p_y = make_pred(&table, "P", vec![y.clone()]);
        let q_y = make_pred(&table, "Q", vec![y]);
        let clause2 = Clause::new(vec![
            Literal::new(false, p_y),
            Literal::new(true, q_y),
        ]);
        // -Q(a)
        let clause3 = Clause::new(vec![Literal::new(false, q_a)]);

        let mut prover = Prover::with_config(ProverConfig::default(), table);
        prover.add_sos(clause1);
        prover.add_usable(clause2);
        prover.add_usable(clause3);

        let result = prover.search();
        assert!(matches!(result, ProofResult::Proof { .. }));
    }

    #[test]
    fn respects_max_given_limit() {
        let config = ProverConfig {
            max_clauses: 10000,
            max_given: 5,
            pick_given_ratio: 4,
            max_seconds: 0,
            auto_mode: false,
            use_hyper_res: false,
            use_binary_res: true,
            use_para_into: false,
            use_para_from: false,
            use_demod: false,
            use_factor: false,
            use_ur_res: false,
            ..Default::default()
        };

        // Create clauses before moving table to prover
        let table = SymbolTable::new();
        let mut clauses = Vec::new();
        for i in 0..10 {
            let a = make_const(&table, &format!("a{}", i));
            let pred = make_pred(&table, "P", vec![a.clone()]);
            let neg_pred = make_pred(&table, "Q", vec![a]);
            clauses.push(Clause::new(vec![
                Literal::new(true, pred),
                Literal::new(true, neg_pred),
            ]));
        }
        
        let mut prover = Prover::with_config(config, table); // Move table here
        for c in clauses {
            prover.add_sos(c);
        }

        let result = prover.search();
        // With max_given=5 and 10 clauses in sos, should hit limit
        match result {
            ProofResult::ResourceLimit { limit_type, .. } => {
                assert_eq!(limit_type, "max_given");
            }
            ProofResult::Saturated { .. } => {
                // Also acceptable - saturated before limit
            }
            _ => panic!("expected ResourceLimit or Saturated"),
        }
    }

    #[test]
    fn weight_based_clause_selection() {
        // Test that weight-based selection picks lighter clauses first
        let table = SymbolTable::new();

        // Create predicates with different weights
        let p_sym = table.intern("P", 1, SymbolKind::Predicate);
        let q_sym = table.intern("Q", 1, SymbolKind::Predicate);
        let a = make_const(&table, "a");

        let mut config = ProverConfig::default();
        config.pick_given_ratio = 100; // Always select by weight for this test

        // Pre-create clauses so we can move table
        let q_a = make_pred(&table, "Q", vec![a.clone()]);
        let p_a = make_pred(&table, "P", vec![a]);
        let c1 = Clause::new(vec![Literal::new(true, q_a)]);
        let c2 = Clause::new(vec![Literal::new(true, p_a)]);

        let mut prover = Prover::with_config(config, table);

        // Set Q to have much higher weight than P
        prover.set_symbol_weight(p_sym, 1);
        prover.set_symbol_weight(q_sym, 100);

        // Add Q(a) first (heavy), then P(a) (light)
        prover.add_sos(c1);
        prover.add_sos(c2);

        // Select first clause - should be P(a) (lighter) not Q(a) (heavier)
        let first_id = prover.select_lightest_clause().unwrap();
        let first_clause = prover.arena.get(first_id).unwrap();

        // Check that the selected clause is P(a) by verifying its predicate symbol
        if let Term::Application { symbol, .. } = &first_clause.literals[0].atom {
            assert_eq!(*symbol, p_sym, "Should select P(a) with weight 2, not Q(a) with weight 101");
        } else {
            panic!("Expected Application term");
        }
    }
}

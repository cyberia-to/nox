// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jets — Layer 3 optimized implementations (specs/jets/)
//!
//! Jets are observationally equivalent to pure Layer 1 nox formulas.
//! They are NOT new patterns with dedicated tags — tags 0-17 are the
//! complete Layer 1+2 pattern set. Jets are recognized by formula hash:
//!
//!   jet_registry: H(formula_noun) → jet_implementation
//!
//! At reduction time, before tag-dispatching, reduce_inner checks whether
//! H(formula) is in the jet registry. If yes the jet runs; if no the formula
//! reduces normally through Layer 1 patterns. Remove all jets: identical
//! results, orders of magnitude slower. See specs/jets.md.

pub mod registry;
pub mod formulas;
pub mod backends;

pub mod merkle_verify;
pub mod poly_eval;
pub mod fri_fold;
pub mod ntt;
pub mod state;
pub mod decider;

#[cfg(feature = "brakedown")]
pub mod poly_eval_brakedown;

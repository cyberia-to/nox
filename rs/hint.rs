// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! hint — non-deterministic witness injection (Layer 2)
//!
//! the prover provides a witness; Layer 1 constraints validate it.
//! the verifier never calls provide() — it checks the zheng proof.

use nebu::Goldilocks;
use crate::noun::{Order, NounId};

/// hint provider trait — the prover's interface to inject witnesses
pub trait HintProvider<const N: usize> {
    /// provide a witness for the given tag and object
    ///
    /// tag: field element identifying WHICH hint (e.g., 0x01 = private key)
    /// object: the data the formula operates on
    ///
    /// returns Some(witness_noun) or None (= halt, prover doesn't know)
    fn provide(&self, order: &mut Order<N>, tag: Goldilocks, object: NounId) -> Option<NounId>;
}

/// null hint provider — always returns None (no hints available)
/// used for pure Layer 1 execution without privacy/search
pub struct NullHints;

impl<const N: usize> HintProvider<N> for NullHints {
    fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        None
    }
}

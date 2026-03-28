// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! call — non-deterministic witness injection (Layer 2)
//!
//! the prover provides a witness; Layer 1 constraints validate it.
//! the verifier never calls provide() — it checks the zheng proof.

use nebu::Goldilocks;
use crate::noun::{Order, NounId};

/// call provider trait — the prover's interface to inject witnesses
pub trait CallProvider<const N: usize> {
    /// provide a witness for the given tag and object
    ///
    /// tag: field element identifying WHICH call (e.g., 0x01 = private key)
    /// object: the data the formula operates on
    ///
    /// returns Some(witness_noun) or None (= halt, prover doesn't know)
    fn provide(&self, order: &mut Order<N>, tag: Goldilocks, object: NounId) -> Option<NounId>;
}

/// null call provider — always returns None (no calls available)
/// used for pure Layer 1 execution without privacy/search
pub struct NullCalls;

impl<const N: usize> CallProvider<N> for NullCalls {
    fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        None
    }
}

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
use crate::noun::{Arena, NounRef};

/// hint provider trait — the prover's interface to inject witnesses
pub trait HintProvider<const N: usize> {
    /// provide a witness for the given tag and subject
    ///
    /// tag: field element identifying WHICH hint (e.g., 0x01 = private key)
    /// subject: the current computation subject
    ///
    /// returns Some(witness_noun) or None (= halt, prover doesn't know)
    fn provide(&self, arena: &mut Arena<N>, tag: Goldilocks, subject: NounRef) -> Option<NounRef>;
}

/// null hint provider — always returns None (no hints available)
/// used for pure Layer 1 execution without privacy/search
pub struct NullHints;

impl<const N: usize> HintProvider<N> for NullHints {
    fn provide(&self, _arena: &mut Arena<N>, _tag: Goldilocks, _subject: NounRef) -> Option<NounRef> {
        None
    }
}

// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! call — non-deterministic witness injection (Layer 2)
//! look — deterministic state read (BBG polynomial evaluation)
//!
//! the prover provides a witness; Layer 1 constraints validate it.
//! the verifier never calls provide() — it checks the zheng proof.
//! look reads committed polynomial state — same inputs always produce same output.

use nebu::Goldilocks;
use crate::noun::{Order, NounId};

/// look provider trait — deterministic BBG state reads
///
/// namespace: evaluation dimension of BBG_poly (0..9)
/// key: evaluation point within that dimension
///
/// returns Some(value) or None (= lookup unavailable)
pub trait LookProvider {
    fn look(&self, namespace: Goldilocks, key: Goldilocks) -> Option<Goldilocks>;
}

/// null look provider — always returns None (no BBG state available)
/// used for testing without an authenticated state layer
pub struct NullLooks;

impl LookProvider for NullLooks {
    fn look(&self, _namespace: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
        None
    }
}

/// call provider trait — the prover's interface to inject witnesses
///
/// extends LookProvider: every call provider also supports deterministic lookups.
/// NullCalls provides both (returning None for both).
pub trait CallProvider<const N: usize>: LookProvider {
    /// provide a witness for the given tag and object
    ///
    /// tag: field element identifying WHICH call (e.g., 0x01 = private key)
    /// object: the data the formula operates on
    ///
    /// returns Some(witness_noun) or None (= halt, prover doesn't know)
    fn provide(&self, order: &mut Order<N>, tag: Goldilocks, object: NounId) -> Option<NounId>;
}

/// null call provider — always returns None (no calls or lookups available)
/// used for pure Layer 1 execution without privacy/search/BBG
pub struct NullCalls;

impl LookProvider for NullCalls {
    fn look(&self, _namespace: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
        None
    }
}

impl<const N: usize> CallProvider<N> for NullCalls {
    fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        None
    }
}

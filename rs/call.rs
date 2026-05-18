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
/// commitment: BBG root field element extracted from the object at BBG_ROOT_AXIS
/// namespace:  evaluation dimension of BBG_poly (0..9)
/// key:        evaluation point within that dimension
///
/// returns Some(value) or None (= lookup unavailable, e.g. BBG not connected)
pub trait LookProvider {
    fn look(&self, commitment: Goldilocks, namespace: Goldilocks, key: Goldilocks) -> Option<Goldilocks>;
}

/// call provider trait — the prover's interface to inject witnesses
///
/// extends LookProvider: every call provider also supports deterministic lookups.
/// NullCalls provides both (returning None for both).
///
/// `Sync` is required because the `std` feature runs binary sub-formulas on
/// separate threads, each holding a `&dyn CallProvider<N>` read-only reference.
/// All current implementations (NullCalls, test stubs) are unit structs and
/// trivially `Sync`. User-defined providers must not hold non-Sync interior state.
pub trait CallProvider<const N: usize>: LookProvider + Sync {
    /// provide a witness for the given tag and object
    ///
    /// tag: field element identifying WHICH call (e.g., 0x01 = private key)
    /// object: the data the formula operates on
    ///
    /// returns Some(witness_noun) or None (= halt, prover doesn't know)
    fn provide(&self, order: &mut Order<N>, tag: Goldilocks, object: NounId) -> Option<NounId>;

    /// Return the 32-byte Lens commitment for the noun polynomial of `object_id`.
    ///
    /// Called by pattern 0 (axis) to populate r[11]-r[14] with the commitment
    /// bytes, enabling zheng to circuit-constrain the opening. Proof-unaware
    /// callers (interpreter, tests) return None — registers stay zero.
    fn axis_commitment(&self, _object_id: u64) -> Option<[u8; 32]> {
        None
    }
}

/// null provider — always returns None for both call and look.
/// used for pure Layer 1 execution and tests without privacy/BBG.
pub struct NullCalls;

/// alias kept for spec-side terminology; same type as NullCalls.
pub type NullLooks = NullCalls;

impl LookProvider for NullCalls {
    fn look(&self, _commitment: Goldilocks, _namespace: Goldilocks, _key: Goldilocks) -> Option<Goldilocks> {
        None
    }
}

impl<const N: usize> CallProvider<N> for NullCalls {
    fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        None
    }
}

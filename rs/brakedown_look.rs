// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! BrakedownLookProvider — Brakedown-backed BBG state provider.
//!
//! Holds a flat evaluation table (MultilinearPoly) as the BBG polynomial.
//! `look(commitment, namespace, key)` returns the value at `key` as an index
//! into the evaluation table and accumulates the opening for batch proof
//! generation.
//!
//! Only available with the `brakedown` feature (implies `std`).

extern crate alloc;

use std::sync::Mutex;
use alloc::vec::Vec;

use cyber_lens_brakedown::Brakedown;
use cyber_lens_core::{Commitment, Lens, MultilinearPoly};
use nebu::Goldilocks;

use crate::data::{Reduction, Order};
use crate::call::{CallProvider, LookProvider};

/// A pending opening for batch proof generation.
pub struct LookOpening {
    pub namespace: Goldilocks,
    pub key: Goldilocks,
    pub value: Goldilocks,
}

/// Brakedown-backed look provider.
///
/// The provider holds a multilinear polynomial and its Brakedown commitment.
/// `look` returns `poly.evals[key as usize]` and records the opening.
/// The `commitment_field` is the first 8 bytes of the Brakedown hash,
/// interpreted as a little-endian u64 — the same value stored in BBG root limb
/// l0 (axis 4 of the object data).
pub struct BrakedownLookProvider {
    poly: MultilinearPoly<Goldilocks>,
    commitment: Commitment,
    commitment_field: Goldilocks,
    openings: Mutex<Vec<LookOpening>>,
}

impl BrakedownLookProvider {
    pub fn new(poly: MultilinearPoly<Goldilocks>) -> Self {
        let commitment = Brakedown::commit(&poly);
        let bytes = commitment.as_bytes();
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[0..8]);
        let commitment_field = Goldilocks::new(u64::from_le_bytes(buf));
        Self { poly, commitment, commitment_field, openings: Mutex::new(Vec::new()) }
    }

    /// Drain accumulated openings for batch proof generation.
    pub fn drain_openings(&self) -> Vec<LookOpening> {
        let mut guard = self.openings.lock().unwrap();
        core::mem::take(&mut *guard)
    }

    /// The polynomial committed at construction time.
    pub fn poly(&self) -> &MultilinearPoly<Goldilocks> { &self.poly }

    /// The Brakedown commitment to `poly`.
    pub fn commitment(&self) -> &Commitment { &self.commitment }

    pub fn commitment_field(&self) -> Goldilocks { self.commitment_field }
}

impl LookProvider for BrakedownLookProvider {
    fn look(&self, _commitment: Goldilocks, namespace: Goldilocks, key: Goldilocks) -> Option<Goldilocks> {
        let idx = key.as_u64() as usize;
        if idx >= self.poly.evals.len() {
            return None;
        }
        let value = self.poly.evals[idx];
        self.openings.lock().unwrap().push(LookOpening { namespace, key, value });
        Some(value)
    }
}

impl<const N: usize> CallProvider<N> for BrakedownLookProvider {
    fn provide(&self, _order: &mut Reduction<N>, _tag: Goldilocks, _object: Order) -> Option<Order> {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::reduce::{reduce, Outcome};
    use crate::trace::NoTrace;
    use crate::data::{Reduction};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// Build object data `[[l0|[l1|[l2|l3]]]|rest]` with the BBG root limbs.
    fn make_object<const N: usize>(
        reduction: &mut Reduction<N>,
        l0: Goldilocks, l1: Goldilocks, l2: Goldilocks, l3: Goldilocks,
        rest: Order,
    ) -> Order {
        let a0 = reduction.atom(l0).unwrap();
        let a1 = reduction.atom(l1).unwrap();
        let a2 = reduction.atom(l2).unwrap();
        let a3 = reduction.atom(l3).unwrap();
        let inner = reduction.pair(a2, a3).unwrap();
        let mid = reduction.pair(a1, inner).unwrap();
        let root_pair = reduction.pair(a0, mid).unwrap();
        reduction.pair(root_pair, rest).unwrap()
    }

    /// Build formula [17 | [[1|ns] | [1|key]]] — look with quoted ns and key.
    fn make_look<const N: usize>(reduction: &mut Reduction<N>, ns: u64, key: u64) -> Order {
        let t17 = reduction.atom(g(17)).unwrap();
        let t1  = reduction.atom(g(1)).unwrap();
        let vns  = reduction.atom(g(ns)).unwrap();
        let vkey = reduction.atom(g(key)).unwrap();
        let ns_formula  = reduction.pair(t1, vns).unwrap();
        let key_formula = reduction.pair(t1, vkey).unwrap();
        let body = reduction.pair(ns_formula, key_formula).unwrap();
        reduction.pair(t17, body).unwrap()
    }

    #[test]
    fn look_returns_table_value() {
        let evals = alloc::vec![g(10), g(20), g(30), g(40)];
        let provider = BrakedownLookProvider::new(MultilinearPoly::new(evals));
        let l0 = provider.commitment_field();

        let mut ar = Reduction::<512>::new();
        let rest = ar.atom(g(0)).unwrap();
        let object = make_object(&mut ar, l0, g(0), g(0), g(0), rest);
        // key=2 → evals[2] = 30
        let formula = make_look(&mut ar, 0, 2);

        match reduce(&mut ar, object, formula, 10_000, &provider, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                let v = ar.atom_value(r).unwrap();
                assert_eq!(v, g(30));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn drain_openings_cleared_after_drain() {
        let evals = alloc::vec![g(1), g(2)];
        let provider = BrakedownLookProvider::new(MultilinearPoly::new(evals));
        provider.look(g(0), g(0), g(0));
        provider.look(g(0), g(0), g(1));
        let ops = provider.drain_openings();
        assert_eq!(ops.len(), 2);
        assert!(provider.drain_openings().is_empty());
    }

    #[test]
    fn look_out_of_bounds_returns_none() {
        let evals = alloc::vec![g(1), g(2)];
        let provider = BrakedownLookProvider::new(MultilinearPoly::new(evals));
        assert!(provider.look(g(0), g(0), g(99)).is_none());
    }
}

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

use cyb_lens_brakedown::Brakedown;
use cyb_lens_core::{Lens, MultilinearPoly};
use nebu::Goldilocks;

use crate::noun::{Order, NounId};
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
/// l0 (axis 4 of the object noun).
pub struct BrakedownLookProvider {
    poly: MultilinearPoly<Goldilocks>,
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
        Self { poly, commitment_field, openings: Mutex::new(Vec::new()) }
    }

    /// Drain accumulated openings for batch proof generation.
    pub fn drain_openings(&self) -> Vec<LookOpening> {
        let mut guard = self.openings.lock().unwrap();
        core::mem::take(&mut *guard)
    }

    pub fn commitment_field(&self) -> Goldilocks {
        self.commitment_field
    }
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
    fn provide(&self, _order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::reduce::{reduce, Outcome};
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// Build object noun `[[l0|[l1|[l2|l3]]]|rest]` with the BBG root limbs.
    fn make_object<const N: usize>(
        order: &mut Order<N>,
        l0: Goldilocks, l1: Goldilocks, l2: Goldilocks, l3: Goldilocks,
        rest: NounId,
    ) -> NounId {
        let a0 = order.atom(l0, Tag::Field).unwrap();
        let a1 = order.atom(l1, Tag::Field).unwrap();
        let a2 = order.atom(l2, Tag::Field).unwrap();
        let a3 = order.atom(l3, Tag::Field).unwrap();
        let inner = order.cell(a2, a3).unwrap();
        let mid = order.cell(a1, inner).unwrap();
        let root_cell = order.cell(a0, mid).unwrap();
        order.cell(root_cell, rest).unwrap()
    }

    /// Build formula [17 | [[1|ns] | [1|key]]] — look with quoted ns and key.
    fn make_look<const N: usize>(order: &mut Order<N>, ns: u64, key: u64) -> NounId {
        let t17 = order.atom(g(17), Tag::Field).unwrap();
        let t1  = order.atom(g(1),  Tag::Field).unwrap();
        let vns  = order.atom(g(ns),  Tag::Field).unwrap();
        let vkey = order.atom(g(key), Tag::Field).unwrap();
        let ns_formula  = order.cell(t1, vns).unwrap();
        let key_formula = order.cell(t1, vkey).unwrap();
        let body = order.cell(ns_formula, key_formula).unwrap();
        order.cell(t17, body).unwrap()
    }

    #[test]
    fn look_returns_table_value() {
        let evals = alloc::vec![g(10), g(20), g(30), g(40)];
        let provider = BrakedownLookProvider::new(MultilinearPoly::new(evals));
        let l0 = provider.commitment_field();

        let mut ar = Order::<512>::new();
        let rest = ar.atom(g(0), Tag::Field).unwrap();
        let object = make_object(&mut ar, l0, g(0), g(0), g(0), rest);
        // key=2 → evals[2] = 30
        let formula = make_look(&mut ar, 0, 2);

        match reduce(&mut ar, object, formula, 10_000, &provider, &mut NoTrace) {
            Outcome::Ok(r, _) => {
                let (v, _) = ar.atom_value(r).unwrap();
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

// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: fri_fold — FRI polynomial folding over Goldilocks
//!
//! Folds a balanced binary tree of 2^k evaluations to a single value using
//! challenge r. Each level applies the linear interpolation:
//!   result = left + r * (right − left)
//!
//! Registry calling convention:
//!   object = [[k | formula] | [evals_tree | r]]
//!   k          = number of fold levels (log2 of evaluation count)
//!   formula    = self-reference (jet ignores)
//!   evals_tree = balanced binary tree of 2^k field atoms
//!   r          = field challenge (same for every level)
//!
//! Budget: 2^k − 1 (one operation per pair merged across all levels).

extern crate alloc;
use alloc::vec::Vec;

use nebu::Goldilocks;
use crate::noun::{Order, NounId, Noun, Tag};
use crate::reduce::{Outcome, ErrorKind, cell_pair};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn fri_fold_jet<const N: usize>(
    order: &mut Order<N>, object: NounId, _body: NounId, budget: u64,
    _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [[k | formula] | [evals_tree | r]]
    let (lhs, rhs) = match cell_pair(order, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (k_id, _formula_id) = match cell_pair(order, lhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (evals_id, r_id) = match cell_pair(order, rhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };

    let k = match order.atom_value(k_id) {
        Some((v, _)) => v.as_u64() as usize,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let r = match order.atom_value(r_id) {
        Some((v, _)) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };

    let mut evals: Vec<Goldilocks> = Vec::new();
    if !flatten_tree(order, evals_id, &mut evals) {
        return Outcome::Error(ErrorKind::TypeError);
    }
    let expected = 1usize << k;
    if evals.len() != expected {
        return Outcome::Error(ErrorKind::TypeError);
    }

    // Budget: 2^k − 1 folding operations
    let cost = (expected as u64).saturating_sub(1);
    if budget < cost {
        return Outcome::Halt(budget);
    }
    let remaining = budget - cost;

    // Iterative fold: each round halves the evaluation count.
    while evals.len() > 1 {
        let half = evals.len() / 2;
        let mut next = Vec::with_capacity(half);
        for i in 0..half {
            let left  = evals[2 * i];
            let right = evals[2 * i + 1];
            next.push(left + r * (right - left));
        }
        evals = next;
    }

    let value = evals[0];

    row.r[4] = evals_id as u64;
    row.r[5] = r_id as u64;
    row.r[6] = value.as_u64();

    match order.atom(value, Tag::Field) {
        Some(result) => Outcome::Ok(result, remaining),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

fn flatten_tree<const N: usize>(order: &Order<N>, id: NounId, out: &mut Vec<Goldilocks>) -> bool {
    let inner = match order.get(id) {
        Some(e) => e.inner,
        None => return false,
    };
    match inner {
        Noun::Atom { .. } => match order.atom_value(id) {
            Some((v, _)) => { out.push(v); true }
            None => false,
        },
        Noun::Cell { left, right } => {
            flatten_tree(order, left, out) && flatten_tree(order, right, out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::Outcome;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, TraceRow};
    use crate::noun::{Order, Tag};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn build_tree<const N: usize>(ar: &mut Order<N>, vals: &[Goldilocks]) -> NounId {
        assert!(vals.len().is_power_of_two() && !vals.is_empty());
        if vals.len() == 1 {
            return ar.atom(vals[0], Tag::Field).unwrap();
        }
        let mid = vals.len() / 2;
        let l = build_tree(ar, &vals[..mid]);
        let r = build_tree(ar, &vals[mid..]);
        ar.cell(l, r).unwrap()
    }

    fn run<const N: usize>(
        ar: &mut Order<N>, evals: &[Goldilocks], r: Goldilocks,
    ) -> Outcome {
        let k_val = evals.len().trailing_zeros() as u64;
        let k_id  = ar.atom(g(k_val), Tag::Field).unwrap();
        let dummy = ar.atom(g(0), Tag::Field).unwrap();
        let lhs   = ar.cell(k_id, dummy).unwrap();
        let tree  = build_tree(ar, evals);
        let r_id  = ar.atom(r, Tag::Field).unwrap();
        let rhs   = ar.cell(tree, r_id).unwrap();
        let obj   = ar.cell(lhs, rhs).unwrap();
        let body  = ar.atom(g(0), Tag::Field).unwrap();
        let mut row = TraceRow::default();
        fri_fold_jet(ar, obj, body, 100_000, &NullCalls, &mut NoTrace, 0, &mut row)
    }

    #[test]
    fn single_element_returns_itself() {
        let mut ar = Order::<256>::new();
        // k=0, evals=[7], any r → result=7
        match run(&mut ar, &[g(7)], g(3)) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(7)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn two_elements_at_r_zero_returns_left() {
        // r=0: result = left + 0*(right-left) = left
        let mut ar = Order::<256>::new();
        match run(&mut ar, &[g(3), g(7)], g(0)) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(3)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn two_elements_at_r_one_returns_right() {
        // r=1: result = left + 1*(right-left) = right
        let mut ar = Order::<256>::new();
        match run(&mut ar, &[g(3), g(7)], g(1)) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(7)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn four_elements_fold_two_levels() {
        // evals=[0,4,8,12], r=1/2 (half in Goldilocks)
        // level-1: [(0+4)/2, (8+12)/2] = [2, 10]
        // level-2: (2+10)/2 = 6
        let mut ar = Order::<512>::new();
        let p = 0xFFFF_FFFF_0000_0001u64;
        let half = Goldilocks::new((p + 1) / 2);
        match run(&mut ar, &[g(0), g(4), g(8), g(12)], half) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(6)),
            o => panic!("{:?}", o),
        }
    }
}

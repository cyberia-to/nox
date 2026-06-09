// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: poly_eval — multilinear polynomial evaluation (Goldilocks field)
//!
//! Registry calling convention:
//!   object = [[k | formula] | [evals_tree | point]]
//!   k          = number of variables (log2 of evaluation count)
//!   formula    = self-reference used by the pure L1 formula (jet ignores)
//!   evals_tree = balanced binary tree of 2^k field atoms
//!   point      = right-nested cons list of k field coordinates (big-endian: MSV first)
//!
//! Budget: 2^k (one unit per leaf evaluation retrieved).
//! Point encoding: axis 14 = x_{k-1} (most-significant variable), axis 15 = rest.

extern crate alloc;
use alloc::vec::Vec;

use nebu::Goldilocks;
use crate::data::{Reduction, Order, Data};
use crate::reduce::{Outcome, ErrorKind, pair_children};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn poly_eval_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [[k | formula] | [evals_tree | point]]
    let (lhs, rhs) = match pair_children(reduction, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (k_id, _formula_id) = match pair_children(reduction, lhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (evals_id, point_id) = match pair_children(reduction, rhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };

    let k = match reduction.atom_value(k_id) {
        Some(v) => v.as_u64() as usize,
        None => return Outcome::Error(ErrorKind::TypeError),
    };

    // Flatten the balanced binary tree of evaluations into a Vec.
    let mut evals: Vec<Goldilocks> = Vec::new();
    if !flatten_tree(reduction, evals_id, &mut evals) {
        return Outcome::Error(ErrorKind::TypeError);
    }
    let expected = 1usize << k;
    if evals.len() != expected {
        return Outcome::Error(ErrorKind::TypeError);
    }

    // Decode k coordinates from the right-nested point list (big-endian, MSV first).
    let mut point: Vec<Goldilocks> = Vec::with_capacity(k);
    let mut cur = point_id;
    for _ in 0..k {
        match pair_children(reduction, cur) {
            Some((head, tail)) => match reduction.atom_value(head) {
                Some(v) => { point.push(v); cur = tail; }
                None => return Outcome::Error(ErrorKind::TypeError),
            },
            None => return Outcome::Error(ErrorKind::TypeError),
        }
    }

    let cost = expected as u64;
    if budget < cost {
        return Outcome::Halt(budget);
    }
    let remaining = budget - cost;

    let value = multilinear_eval(&evals, &point);

    row.r[4] = evals_id as u64;
    row.r[5] = point_id as u64;
    row.r[6] = value.as_u64();

    match reduction.atom(value) {
        Some(r) => Outcome::Ok(r, remaining),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

/// Flatten a balanced binary tree of field atoms into a Vec (left-to-right DFS).
fn flatten_tree<const N: usize>(reduction: &Reduction<N>, id: Order, out: &mut Vec<Goldilocks>) -> bool {
    let inner = match reduction.get(id) {
        Some(e) => e.inner,
        None => return false,
    };
    match inner {
        Data::Atom { .. } => match reduction.atom_value(id) {
            Some(v) => { out.push(v); true }
            None => false,
        },
        Data::Pair { left, right } => {
            flatten_tree(reduction, left, out) && flatten_tree(reduction, right, out)
        }
    }
}

/// Multilinear evaluation over a balanced binary tree of evaluations.
///
/// point[0] = most-significant variable (x_{k-1}): splits the tree left/right.
/// evals[..mid] = x_{k-1}=0 half, evals[mid..] = x_{k-1}=1 half.
/// result = vl + x_{k-1}*(vr - vl).
fn multilinear_eval(evals: &[Goldilocks], point: &[Goldilocks]) -> Goldilocks {
    if point.is_empty() {
        return evals[0];
    }
    let mid = evals.len() / 2;
    let vl = multilinear_eval(&evals[..mid], &point[1..]);
    let vr = multilinear_eval(&evals[mid..], &point[1..]);
    vl + point[0] * (vr - vl)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::Outcome;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, TraceRow};
    use crate::data::{Reduction};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// Build a balanced binary tree from evaluations (power-of-two length).
    fn build_tree<const N: usize>(ar: &mut Reduction<N>, vals: &[Goldilocks]) -> Order {
        assert!(vals.len().is_power_of_two() && !vals.is_empty());
        if vals.len() == 1 {
            return ar.atom(vals[0]).unwrap();
        }
        let mid = vals.len() / 2;
        let left  = build_tree(ar, &vals[..mid]);
        let right = build_tree(ar, &vals[mid..]);
        ar.pair(left, right).unwrap()
    }

    /// Build a right-nested point list [x0 | [x1 | ... term]] with atom terminator.
    fn build_point<const N: usize>(ar: &mut Reduction<N>, coords: &[Goldilocks]) -> Order {
        let term = ar.atom(g(0)).unwrap();
        coords.iter().rev().fold(term, |acc, &c| {
            let h = ar.atom(c).unwrap();
            ar.pair(h, acc).unwrap()
        })
    }

    /// Build full calling object and run poly_eval_jet.
    fn run<const N: usize>(
        ar: &mut Reduction<N>, evals: &[Goldilocks], point: &[Goldilocks],
    ) -> Outcome {
        let k = evals.len().trailing_zeros() as u64;
        let k_id  = ar.atom(g(k)).unwrap();
        let dummy = ar.atom(g(0)).unwrap(); // formula placeholder
        let lhs   = ar.pair(k_id, dummy).unwrap();
        let tree  = build_tree(ar, evals);
        let pt    = build_point(ar, point);
        let rhs   = ar.pair(tree, pt).unwrap();
        let obj   = ar.pair(lhs, rhs).unwrap();
        let body  = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        poly_eval_jet(ar, obj, body, 100_000, &NullCalls, &mut NoTrace, 0, &mut row)
    }

    #[test]
    fn at_zero_returns_first_eval() {
        let mut ar = Reduction::<256>::new();
        match run(&mut ar, &[g(3), g(7)], &[g(0)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(3)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn at_one_returns_second_eval() {
        let mut ar = Reduction::<256>::new();
        match run(&mut ar, &[g(3), g(7)], &[g(1)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(7)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn two_vars_at_origin() {
        // f(x0,x1) with evals [10,20,30,40]; evaluate at x1=0, x0=0 → 10
        let mut ar = Reduction::<512>::new();
        match run(&mut ar, &[g(10), g(20), g(30), g(40)], &[g(0), g(0)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(10)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn two_vars_at_one_one() {
        // f(1,1) = evals[3] = 40
        let mut ar = Reduction::<512>::new();
        match run(&mut ar, &[g(10), g(20), g(30), g(40)], &[g(1), g(1)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(40)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn budget_exhaustion() {
        let mut ar = Reduction::<256>::new();
        // k=1 needs cost=2, pass budget=1 → Halt
        let k_id  = ar.atom(g(1)).unwrap();
        let dummy = ar.atom(g(0)).unwrap();
        let lhs   = ar.pair(k_id, dummy).unwrap();
        let e0    = ar.atom(g(3)).unwrap();
        let e1    = ar.atom(g(7)).unwrap();
        let tree  = ar.pair(e0, e1).unwrap();
        let term  = ar.atom(g(0)).unwrap();
        let pt_h  = ar.atom(g(0)).unwrap();
        let pt    = ar.pair(pt_h, term).unwrap();
        let rhs   = ar.pair(tree, pt).unwrap();
        let obj   = ar.pair(lhs, rhs).unwrap();
        let body  = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match poly_eval_jet(&mut ar, obj, body, 1, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}

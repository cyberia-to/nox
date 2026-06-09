// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: merkle_verify — nox-native Merkle path authentication
//!
//! Registry calling convention:
//!   object = [[depth | [root | formula]] | [current | path]]
//!   depth   = number of path steps remaining
//!   root    = expected root hash-data
//!   formula = self-reference (jet ignores)
//!   current = current hash-data (leaf at start)
//!   path    = right-nested cons list of [sibling_hash_data | dir_atom] pairs,
//!             terminated by any atom
//!   dir     = 0: sibling is left child; 1: sibling is right child
//!
//! Returns: Field atom 1 (valid path) or 0 (invalid).
//! Budget: depth × 25 (one hash per step, matching COSTS[hash] = 25).

extern crate alloc;
use alloc::vec::Vec;

use hemera::StepSponge;
use hemera::field::Goldilocks as HemeraGold;
use nebu::Goldilocks;
use crate::data::{Reduction, Order, Data};
use crate::data::hash::hash_pair;
use crate::reduce::{Outcome, ErrorKind, pair_children, make_field};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn merkle_verify_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [[depth | [root | formula]] | [current | path]]
    let (meta, data) = match pair_children(reduction, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (depth_id, root_formula) = match pair_children(reduction, meta) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (root_id, _formula_id) = match pair_children(reduction, root_formula) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (current_id, path_id) = match pair_children(reduction, data) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };

    let depth = match reduction.atom_value(depth_id) {
        Some(v) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::TypeError),
    };

    // Budget: depth × 25 (hash cost per step)
    let cost = depth.saturating_mul(25);
    if budget < cost {
        return Outcome::Halt(budget);
    }
    let remaining = budget - cost;

    let path = match decode_path(reduction, path_id) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::TypeError),
    };

    let valid = verify_path(reduction, current_id, &path, root_id);
    let result_val = if valid { Goldilocks::ONE } else { Goldilocks::ZERO };

    row.r[4] = root_id as u64;
    row.r[5] = current_id as u64;
    row.r[6] = path_id as u64;
    row.r[7] = valid as u64;

    make_field(reduction, result_val, remaining)
}

fn decode_path<const N: usize>(reduction: &Reduction<N>, mut id: Order) -> Option<Vec<(Order, u64)>> {
    let mut steps = Vec::new();
    loop {
        let inner = reduction.get(id)?.inner;
        match inner {
            Data::Atom { .. } => break,
            Data::Pair { left, right } => {
                let (sibling_id, dir_id) = pair_children(reduction, left)?;
                let dir_val = reduction.atom_value(dir_id)?;
                steps.push((sibling_id, dir_val.as_u64()));
                id = right;
            }
        }
    }
    Some(steps)
}

/// Verify a Merkle path using nox-native hash semantics.
///
/// Each step: new_current = Poseidon2(hash_pair(dig_sib, dig_cur)) or reversed.
/// Matches what pattern 15 applied to cons(sibling, current) computes.
fn verify_path<const N: usize>(
    reduction: &mut Reduction<N>,
    leaf_id: Order,
    path: &[(Order, u64)],
    root_id: Order,
) -> bool {
    let mut current_id = leaf_id;
    for &(sibling_id, dir) in path {
        let sib_dig = match reduction.digest(sibling_id) {
            Some(d) => *d,
            None => return false,
        };
        let cur_dig = match reduction.digest(current_id) {
            Some(d) => *d,
            None => return false,
        };
        let pair_dig = if dir == 0 {
            hash_pair(&sib_dig, &cur_dig)
        } else {
            hash_pair(&cur_dig, &sib_dig)
        };
        let new_hash = poseidon2_digest(&pair_dig);
        current_id = match reduction.hash_data(&new_hash) {
            Some(id) => id,
            None => return false,
        };
    }
    let cur_dig  = match reduction.digest(current_id) { Some(d) => *d, None => return false };
    let root_dig = match reduction.digest(root_id)    { Some(d) => *d, None => return false };
    cur_dig == root_dig
}

fn poseidon2_digest(input: &[Goldilocks; 4]) -> [Goldilocks; 4] {
    let mut rate = [HemeraGold::ZERO; 8];
    for i in 0..4 {
        rate[i] = HemeraGold::new(input[i].as_u64());
    }
    let mut sponge = StepSponge::absorb(&rate);
    let mut state  = [HemeraGold::ZERO; 16];
    while !sponge.done() {
        state = sponge.step();
    }
    [
        Goldilocks::new(state[0].as_canonical_u64()),
        Goldilocks::new(state[1].as_canonical_u64()),
        Goldilocks::new(state[2].as_canonical_u64()),
        Goldilocks::new(state[3].as_canonical_u64()),
    ]
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::reduce::Outcome;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, TraceRow};
    use crate::data::{Reduction};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn nox_hash<const N: usize>(ar: &mut Reduction<N>, id: Order) -> Order {
        let dig = *ar.digest(id).unwrap();
        let out = poseidon2_digest(&dig);
        ar.hash_data(&out).unwrap()
    }

    fn two_leaf_tree<const N: usize>(ar: &mut Reduction<N>) -> (Order, Order, Order) {
        let leaf0 = ar.atom(g(100)).unwrap();
        let leaf1 = ar.atom(g(200)).unwrap();
        let hn0   = nox_hash(ar, leaf0);
        let hn1   = nox_hash(ar, leaf1);
        let cons  = ar.pair(hn0, hn1).unwrap();
        let root  = nox_hash(ar, cons);
        (root, hn0, hn1)
    }

    /// Build calling object and run merkle_verify_jet.
    fn run<const N: usize>(
        ar: &mut Reduction<N>,
        root: Order, current: Order, sibling: Order, dir: u64,
    ) -> Outcome {
        let depth_id  = ar.atom(g(1)).unwrap();
        let formula_p = ar.atom(g(0)).unwrap(); // placeholder
        let root_f    = ar.pair(root, formula_p).unwrap();
        let meta      = ar.pair(depth_id, root_f).unwrap();
        let dir_atom  = ar.atom(g(dir)).unwrap();
        let step      = ar.pair(sibling, dir_atom).unwrap();
        let term      = ar.atom(g(0)).unwrap();
        let path      = ar.pair(step, term).unwrap();
        let data      = ar.pair(current, path).unwrap();
        let object    = ar.pair(meta, data).unwrap();
        let dummy_body = ar.atom(g(0)).unwrap();
        let mut row  = TraceRow::default();
        merkle_verify_jet(ar, object, dummy_body, 10_000, &NullCalls, &mut NoTrace, 0, &mut row)
    }

    #[test]
    fn valid_left_leaf_returns_one() {
        let mut ar = Reduction::<512>::new();
        let (root, hn0, hn1) = two_leaf_tree(&mut ar);
        // leaf0 is left child → sibling (hn1) is right → dir=1
        match run(&mut ar, root, hn0, hn1, 1) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn wrong_leaf_returns_zero() {
        let mut ar = Reduction::<512>::new();
        let (root, _hn0, hn1) = two_leaf_tree(&mut ar);
        let wrong_atom = ar.atom(g(999)).unwrap();
        let wrong_hash = nox_hash(&mut ar, wrong_atom);
        match run(&mut ar, root, wrong_hash, hn1, 1) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn valid_right_leaf_returns_one() {
        let mut ar = Reduction::<512>::new();
        let (root, hn0, hn1) = two_leaf_tree(&mut ar);
        // leaf1 is right child → sibling (hn0) is left → dir=0
        match run(&mut ar, root, hn1, hn0, 0) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap(), g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn budget_exhaustion_halts() {
        let mut ar = Reduction::<512>::new();
        let (root, hn0, hn1) = two_leaf_tree(&mut ar);
        // depth=1, cost=25 — pass budget=10 → Halt
        let depth_id  = ar.atom(g(1)).unwrap();
        let formula_p = ar.atom(g(0)).unwrap();
        let root_f    = ar.pair(root, formula_p).unwrap();
        let meta      = ar.pair(depth_id, root_f).unwrap();
        let dir_atom  = ar.atom(g(1)).unwrap();
        let step      = ar.pair(hn1, dir_atom).unwrap();
        let term      = ar.atom(g(0)).unwrap();
        let path      = ar.pair(step, term).unwrap();
        let data      = ar.pair(hn0, path).unwrap();
        let object    = ar.pair(meta, data).unwrap();
        let dummy_body = ar.atom(g(0)).unwrap();
        let mut row  = TraceRow::default();
        match merkle_verify_jet(&mut ar, object, dummy_body, 10, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}

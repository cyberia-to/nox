// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! jet: ntt — Number Theoretic Transform over Goldilocks
//!
//! Cooley-Tukey decimation-in-time NTT. Input size must be 2^n.
//! Computes the forward or inverse NTT in-place.
//!
//! Registry calling convention:
//!   object = [[n | formula] | [values_tree | omega]]
//!   n          = log2 of the input size
//!   formula    = self-reference (jet ignores)
//!   values_tree = balanced binary tree of 2^n field elements (left-to-right order)
//!   omega       = primitive 2^n-th root of unity (for inverse: omega^{-1})
//!
//! Output: balanced binary tree of the same shape, holding the NTT output.
//! Budget: n × 2^n (one unit per butterfly operation).

extern crate alloc;
use alloc::vec::Vec;

use nebu::Goldilocks;
use crate::data::{Reduction, Order, Data};
use crate::reduce::{Outcome, ErrorKind, pair_children};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};

pub fn ntt_jet<const N: usize>(
    reduction: &mut Reduction<N>, object: Order, _body: Order, budget: u64,
    _hints: &dyn CallProvider<N>, _tracer: &mut dyn Tracer, _depth: u64,
    row: &mut TraceRow,
) -> Outcome {
    // object = [[n | formula] | [values_tree | omega]]
    let (lhs, rhs) = match pair_children(reduction, object) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (n_id, _formula_id) = match pair_children(reduction, lhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (tree_id, omega_id) = match pair_children(reduction, rhs) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };

    let n = match reduction.atom_value(n_id) {
        Some(v) => v.as_u64() as usize,
        None => return Outcome::Error(ErrorKind::TypeError),
    };
    let omega = match reduction.atom_value(omega_id) {
        Some(v) => v,
        None => return Outcome::Error(ErrorKind::TypeError),
    };

    let size = 1usize << n;

    // Budget: n × size butterfly operations
    let cost = (n as u64).saturating_mul(size as u64);
    if budget < cost {
        return Outcome::Halt(budget);
    }
    let remaining = budget - cost;

    // Flatten the balanced binary tree into a Vec.
    let mut vals: Vec<Goldilocks> = Vec::with_capacity(size);
    if !flatten_tree(reduction, tree_id, &mut vals) {
        return Outcome::Error(ErrorKind::TypeError);
    }
    if vals.len() != size {
        return Outcome::Error(ErrorKind::TypeError);
    }

    // Cooley-Tukey DIT NTT (bit-reversal permutation first, then butterflies).
    bit_reverse_permute(&mut vals);
    ntt_dit(&mut vals, omega);

    // Write result back as a balanced binary tree into the order.
    let result_tree = match build_tree(reduction, &vals) {
        Some(id) => id,
        None => return Outcome::Error(ErrorKind::Unavailable),
    };

    row.r[4] = tree_id as u64;
    row.r[5] = omega_id as u64;
    row.r[6] = result_tree as u64;

    Outcome::Ok(result_tree, remaining)
}

/// Cooley-Tukey DIT NTT (input must already be in bit-reversed order).
fn ntt_dit(vals: &mut [Goldilocks], omega: Goldilocks) {
    let n = vals.len();
    if n <= 1 { return; }

    let mut len = 2;
    while len <= n {
        // Twiddle step: omega^(n/len) is the primitive len-th root of unity.
        let step = n / len;
        let w_len = pow_field(omega, step as u64);
        let mut j = 0;
        while j < n {
            let mut w = Goldilocks::ONE;
            for k in 0..len / 2 {
                let u = vals[j + k];
                let v = vals[j + k + len / 2] * w;
                vals[j + k]           = u + v;
                vals[j + k + len / 2] = u - v;
                w *= w_len;
            }
            j += len;
        }
        len *= 2;
    }
}

fn pow_field(mut base: Goldilocks, mut exp: u64) -> Goldilocks {
    let mut result = Goldilocks::ONE;
    while exp > 0 {
        if exp & 1 == 1 { result *= base; }
        base = base * base;
        exp >>= 1;
    }
    result
}

fn bit_reverse_permute(vals: &mut [Goldilocks]) {
    let n = vals.len();
    let bits = n.trailing_zeros() as usize;
    for i in 0..n {
        let j = bit_reverse(i, bits);
        if i < j { vals.swap(i, j); }
    }
}

fn bit_reverse(mut x: usize, bits: usize) -> usize {
    let mut result = 0;
    for _ in 0..bits {
        result = (result << 1) | (x & 1);
        x >>= 1;
    }
    result
}

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

fn build_tree<const N: usize>(reduction: &mut Reduction<N>, vals: &[Goldilocks]) -> Option<Order> {
    if vals.len() == 1 {
        return reduction.atom(vals[0]);
    }
    let mid = vals.len() / 2;
    let left  = build_tree(reduction, &vals[..mid])?;
    let right = build_tree(reduction, &vals[mid..])?;
    reduction.pair(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduce::Outcome;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, TraceRow};
    use crate::data::{Reduction};

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    fn make_tree<const N: usize>(ar: &mut Reduction<N>, vals: &[Goldilocks]) -> Order {
        build_tree(ar, vals).unwrap()
    }

    fn run_ntt<const M: usize>(
        ar: &mut Reduction<M>, vals: &[Goldilocks], omega: Goldilocks,
    ) -> Outcome {
        let n_val = vals.len().trailing_zeros() as u64;
        let n_id  = ar.atom(g(n_val)).unwrap();
        let dummy = ar.atom(g(0)).unwrap();
        let lhs   = ar.pair(n_id, dummy).unwrap();
        let tree  = make_tree(ar, vals);
        let om_id = ar.atom(omega).unwrap();
        let rhs   = ar.pair(tree, om_id).unwrap();
        let obj   = ar.pair(lhs, rhs).unwrap();
        let body  = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        ntt_jet(ar, obj, body, 1_000_000, &NullCalls, &mut NoTrace, 0, &mut row)
    }

    #[test]
    fn ntt_size_one_is_identity() {
        let mut ar = Reduction::<256>::new();
        match run_ntt(&mut ar, &[g(42)], g(1)) {
            Outcome::Ok(r, _) => {
                // Single element: the tree is the element itself.
                assert_eq!(ar.atom_value(r).unwrap(), g(42));
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn ntt_size_two_butterfly() {
        // 2-point NTT with omega=1: [a,b] → [a+b, a-b] (no twiddle)
        let mut ar = Reduction::<256>::new();
        let a = g(3);
        let b = g(5);
        let omega = g(1); // omega^1 = 1
        match run_ntt(&mut ar, &[a, b], omega) {
            Outcome::Ok(result_tree, _) => {
                // Extract left and right from the result tree.
                match ar.get(result_tree).map(|e| e.inner) {
                    Some(Data::Pair { left, right }) => {
                        let top = ar.atom_value(left).unwrap();
                        let bot = ar.atom_value(right).unwrap();
                        // a+b and a-b in the Goldilocks field
                        assert_eq!(top, a + b);
                        assert_eq!(bot, a - b);
                    }
                    _ => panic!("expected pair result"),
                }
            }
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn budget_exhaustion_halts() {
        let mut ar = Reduction::<256>::new();
        // n=2, size=4, cost=2*4=8; budget=5 → Halt
        match run_ntt(&mut ar, &[g(1), g(2), g(3), g(4)], g(1)) {
            // budget=1_000_000 → should be Ok (sufficient)
            Outcome::Ok(_, _) => {}
            o => panic!("{:?}", o),
        }
        // Now test with insufficient budget
        let n_id  = ar.atom(g(2)).unwrap();
        let dummy = ar.atom(g(0)).unwrap();
        let lhs   = ar.pair(n_id, dummy).unwrap();
        let tree  = make_tree(&mut ar, &[g(1), g(2), g(3), g(4)]);
        let om    = ar.atom(g(1)).unwrap();
        let rhs   = ar.pair(tree, om).unwrap();
        let obj   = ar.pair(lhs, rhs).unwrap();
        let body  = ar.atom(g(0)).unwrap();
        let mut row = TraceRow::default();
        match ntt_jet(&mut ar, obj, body, 5, &NullCalls, &mut NoTrace, 0, &mut row) {
            Outcome::Halt(_) => {}
            o => panic!("expected Halt, got {:?}", o),
        }
    }
}

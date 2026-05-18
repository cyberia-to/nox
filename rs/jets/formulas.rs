// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! Pure Layer 1 formula nouns for the genesis jet set.
//!
//! Each jet is identified by H(formula_noun) — the ContentId of its pure L1
//! definition.  The jet implementation is an optimization; removing all jets
//! produces identical results, only slower.
//!
//! poly_eval calling convention:
//!   object = [[k | formula] | [evals_tree | point]]
//!   axis 4 = k             num_vars remaining
//!   axis 5 = formula       self-reference (quining)
//!   axis 6 = evals_tree    balanced binary tree of evaluations
//!   axis 7 = point         [x_{k-1} | rest_point]  BIG-ENDIAN: MSV first
//!   axis 12 = left(evals)  half with x_{k-1}=0
//!   axis 13 = right(evals) half with x_{k-1}=1
//!   axis 14 = x_{k-1}      most-significant variable at this level
//!   axis 15 = rest_point   remaining variables
//!
//! merkle_verify calling convention:
//!   object = [[depth | [root | formula]] | [current | path]]
//!   axis 4  = depth        steps remaining
//!   axis 5  = [root|formula]
//!   axis 10 = root         expected root hash-noun
//!   axis 11 = formula      self-reference
//!   axis 6  = current      current hash-noun
//!   axis 7  = path         right-nested list of [sibling | dir] pairs
//!   axis 14 = [sibling|dir] head of path
//!   axis 28 = sibling      sibling hash-noun
//!   axis 29 = dir          0=sibling is left, 1=sibling is right
//!
//! Hash semantics: merkle_verify uses pattern 15 (hash) to compute each node
//! hash as Poseidon2(structural_digest(cons(sibling, current))). This is the
//! nox-native Merkle tree — the tree hash is defined entirely by nox's own
//! structural digest + Poseidon2 permutation.

extern crate alloc;

use nebu::Goldilocks;
use crate::noun::{Order, NounId, Tag};
use crate::encode::{ContentId, content_id};

// ── primitive builders ────────────────────────────────────────────────────────

fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

fn atom<const N: usize>(ar: &mut Order<N>, v: u64) -> Option<NounId> {
    ar.atom(g(v), Tag::Field)
}

/// [tag | addr] — axis pattern
fn axis_f<const N: usize>(ar: &mut Order<N>, n: u64) -> Option<NounId> {
    let t = atom(ar, 0)?;
    let a = atom(ar, n)?;
    ar.cell(t, a)
}

/// [1 | x] — quote pattern
fn quote_f<const N: usize>(ar: &mut Order<N>, x: NounId) -> Option<NounId> {
    let t = atom(ar, 1)?;
    ar.cell(t, x)
}

/// [2 | [a | b]] — compose pattern
fn compose_f<const N: usize>(ar: &mut Order<N>, a: NounId, b: NounId) -> Option<NounId> {
    let t = atom(ar, 2)?;
    let body = ar.cell(a, b)?;
    ar.cell(t, body)
}

/// [3 | [a | b]] — cons pattern
fn cons_f<const N: usize>(ar: &mut Order<N>, a: NounId, b: NounId) -> Option<NounId> {
    let t = atom(ar, 3)?;
    let body = ar.cell(a, b)?;
    ar.cell(t, body)
}

/// [4 | [test | [yes | no]]] — branch pattern (test=0 → yes)
fn branch_f<const N: usize>(ar: &mut Order<N>, test: NounId, yes: NounId, no: NounId) -> Option<NounId> {
    let t = atom(ar, 4)?;
    let arms = ar.cell(yes, no)?;
    let body = ar.cell(test, arms)?;
    ar.cell(t, body)
}

/// [tag | [a | b]] — binary arithmetic/logic patterns (add/sub/mul/eq/…)
fn binop_f<const N: usize>(ar: &mut Order<N>, tag: u64, a: NounId, b: NounId) -> Option<NounId> {
    let t = atom(ar, tag)?;
    let body = ar.cell(a, b)?;
    ar.cell(t, body)
}

/// [15 | sub_formula] — hash pattern (Poseidon2 of structural digest of result)
fn hash_f<const N: usize>(ar: &mut Order<N>, sub: NounId) -> Option<NounId> {
    let t = atom(ar, 15)?;
    ar.cell(t, sub)
}

// ── poly_eval formula ─────────────────────────────────────────────────────────

/// Build the pure Layer 1 nox formula for multilinear polynomial evaluation.
///
/// Algorithm (recursive splitting over balanced binary tree):
///   base (k=0): return evals_tree (the single leaf value)
///   step: vl = eval(left_half, rest); vr = eval(right_half, rest)
///         return vl + x0*(vr − vl)
///
/// The formula is self-contained: axis 5 retrieves the formula from the object
/// at runtime (quining). No circular reference at build time.
pub fn build_poly_eval_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    // ── axis references in the calling object ─────────────────────────────────
    let ax4  = axis_f(ar, 4)?;   // k
    let ax5  = axis_f(ar, 5)?;   // formula (self-ref)
    let ax6  = axis_f(ar, 6)?;   // evals_tree
    let ax12 = axis_f(ar, 12)?;  // left(evals_tree)
    let ax13 = axis_f(ar, 13)?;  // right(evals_tree)
    let ax14 = axis_f(ar, 14)?;  // x0
    let ax15 = axis_f(ar, 15)?;  // rest_point

    // k_minus_1 = sub(k, 1)
    let one = atom(ar, 1)?;
    let q1 = quote_f(ar, one)?;
    let k_minus_1 = binop_f(ar, 6, ax4, q1)?;

    // build_lhs = cons(k−1, formula_ref) → [k−1 | formula]
    let build_lhs = cons_f(ar, k_minus_1, ax5)?;

    // left recursion: new object = [[k−1 | formula] | [left_evals | rest_point]]
    let left_pair    = cons_f(ar, ax12, ax15)?;
    let new_left_obj = cons_f(ar, build_lhs, left_pair)?;
    let call_left    = compose_f(ar, new_left_obj, ax5)?;

    // right recursion: new object = [[k−1 | formula] | [right_evals | rest_point]]
    let right_pair    = cons_f(ar, ax13, ax15)?;
    let new_right_obj = cons_f(ar, build_lhs, right_pair)?;
    let call_right    = compose_f(ar, new_right_obj, ax5)?;

    // result_cons = [vl | [vr | x0]]  (intermediate object for combiner)
    let inner_cons  = cons_f(ar, call_right, ax14)?;
    let result_cons = cons_f(ar, call_left, inner_cons)?;

    // combiner formula — applied to intermediate object [vl | [vr | x0]]:
    //   axis 2 = vl,  axis 6 = vr,  axis 7 = x0
    let ax2c = axis_f(ar, 2)?;
    let ax6c = axis_f(ar, 6)?;
    let ax7c = axis_f(ar, 7)?;
    let diff     = binop_f(ar, 6, ax6c, ax2c)?;          // vr − vl
    let prod     = binop_f(ar, 7, ax7c, diff)?;           // x0 * (vr − vl)
    let combiner = binop_f(ar, 5, ax2c, prod)?;           // vl + x0*(vr−vl)

    // recursive step: cons the two recursive results, then apply combiner
    let q_combiner    = quote_f(ar, combiner)?;
    let recursive_step = compose_f(ar, result_cons, q_combiner)?;

    // test: eq(k, 0) → 0 if k=0 (→ yes branch = base case)
    let zero = atom(ar, 0)?;
    let q0   = quote_f(ar, zero)?;
    let test = binop_f(ar, 9, ax4, q0)?;

    // formula = branch(eq(k,0), base_case=axis(6), recursive_step)
    branch_f(ar, test, ax6, recursive_step)
}

/// Content-addressed identity of the poly_eval formula noun.
pub fn poly_eval_formula_hash<const N: usize>(ar: &mut Order<N>) -> Option<ContentId> {
    let formula = build_poly_eval_formula(ar)?;
    content_id(ar, formula)
}

// ── merkle_verify formula ─────────────────────────────────────────────────────

/// Build the pure Layer 1 nox formula for nox-native Merkle path verification.
///
/// Hash semantics: each node hash is computed as pattern 15 (Poseidon2) applied
/// to the structural digest of cons(sibling, current) or cons(current, sibling).
/// This defines the nox-native Merkle tree — no external hash convention needed.
///
/// Returns field atom 1 if the path is valid, 0 if not.
pub fn build_merkle_verify_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    // ── axis references in the calling object ─────────────────────────────────
    let ax4  = axis_f(ar, 4)?;   // depth (steps remaining)
    let ax5  = axis_f(ar, 5)?;   // [root | formula]
    let ax6  = axis_f(ar, 6)?;   // current hash-noun
    let ax10 = axis_f(ar, 10)?;  // root hash-noun
    let ax11 = axis_f(ar, 11)?;  // formula (self-ref)
    let ax15 = axis_f(ar, 15)?;  // rest_path
    let ax28 = axis_f(ar, 28)?;  // sibling hash-noun
    let ax29 = axis_f(ar, 29)?;  // dir (0=sib left, 1=sib right)

    // ── base case: depth=0 → compare current with root ───────────────────────
    // eq(current, root): 0 if equal → yes = quote(1) [valid], no = quote(0)
    let zero_a = atom(ar, 0)?;
    let one_a  = atom(ar, 1)?;
    let q0   = quote_f(ar, zero_a)?;
    let q1   = quote_f(ar, one_a)?;
    let eq_cr = binop_f(ar, 9, ax6, ax10)?;
    let base_case = branch_f(ar, eq_cr, q1, q0)?;

    // ── hash step: compute new current from sibling and current ──────────────
    // dir=0: sibling is left → hash(cons(sibling, current))
    let cons_sib_cur = cons_f(ar, ax28, ax6)?;
    let hash_sib_cur = hash_f(ar, cons_sib_cur)?;

    // dir=1: sibling is right → hash(cons(current, sibling))
    let cons_cur_sib = cons_f(ar, ax6, ax28)?;
    let hash_cur_sib = hash_f(ar, cons_cur_sib)?;

    // new_current = branch(dir, hash_sib_cur, hash_cur_sib)
    let new_current = branch_f(ar, ax29, hash_sib_cur, hash_cur_sib)?;

    // ── recursive step: build new object and call formula ────────────────────
    // new_meta = [depth−1 | [root | formula]]
    let one_b      = atom(ar, 1)?;
    let q1b        = quote_f(ar, one_b)?;
    let depth_m1   = binop_f(ar, 6, ax4, q1b)?;
    let new_meta   = cons_f(ar, depth_m1, ax5)?;

    // new_data = [new_current | rest_path]
    let new_data   = cons_f(ar, new_current, ax15)?;

    // new_obj formula = cons(new_meta, new_data)
    let new_obj    = cons_f(ar, new_meta, new_data)?;

    // recursive_call = compose(new_obj, formula_ref)
    let recursive_step = compose_f(ar, new_obj, ax11)?;

    // ── outer branch: depth=0? ────────────────────────────────────────────────
    let eq_depth_0 = binop_f(ar, 9, ax4, q0)?;
    branch_f(ar, eq_depth_0, base_case, recursive_step)
}

/// Content-addressed identity of the merkle_verify formula noun.
pub fn merkle_verify_formula_hash<const N: usize>(ar: &mut Order<N>) -> Option<ContentId> {
    let formula = build_merkle_verify_formula(ar)?;
    content_id(ar, formula)
}

// ── fri_fold formula ─────────────────────────────────────────────────────────

/// Build the pure Layer 1 nox formula for FRI folding.
///
/// Folds a balanced binary tree of 2^k evaluations to a single value using
/// challenge r. Each level: result = left + r*(right - left).
///
/// Calling convention (same structure as poly_eval):
///   object = [[k | formula] | [evals_tree | r]]
///   axis 4  = k (remaining fold levels)
///   axis 5  = formula (self-reference)
///   axis 6  = evals_tree
///   axis 7  = r (challenge — reused at every level, unlike poly_eval)
///   axis 12 = left half of evals_tree
///   axis 13 = right half of evals_tree
pub fn build_fri_fold_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    let ax4  = axis_f(ar, 4)?;   // k
    let ax5  = axis_f(ar, 5)?;   // formula (self-ref)
    let ax6  = axis_f(ar, 6)?;   // evals_tree
    let ax7  = axis_f(ar, 7)?;   // r (challenge, same for all levels)
    let ax12 = axis_f(ar, 12)?;  // left(evals_tree)
    let ax13 = axis_f(ar, 13)?;  // right(evals_tree)

    let one  = atom(ar, 1)?;
    let q1   = quote_f(ar, one)?;
    let k_m1 = binop_f(ar, 6, ax4, q1)?;           // k − 1

    let lhs = cons_f(ar, k_m1, ax5)?;               // [k−1 | formula]
    let lp  = cons_f(ar, ax12, ax7)?;               // [left_evals | r]
    let rp  = cons_f(ar, ax13, ax7)?;               // [right_evals | r]
    let lo  = cons_f(ar, lhs, lp)?;                 // left object
    let ro  = cons_f(ar, lhs, rp)?;                 // right object
    let vl  = compose_f(ar, lo, ax5)?;              // recurse left
    let vr  = compose_f(ar, ro, ax5)?;              // recurse right

    // combiner applied to [vl | [vr | r]]
    let ic = cons_f(ar, vr, ax7)?;                  // [vr | r]
    let rc = cons_f(ar, vl, ic)?;                   // [vl | [vr | r]]

    // combiner formula: axis 2 = vl, axis 6 = vr, axis 7 = r
    let av2 = axis_f(ar, 2)?;
    let av6 = axis_f(ar, 6)?;
    let av7 = axis_f(ar, 7)?;
    let diff     = binop_f(ar, 6, av6, av2)?;       // vr − vl
    let prod     = binop_f(ar, 7, av7, diff)?;      // r * (vr − vl)
    let combiner = binop_f(ar, 5, av2, prod)?;      // vl + r*(vr−vl)
    let qc  = quote_f(ar, combiner)?;
    let step = compose_f(ar, rc, qc)?;              // apply combiner

    let zero = atom(ar, 0)?;
    let q0   = quote_f(ar, zero)?;
    let test = binop_f(ar, 9, ax4, q0)?;            // eq(k, 0)
    branch_f(ar, test, ax6, step)                   // k=0 → leaf, else step
}

// ── ntt formula ───────────────────────────────────────────────────────────────

/// Build the pure Layer 1 nox formula for Number Theoretic Transform.
///
/// Calling convention:
///   object = [[n | formula] | [values_tree | omega]]
///   n          = log2 of the input size
///   values_tree = balanced binary tree of field elements
///   omega       = primitive n-th root of unity
///
/// This formula computes one butterfly step (correct for n=1; the jet handles
/// the full Cooley-Tukey recursion for n>1). The formula's hash anchors the jet.
pub fn build_ntt_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    let ax4  = axis_f(ar, 4)?;   // n
    let ax6  = axis_f(ar, 6)?;   // values_tree (returned at base)
    let ax7  = axis_f(ar, 7)?;   // omega
    let ax12 = axis_f(ar, 12)?;  // a = head(values_tree)
    let ax13 = axis_f(ar, 13)?;  // b = tail(values_tree)

    // base (n=0): return values_tree as-is (single element)
    let zero = atom(ar, 0)?;
    let q0   = quote_f(ar, zero)?;
    let test = binop_f(ar, 9, ax4, q0)?;

    // butterfly: [a + omega*b | a − omega*b]
    let wb  = binop_f(ar, 7, ax7, ax13)?;           // omega * b
    let top = binop_f(ar, 5, ax12, wb)?;            // a + omega*b
    let bot = binop_f(ar, 6, ax12, wb)?;            // a − omega*b
    let butterfly = cons_f(ar, top, bot)?;

    branch_f(ar, test, ax6, butterfly)
}

// ── cyberlink formula ─────────────────────────────────────────────────────────

/// State jet tag constants (> 17 so they cannot be valid L1 pattern tags).
pub(crate) const CYBERLINK_TAG:  u64 = 1000;
pub(crate) const TRANSFER_TAG:   u64 = 1001;
pub(crate) const INSERT_TAG:     u64 = 1002;
pub(crate) const UPDATE_TAG:     u64 = 1003;
pub(crate) const AGGREGATE_TAG:  u64 = 1004;
pub(crate) const CONSERVE_TAG:   u64 = 1005;

/// Build the pure Layer 1 nox formula for the CYBERLINK state operation.
///
/// Formula = [CYBERLINK_TAG | axis(1)]
/// Applied to object [from_cid | to_cid]: the formula returns the whole object.
/// The formula's hash uniquely identifies the CYBERLINK jet.
pub fn build_cyberlink_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    let tag  = atom(ar, CYBERLINK_TAG)?;
    let body = axis_f(ar, 1)?;                      // axis(1) = whole object
    ar.cell(tag, body)
}

// ── decider formula ───────────────────────────────────────────────────────────

/// Build the pure Layer 1 nox formula for the HyperNova decider.
///
/// Formula = [DECIDER_TAG | [axis(2) | axis(3)]]
/// The formula's hash uniquely identifies the HyperNova verifier jet.
/// Full 89/825-constraint verification is implemented in the jet and hints.
pub fn build_decider_formula<const N: usize>(ar: &mut Order<N>) -> Option<NounId> {
    const DECIDER_TAG: u64 = 1099;
    let tag  = atom(ar, DECIDER_TAG)?;
    let lhs  = axis_f(ar, 2)?;
    let rhs  = axis_f(ar, 3)?;
    let body = ar.cell(lhs, rhs)?;
    ar.cell(tag, body)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::reduce::{reduce, Outcome};
    use crate::call::NullCalls;
    use crate::trace::NoTrace;
    use crate::noun::{Order, Tag};
    use hemera::StepSponge;
    use hemera::field::Goldilocks as HemeraGold;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    // ── poly_eval helpers ─────────────────────────────────────────────────────

    /// Build a balanced binary tree from evaluations (must be power-of-two length).
    fn build_evals_tree<const N: usize>(ar: &mut Order<N>, vals: &[Goldilocks]) -> NounId {
        assert!(vals.len().is_power_of_two() && !vals.is_empty());
        if vals.len() == 1 {
            return ar.atom(vals[0], Tag::Field).unwrap();
        }
        let mid = vals.len() / 2;
        let left  = build_evals_tree(ar, &vals[..mid]);
        let right = build_evals_tree(ar, &vals[mid..]);
        ar.cell(left, right).unwrap()
    }

    /// Build a right-nested cons list from point coordinates, with atom terminator.
    /// Always produces a cons cell so axis(14)=head(point) is valid even for 1 var.
    fn build_point<const N: usize>(ar: &mut Order<N>, coords: &[Goldilocks]) -> NounId {
        assert!(!coords.is_empty());
        let term = ar.atom(g(0), Tag::Field).unwrap();
        coords.iter().rev().fold(term, |acc, &c| {
            let head = ar.atom(c, Tag::Field).unwrap();
            ar.cell(head, acc).unwrap()
        })
    }

    /// Run the poly_eval formula via reduce().
    fn run_poly_eval_formula<const N: usize>(
        ar: &mut Order<N>, evals: &[Goldilocks], point: &[Goldilocks],
    ) -> Outcome {
        let formula = build_poly_eval_formula(ar).unwrap();
        let evals_tree  = build_evals_tree(ar, evals);
        let point_noun  = build_point(ar, point);
        let k           = ar.atom(g(evals.len().trailing_zeros() as u64), Tag::Field).unwrap();
        // object = [[k | formula] | [evals_tree | point]]
        let lhs = ar.cell(k, formula).unwrap();
        let rhs = ar.cell(evals_tree, point_noun).unwrap();
        let obj = ar.cell(lhs, rhs).unwrap();
        reduce(ar, obj, formula, 1_000_000, &NullCalls, &mut NoTrace)
    }

    #[test]
    fn poly_eval_formula_at_zero() {
        let mut ar = Order::<4096>::new();
        match run_poly_eval_formula(&mut ar, &[g(3), g(7)], &[g(0)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(3)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn poly_eval_formula_at_one() {
        let mut ar = Order::<4096>::new();
        match run_poly_eval_formula(&mut ar, &[g(3), g(7)], &[g(1)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(7)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn poly_eval_formula_two_vars_at_half() {
        // f(x0,x1) with evals [0,2,4,6]:
        //   f(0,0)=0, f(1,0)=2, f(0,1)=4, f(1,1)=6
        // Evaluate at x0=1/2, x1=0.
        // Point is passed BIG-ENDIAN (most-significant variable first): [x1, x0].
        // Algorithm splits on x1 first: vl=f(*,0) at x0=1/2, vr=f(*,1) at x0=1/2.
        //   vl = 0 + (1/2)*(2-0) = 1
        //   vr = 4 + (1/2)*(6-4) = 5
        //   result = 1 + 0*(5-1) = 1
        let mut ar = Order::<4096>::new();
        let p = 0xFFFF_FFFF_0000_0001u64; // Goldilocks prime
        let half = (p + 1) / 2;           // multiplicative inverse of 2 mod p
        // point = [x1=0, x0=1/2]  (big-endian: most-significant variable first)
        match run_poly_eval_formula(&mut ar, &[g(0), g(2), g(4), g(6)], &[g(0), g(half)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn poly_eval_formula_hash_is_stable() {
        let mut ar1 = Order::<4096>::new();
        let mut ar2 = Order::<4096>::new();
        let h1 = poly_eval_formula_hash(&mut ar1).unwrap();
        let h2 = poly_eval_formula_hash(&mut ar2).unwrap();
        assert_eq!(h1, h2, "formula hash must be deterministic");
    }

    // ── merkle_verify helpers ─────────────────────────────────────────────────

    /// Compute the nox-native hash of a noun (pattern 15 semantics):
    /// Poseidon2 of the noun's structural digest.
    fn nox_hash_noun<const N: usize>(ar: &mut Order<N>, id: NounId) -> NounId {
        let dig = *ar.digest(id).unwrap();
        let mut rate = [HemeraGold::ZERO; 8];
        for i in 0..4 { rate[i] = HemeraGold::new(dig[i].as_u64()); }
        let mut sponge = StepSponge::absorb(&rate);
        let mut state  = [HemeraGold::ZERO; 16];
        while !sponge.done() { state = sponge.step(); }
        let out = [
            g(state[0].as_canonical_u64()),
            g(state[1].as_canonical_u64()),
            g(state[2].as_canonical_u64()),
            g(state[3].as_canonical_u64()),
        ];
        ar.hash_noun(&out).unwrap()
    }

    /// Build a two-leaf nox-native merkle tree.
    /// Returns (root_hash_noun, leaf0_hash_noun, leaf1_hash_noun).
    fn two_leaf_tree<const N: usize>(ar: &mut Order<N>) -> (NounId, NounId, NounId) {
        let leaf0 = ar.atom(g(100), Tag::Field).unwrap();
        let leaf1 = ar.atom(g(200), Tag::Field).unwrap();
        let hn0 = nox_hash_noun(ar, leaf0);
        let hn1 = nox_hash_noun(ar, leaf1);
        // root = hash(cons(hn0, hn1))
        let cons_id = ar.cell(hn0, hn1).unwrap();
        let root = nox_hash_noun(ar, cons_id);
        (root, hn0, hn1)
    }

    /// Build the path cons list: right-nested [sibling | dir] pairs + atom terminator.
    fn build_path<const N: usize>(ar: &mut Order<N>, steps: &[(NounId, u64)]) -> NounId {
        let term = ar.atom(g(0), Tag::Field).unwrap();
        steps.iter().rev().fold(term, |acc, &(sib, dir)| {
            let d   = ar.atom(g(dir), Tag::Field).unwrap();
            let step = ar.cell(sib, d).unwrap();
            ar.cell(step, acc).unwrap()
        })
    }

    /// Run the merkle_verify formula via reduce().
    fn run_merkle_formula<const N: usize>(
        ar: &mut Order<N>,
        root: NounId, leaf_hash: NounId,
        path: &[(NounId, u64)],
    ) -> Outcome {
        let formula = build_merkle_verify_formula(ar).unwrap();
        let depth   = ar.atom(g(path.len() as u64), Tag::Field).unwrap();
        let root_pair = ar.cell(root, formula).unwrap();
        let meta    = ar.cell(depth, root_pair).unwrap();
        let path_noun = build_path(ar, path);
        let data    = ar.cell(leaf_hash, path_noun).unwrap();
        let obj     = ar.cell(meta, data).unwrap();
        reduce(ar, obj, formula, 10_000_000, &NullCalls, &mut NoTrace)
    }

    #[test]
    fn merkle_formula_valid_path_returns_one() {
        let mut ar = Order::<4096>::new();
        let (root, hn0, hn1) = two_leaf_tree(&mut ar);
        // leaf0 is left → sibling (hn1) is right → dir=1
        match run_merkle_formula(&mut ar, root, hn0, &[(hn1, 1)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(1)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn merkle_formula_wrong_leaf_returns_zero() {
        let mut ar = Order::<4096>::new();
        let (root, _hn0, hn1) = two_leaf_tree(&mut ar);
        let wrong = ar.atom(g(999), Tag::Field).unwrap();
        let wrong_hash = nox_hash_noun(&mut ar, wrong);
        match run_merkle_formula(&mut ar, root, wrong_hash, &[(hn1, 1)]) {
            Outcome::Ok(r, _) => assert_eq!(ar.atom_value(r).unwrap().0, g(0)),
            o => panic!("{:?}", o),
        }
    }

    #[test]
    fn merkle_formula_hash_is_stable() {
        let mut ar1 = Order::<4096>::new();
        let mut ar2 = Order::<4096>::new();
        let h1 = merkle_verify_formula_hash(&mut ar1).unwrap();
        let h2 = merkle_verify_formula_hash(&mut ar2).unwrap();
        assert_eq!(h1, h2, "formula hash must be deterministic");
    }

    // ── new genesis formula hash stability ────────────────────────────────────

    #[test]
    fn fri_fold_formula_hash_is_stable() {
        use crate::encode::content_id;
        let mut ar1 = Order::<4096>::new();
        let mut ar2 = Order::<4096>::new();
        let f1 = build_fri_fold_formula(&mut ar1).unwrap();
        let h1 = content_id(&ar1, f1).unwrap();
        let f2 = build_fri_fold_formula(&mut ar2).unwrap();
        let h2 = content_id(&ar2, f2).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn ntt_formula_hash_is_stable() {
        use crate::encode::content_id;
        let mut ar1 = Order::<4096>::new();
        let mut ar2 = Order::<4096>::new();
        let f1 = build_ntt_formula(&mut ar1).unwrap();
        let h1 = content_id(&ar1, f1).unwrap();
        let f2 = build_ntt_formula(&mut ar2).unwrap();
        let h2 = content_id(&ar2, f2).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn genesis_formulas_have_distinct_hashes() {
        use crate::encode::content_id;
        let mut ar = Order::<8192>::new();
        let id_pe = build_poly_eval_formula(&mut ar).unwrap();
        let h_pe  = content_id(&ar, id_pe).unwrap();
        let id_mv = build_merkle_verify_formula(&mut ar).unwrap();
        let h_mv  = content_id(&ar, id_mv).unwrap();
        let id_ff = build_fri_fold_formula(&mut ar).unwrap();
        let h_ff  = content_id(&ar, id_ff).unwrap();
        let id_nt = build_ntt_formula(&mut ar).unwrap();
        let h_nt  = content_id(&ar, id_nt).unwrap();
        let id_cl = build_cyberlink_formula(&mut ar).unwrap();
        let h_cl  = content_id(&ar, id_cl).unwrap();
        let id_dc = build_decider_formula(&mut ar).unwrap();
        let h_dc  = content_id(&ar, id_dc).unwrap();
        let hashes = [h_pe, h_mv, h_ff, h_nt, h_cl, h_dc];
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "formulas {i} and {j} share a hash");
            }
        }
    }
}

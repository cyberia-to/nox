// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! reduce — dispatch + budget metering + shared helpers
//! pattern implementations live in patterns/
//! every reduce() call emits one TraceRow via the Tracer

use nebu::Goldilocks;
use crate::noun::{Order, NounId, Noun, Tag, NIL};
use crate::call::CallProvider;
use crate::trace::{Tracer, TraceRow};
use crate::patterns;
use crate::jets::registry::{JetRegistry, digest_key};

// ── pattern tags + per-pattern costs ──────────────────────────
//
// Indexed by tag; row count equals cost for multi-row patterns so zheng
// constrains one soundness witness slot per budget unit. Keep this table
// in sync with specs/trace.md and specs/patterns/README.md cost column.

const COSTS: [u64; 18] = [
    1,   // 0  axis
    1,   // 1  quote
    1,   // 2  compose
    1,   // 3  cons
    1,   // 4  branch
    1,   // 5  add
    1,   // 6  sub
    1,   // 7  mul
    64,  // 8  inv   — Fermat exponent, one row per bit of p-2
    1,   // 9  eq
    64,  // 10 lt    — bit decomp of two 64-bit canonical reprs
    32,  // 11 xor   — bit decomp of 32-bit word
    32,  // 12 and
    32,  // 13 not
    32,  // 14 shl
    25,  // 15 hash  — 24 Poseidon2 rounds + 1 squeeze row
    1,   // 16 call
    1,   // 17 look
];

// Tags that emit their own rows (cost > 1 by construction).
const TAG_INV:  u64 = 8;
const TAG_LT:   u64 = 10;
const TAG_XOR:  u64 = 11;
const TAG_AND:  u64 = 12;
const TAG_NOT:  u64 = 13;
const TAG_SHL:  u64 = 14;
const TAG_HASH: u64 = 15;

// ── word arithmetic ───────────────────────────────────────────
pub(crate) const WORD_MASK: u64 = 0xFFFF_FFFF;

// ── safety ────────────────────────────────────────────────────
//
// MAX_DEPTH bounds the recursion of reduce_inner. Each frame holds a
// TraceRow (128 bytes) plus locals (~few hundred bytes). With macOS's
// default 8 MiB main-thread stack and the CLI's 32 MiB worker thread,
// 1000 frames is well within budget on every supported target. The
// cap exists so malformed deeply-recursive formulas terminate predictably
// instead of stack-overflowing.
const MAX_DEPTH: u64 = 1000;

#[derive(Debug)]
pub enum Outcome {
    Ok(NounId, u64),
    Halt(u64),
    Error(ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError    = 0,
    AxisError    = 1,
    InvZero      = 2,
    Unavailable  = 3,
    Malformed    = 4,
    CallRejected = 5,
}

fn cost(tag: u64) -> u64 {
    // unknown tags get COST_DEFAULT; the dispatch later returns Malformed
    if (tag as usize) < COSTS.len() { COSTS[tag as usize] } else { 1 }
}

/// Public entry point — depth starts at 0, no jet registry (pure L1).
/// Existing callers and all tests use this. Jets never fire on this path.
pub fn reduce<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T,
) -> Outcome {
    reduce_inner(order, object, formula, budget, hints, tracer, 0, &JetRegistry::empty())
}

/// Entry point with jet registry. Use this in production to enable jet dispatch.
pub fn reduce_with_registry<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T,
    registry: &JetRegistry<N>,
) -> Outcome {
    reduce_inner(order, object, formula, budget, hints, tracer, 0, registry)
}

pub(crate) fn reduce_inner<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> Outcome {
    if depth > MAX_DEPTH {
        emit_error_row(tracer, object, formula, 0, budget, ErrorKind::Malformed);
        return Outcome::Error(ErrorKind::Malformed);
    }

    let (tag_ref, body) = match order.get(formula) {
        Some(e) => match e.inner {
            Noun::Cell { left, right } => (left, right),
            Noun::Atom { .. } => {
                emit_error_row(tracer, object, formula, 0, budget, ErrorKind::Malformed);
                return Outcome::Error(ErrorKind::Malformed);
            }
        },
        None => {
            emit_error_row(tracer, object, formula, 0, budget, ErrorKind::Malformed);
            return Outcome::Error(ErrorKind::Malformed);
        }
    };
    let tag = match order.atom_value(tag_ref) {
        Some((v, _)) => v.as_u64(),
        None => {
            emit_error_row(tracer, object, formula, 0, budget, ErrorKind::Malformed);
            return Outcome::Error(ErrorKind::Malformed);
        }
    };

    let budget_in = budget;
    let mut row = TraceRow::default();
    row.r[0] = tag;
    row.r[1] = object as u64;
    row.r[2] = formula as u64;
    row.r[8] = budget_in;

    // ── jet registry check ───────────────────────────────────────────────────
    // Fires before tag cost and before tag dispatch. Jets own all budget.
    if let Some(fkey) = order.digest(formula).map(digest_key) {
        let jet = registry.lookup_exact(&fkey)
            .or_else(|| registry.lookup_template(order, formula));
        if let Some(jet_fn) = jet {
            let outcome = jet_fn(order, object, body, budget_in, hints,
                tracer as &mut dyn Tracer, depth, &mut row);
            row.r[3] = match &outcome { Outcome::Ok(r, _) => *r as u64, _ => NIL as u64 };
            row.r[9] = match &outcome {
                Outcome::Ok(_, b) | Outcome::Halt(b) => *b,
                Outcome::Error(_) => 0,
            };
            row.r[10] = match &outcome { Outcome::Error(k) => *k as u64, _ => 0 };
            tracer.record(row);
            return outcome;
        }
    }

    // ── normal tag-based dispatch ────────────────────────────────────────────
    let c = cost(tag);
    if budget_in < c {
        emit_halt_row(tracer, object, formula, tag, budget_in);
        return Outcome::Halt(budget_in);
    }
    let budget = budget_in - c;

    // multi-row patterns emit their own rows. these patterns expose per-bit
    // (or per-step) soundness witnesses that don't fit a single 16-col row.
    let is_multi_row = matches!(
        tag,
        TAG_INV | TAG_LT | TAG_XOR | TAG_AND | TAG_NOT | TAG_SHL | TAG_HASH
    );

    let outcome = match tag {
        0  => patterns::axis::axis(order, object, body, budget, hints, &mut row),
        1  => patterns::quote::quote(body, budget, &mut row),
        2  => patterns::compose::compose(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        3  => patterns::cons::cons(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        4  => patterns::branch::branch(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        5  => patterns::add::add(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        6  => patterns::sub::sub(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        7  => patterns::mul::mul(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        8  => patterns::inv::inv(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        9  => patterns::eq::eq(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        10 => patterns::lt::lt(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        11 => patterns::xor::xor(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        12 => patterns::and::and(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        13 => patterns::not::not(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        14 => patterns::shl::shl(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        15 => patterns::hash::hash(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        16 => patterns::call::call_witness(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        17 => patterns::look::look(order, object, body, budget, hints, tracer, depth, &mut row, registry),
        _  => Outcome::Error(ErrorKind::Malformed),
    };

    if !is_multi_row {
        row.r[3] = match &outcome { Outcome::Ok(r, _) => *r as u64, _ => NIL as u64 };
        row.r[9] = match &outcome { Outcome::Ok(_, b) | Outcome::Halt(b) => *b, Outcome::Error(_) => budget };
        row.r[10] = match &outcome { Outcome::Error(k) => *k as u64, _ => 0 };
        tracer.record(row);
    }

    outcome
}

// === helpers used by pattern implementations (pub(crate)) ===

/// Emit a synthetic error row for fast-fail paths (depth-exceeded, malformed
/// dispatch). Preserves the "every reduce() call emits at least one row"
/// invariant so the verifier can bind the halted step to a budget state.
fn emit_error_row<T: Tracer>(
    tracer: &mut T, object: NounId, formula: NounId,
    tag: u64, budget_in: u64, kind: ErrorKind,
) {
    let mut row = TraceRow::default();
    row.r[0] = tag;
    row.r[1] = object as u64;
    row.r[2] = formula as u64;
    row.r[3] = NIL as u64;
    row.r[8] = budget_in;
    row.r[9] = budget_in; // no cost charged on early-fail
    row.r[10] = kind as u64;
    tracer.record(row);
}

/// Emit a halt row for budget exhaustion. Status: budget_in == budget_out.
fn emit_halt_row<T: Tracer>(
    tracer: &mut T, object: NounId, formula: NounId, tag: u64, budget_in: u64,
) {
    let mut row = TraceRow::default();
    row.r[0] = tag;
    row.r[1] = object as u64;
    row.r[2] = formula as u64;
    row.r[3] = NIL as u64;
    row.r[8] = budget_in;
    row.r[9] = budget_in;
    tracer.record(row);
}

pub(crate) fn cell_pair<const N: usize>(order: &Order<N>, r: NounId) -> Option<(NounId, NounId)> {
    match order.get(r)?.inner {
        Noun::Cell { left, right } => Some((left, right)),
        _ => None,
    }
}

pub(crate) fn evaluate<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(NounId, u64), Outcome> {
    match reduce_inner(order, object, formula, budget, hints, tracer, depth + 1, registry) {
        Outcome::Ok(r, b) => Ok((r, b)),
        other => Err(other),
    }
}

pub(crate) fn make_field<const N: usize>(order: &mut Order<N>, v: Goldilocks, budget: u64) -> Outcome {
    match order.atom(v, Tag::Field) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

// === bound-partitioned evaluation helpers (option β scheduling) =============
//
// `evaluate_binary` / `evaluate_unary` apply the parallel-canonical
// partitioning rule from specs/reduction.md §parallel reduction:
//
// - If both children have static (non-Dynamic) bounds and their sum fits
//   the current budget, each child is evaluated with its OWN bound() as
//   the budget. Unused budget is refunded at the join. This yields the
//   identical witness regardless of execution scheduling — sequential
//   single-thread runs and a future parallel scheduler produce the same
//   per-row budget values.
// - If any child is Dynamic, or the bound sum exceeds the parent's
//   budget, the helper falls back to classical sequential threading
//   (f → f1 → f2). This path agrees with current sequential semantics
//   and preserves the precise halt point.

/// Evaluate two sub-formulas under the parallel-canonical partition rule.
/// Returns (val_a, val_b, remaining_budget_for_parent).
pub(crate) fn evaluate_binary<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId,
    a: NounId, b: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(NounId, NounId, u64), Outcome> {
    let ba_cost = crate::bound::bound(order, a);
    let bb_cost = crate::bound::bound(order, b);
    let ba = ba_cost.value();
    let bb = bb_cost.value();
    // Partition only when both sub-formulas have static bounds AND the sum
    // fits the available budget. Saturating add prevents overflow on
    // pathologically large bounds.
    let can_partition = !ba_cost.is_dynamic()
        && !bb_cost.is_dynamic()
        && ba.saturating_add(bb) <= budget;

    if can_partition {
        #[cfg(feature = "std")]
        {
            // par_binary uses an empty registry in sub-orders (correct: jets
            // fire at the top-level reduce_inner; sub-evaluations use L1).
            return crate::parallel::par_binary(
                order, object, a, b, ba, bb, budget, hints, tracer, depth,
            );
        }
        #[allow(unreachable_code)]
        {
            let (va, ra) = evaluate(order, object, a, ba, hints, tracer, depth, registry)?;
            let (vb, rb) = evaluate(order, object, b, bb, hints, tracer, depth, registry)?;
            let used = (ba - ra).saturating_add(bb - rb);
            Ok((va, vb, budget - used))
        }
    } else {
        let (va, b1) = evaluate(order, object, a, budget, hints, tracer, depth, registry)?;
        let (vb, b2) = evaluate(order, object, b, b1, hints, tracer, depth, registry)?;
        Ok((va, vb, b2))
    }
}

/// Evaluate a single sub-formula under the partition rule.
/// Returns (val, remaining_budget_for_parent).
pub(crate) fn evaluate_unary<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, a: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(NounId, u64), Outcome> {
    let ba_cost = crate::bound::bound(order, a);
    let ba = ba_cost.value();
    let can_partition = !ba_cost.is_dynamic() && ba <= budget;

    if can_partition {
        let (va, ra) = evaluate(order, object, a, ba, hints, tracer, depth, registry)?;
        Ok((va, budget - (ba - ra)))
    } else {
        evaluate(order, object, a, budget, hints, tracer, depth, registry)
    }
}

/// Evaluate two field-typed sub-formulas with partitioning.
pub(crate) fn evaluate_binary_field<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId,
    a: NounId, b: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(Goldilocks, Goldilocks, u64), Outcome> {
    let (ra, rb, remaining) = evaluate_binary(order, object, a, b, budget, hints, tracer, depth, registry)?;
    let va = match order.atom_value(ra) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => v,
        _ => return Err(Outcome::Error(ErrorKind::TypeError)),
    };
    let vb = match order.atom_value(rb) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => v,
        _ => return Err(Outcome::Error(ErrorKind::TypeError)),
    };
    Ok((va, vb, remaining))
}

/// Evaluate two word-typed sub-formulas with partitioning.
pub(crate) fn evaluate_binary_word<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId,
    a: NounId, b: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(u64, u64, u64), Outcome> {
    let (ra, rb, remaining) = evaluate_binary(order, object, a, b, budget, hints, tracer, depth, registry)?;
    let va = match order.atom_value(ra) {
        Some((v, Tag::Word)) => v.as_u64(),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => v.as_u64(),
        _ => return Err(Outcome::Error(ErrorKind::TypeError)),
    };
    let vb = match order.atom_value(rb) {
        Some((v, Tag::Word)) => v.as_u64(),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => v.as_u64(),
        _ => return Err(Outcome::Error(ErrorKind::TypeError)),
    };
    Ok((va, vb, remaining))
}

/// Evaluate a single field-typed sub-formula with partitioning.
pub(crate) fn evaluate_unary_field<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, a: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(Goldilocks, u64), Outcome> {
    let (ra, remaining) = evaluate_unary(order, object, a, budget, hints, tracer, depth, registry)?;
    match order.atom_value(ra) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => Ok((v, remaining)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

/// Evaluate a single word-typed sub-formula with partitioning.
pub(crate) fn evaluate_unary_word<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, a: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    registry: &JetRegistry<N>,
) -> core::result::Result<(u64, u64), Outcome> {
    let (ra, remaining) = evaluate_unary(order, object, a, budget, hints, tracer, depth, registry)?;
    match order.atom_value(ra) {
        Some((v, Tag::Word)) => Ok((v.as_u64(), remaining)),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Ok((v.as_u64(), remaining)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub(crate) fn field_binary_op<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
    registry: &JetRegistry<N>,
    op: fn(Goldilocks, Goldilocks) -> Goldilocks,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (va, vb, budget) = match evaluate_binary_field(order, object, a, b, budget, hints, tracer, depth, registry) {
        Ok(v) => v, Err(o) => return o,
    };
    let vc = op(va, vb);
    row.r[4] = va.as_u64();
    row.r[5] = vb.as_u64();
    row.r[6] = vc.as_u64();
    make_field(order, vc, budget)
}

/// Emit one row at bit position k for a multi-row bit-decomp pattern.
///
/// Layout: r0/r1/r2/r8 are inherited from the template `row` (pre-filled by
/// reduce_inner). r4=a, r5=b, r6=c carry packed operands+result across all rows.
/// r7=k bit position. r10=a_k, r11=b_k, r12=c_k expose the per-bit witness.
/// r3 and r9 are zero on intermediate rows; the caller sets them on the
/// terminal row (k = last) before recording.
pub(crate) fn emit_bit_row<T: Tracer>(
    template: &TraceRow, tracer: &mut T,
    a: u64, b: u64, c: u64,
    k: u32, a_k: u64, b_k: u64, c_k: u64,
    is_last: bool, result_id: NounId, budget_out: u64,
) {
    let mut r = TraceRow::default();
    r.r[0] = template.r[0];
    r.r[1] = template.r[1];
    r.r[2] = template.r[2];
    r.r[4] = a;
    r.r[5] = b;
    r.r[6] = c;
    r.r[7] = k as u64;
    r.r[8] = template.r[8];
    r.r[10] = a_k;
    r.r[11] = b_k;
    r.r[12] = c_k;
    if is_last {
        r.r[3] = result_id as u64;
        r.r[9] = budget_out;
    }
    tracer.record(r);
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::vec::Vec;
    use super::*;
    use crate::call::NullCalls;
    use crate::trace::{NoTrace, VecTrace};
    use nebu::Goldilocks;

    fn g(v: u64) -> Goldilocks { Goldilocks::new(v) }

    /// reduce on a formula that's an atom (not a cell) is Malformed,
    /// and the error path emits a synthetic trace row.
    #[test]
    fn atom_formula_is_malformed_and_records_row() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let formula = ar.atom(g(1), Tag::Field).unwrap(); // atom formula
        let mut tr = VecTrace::default();
        match reduce(&mut ar, obj, formula, 100, &NullCalls, &mut tr) {
            Outcome::Error(ErrorKind::Malformed) => {}
            o => panic!("expected Malformed, got {:?}", o),
        }
        assert_eq!(tr.0.len(), 1, "error path emits one synthetic row");
        assert_eq!(tr.0[0].col(10), ErrorKind::Malformed as u64);
    }

    /// Budget less than the dispatch cost halts before pattern executes
    /// and emits a synthetic halt row.
    #[test]
    fn budget_below_cost_halts_and_records_row() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let t15 = ar.atom(g(15), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let body = ar.cell(t1, obj).unwrap();
        let formula = ar.cell(t15, body).unwrap();
        let mut tr = VecTrace::default();
        // hash cost = 25, budget = 10 → halt before dispatching
        match reduce(&mut ar, obj, formula, 10, &NullCalls, &mut tr) {
            Outcome::Halt(b) => assert_eq!(b, 10),
            o => panic!("expected Halt(10), got {:?}", o),
        }
        assert_eq!(tr.0.len(), 1, "halt path emits one synthetic row");
        assert_eq!(tr.0[0].col(0), 15);
        assert_eq!(tr.0[0].col(8), 10);
        assert_eq!(tr.0[0].col(9), 10);
    }

    /// Construct a small nested compose chain. compose's third reduce
    /// `reduce(obj, frm)` operates on the runtime result of the second
    /// sub-expression. When that result is an atom (not a cell), reduce_inner
    /// returns Malformed. The chain only needs to be deep enough that the
    /// reducer actually walks into the malformed continuation.
    #[test]
    fn malformed_compose_continuation_returns_malformed() {
        let mut ar = Order::<256>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let t2 = ar.atom(g(2), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let zero = ar.atom(g(0), Tag::Field).unwrap();
        let quote_zero = ar.cell(t1, zero).unwrap();
        // [2 [[1 0] [1 0]]] — compose with two quote-zero sub-formulas.
        // After evaluation: obj=0, frm=0 → reduce_inner(0, 0) is Malformed.
        let body = ar.cell(quote_zero, quote_zero).unwrap();
        let formula = ar.cell(t2, body).unwrap();
        match reduce(&mut ar, obj, formula, 1_000_000, &NullCalls, &mut NoTrace) {
            Outcome::Error(ErrorKind::Malformed) => {}
            o => panic!("expected Malformed from compose's atom-formula continuation, got {:?}", o),
        }
    }

    /// Public reduce() returns Outcome::Ok for a trivial formula.
    #[test]
    fn ok_variant_returns_result() {
        let mut ar = Order::<1024>::new();
        let obj = ar.atom(g(0), Tag::Field).unwrap();
        let t1 = ar.atom(g(1), Tag::Field).unwrap();
        let v = ar.atom(g(42), Tag::Field).unwrap();
        let formula = ar.cell(t1, v).unwrap();
        match reduce(&mut ar, obj, formula, 10, &NullCalls, &mut NoTrace) {
            Outcome::Ok(r, b) => {
                assert_eq!(ar.atom_value(r).unwrap().0, g(42));
                assert_eq!(b, 9, "quote costs 1");
            }
            o => panic!("{:?}", o),
        }
    }

    /// Cost table is in sync with the dispatch arm count.
    #[test]
    fn cost_table_length_matches_dispatch() {
        // 18 patterns (0..=17). Jets (poly_eval, merkle_verify, …) are recognized
        // by formula hash at dispatch time — not by dedicated tag slots.
        let _: [u64; 18] = COSTS;
    }

    /// Cost lookups for known multi-row tags return the multi-row cost.
    #[test]
    fn cost_table_multi_row_values() {
        assert_eq!(cost(8), 64);    // inv
        assert_eq!(cost(10), 64);   // lt
        assert_eq!(cost(11), 32);   // xor
        assert_eq!(cost(15), 25);   // hash (24 rounds + 1 squeeze)
        assert_eq!(cost(99), 1);    // unknown tag → default
    }

    // suppress dead-warning on alloc helper
    #[test]
    fn _vec_used() { let _: Vec<u64> = Vec::new(); }
}

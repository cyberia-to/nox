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

// ── pattern tags ──────────────────────────────────────────────
const TAG_INV:  u64 = 8;
const TAG_LT:   u64 = 10;
const TAG_XOR:  u64 = 11;
const TAG_AND:  u64 = 12;
const TAG_NOT:  u64 = 13;
const TAG_SHL:  u64 = 14;
const TAG_HASH: u64 = 15;

// ── budget costs ──────────────────────────────────────────────
// for multi-row patterns: row count = cost. one row per bit of soundness
// witness so zheng can constrain bitwise / comparison gadgets.
const COST_INV:     u64 = 64;
const COST_LT:      u64 = 64;
const COST_XOR:     u64 = 32;
const COST_AND:     u64 = 32;
const COST_NOT:     u64 = 32;
const COST_SHL:     u64 = 32;
const COST_HASH:    u64 = 300;
const COST_DEFAULT: u64 = 1;

// ── word arithmetic ───────────────────────────────────────────
pub(crate) const WORD_MASK: u64 = 0xFFFF_FFFF;

// ── safety ────────────────────────────────────────────────────
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
    match tag {
        TAG_INV  => COST_INV,
        TAG_LT   => COST_LT,
        TAG_XOR  => COST_XOR,
        TAG_AND  => COST_AND,
        TAG_NOT  => COST_NOT,
        TAG_SHL  => COST_SHL,
        TAG_HASH => COST_HASH,
        _        => COST_DEFAULT,
    }
}

/// public entry point — depth starts at 0
pub fn reduce<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T,
) -> Outcome {
    reduce_inner(order, object, formula, budget, hints, tracer, 0)
}

pub(crate) fn reduce_inner<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
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
    let c = cost(tag);
    let budget_in = budget;
    if budget < c {
        emit_halt_row(tracer, object, formula, tag, budget);
        return Outcome::Halt(budget);
    }
    let budget = budget - c;

    // pre-fill common registers; patterns fill r[4]-r[7] (multi-row: all rows)
    let mut row = TraceRow::default();
    row.r[0] = tag;
    row.r[1] = object as u64;
    row.r[2] = formula as u64;
    row.r[8] = budget_in;

    // multi-row patterns emit their own rows. these patterns expose per-bit
    // (or per-step) soundness witnesses that don't fit a single 16-col row.
    let is_multi_row = matches!(
        tag,
        TAG_INV | TAG_LT | TAG_XOR | TAG_AND | TAG_NOT | TAG_SHL | TAG_HASH
    );

    let outcome = match tag {
        0  => patterns::axis::axis(order, object, body, budget, &mut row),
        1  => patterns::quote::quote(body, budget, &mut row),
        2  => patterns::compose::compose(order, object, body, budget, hints, tracer, depth, &mut row),
        3  => patterns::cons::cons(order, object, body, budget, hints, tracer, depth, &mut row),
        4  => patterns::branch::branch(order, object, body, budget, hints, tracer, depth, &mut row),
        5  => patterns::add::add(order, object, body, budget, hints, tracer, depth, &mut row),
        6  => patterns::sub::sub(order, object, body, budget, hints, tracer, depth, &mut row),
        7  => patterns::mul::mul(order, object, body, budget, hints, tracer, depth, &mut row),
        8  => patterns::inv::inv(order, object, body, budget, hints, tracer, depth, &mut row),
        9  => patterns::eq::eq(order, object, body, budget, hints, tracer, depth, &mut row),
        10 => patterns::lt::lt(order, object, body, budget, hints, tracer, depth, &mut row),
        11 => patterns::xor::xor(order, object, body, budget, hints, tracer, depth, &mut row),
        12 => patterns::and::and(order, object, body, budget, hints, tracer, depth, &mut row),
        13 => patterns::not::not(order, object, body, budget, hints, tracer, depth, &mut row),
        14 => patterns::shl::shl(order, object, body, budget, hints, tracer, depth, &mut row),
        15 => patterns::hash::hash(order, object, body, budget, hints, tracer, depth, &mut row),
        16 => patterns::call::call_witness(order, object, body, budget, hints, tracer, depth, &mut row),
        17 => patterns::look::look(order, object, body, budget, hints, tracer, depth, &mut row),
        _  => Outcome::Error(ErrorKind::Malformed),
    };

    if !is_multi_row {
        row.r[3] = match &outcome { Outcome::Ok(r, _) => *r as u64, _ => NIL as u64 };
        row.r[9] = match &outcome { Outcome::Ok(_, b) | Outcome::Halt(b) => *b, Outcome::Error(_) => 0 };
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
) -> core::result::Result<(NounId, u64), Outcome> {
    match reduce_inner(order, object, formula, budget, hints, tracer, depth + 1) {
        Outcome::Ok(r, b) => Ok((r, b)),
        other => Err(other),
    }
}

pub(crate) fn evaluate_field<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
) -> core::result::Result<(Goldilocks, u64), Outcome> {
    let (result, budget) = evaluate(order, object, formula, budget, hints, tracer, depth)?;
    match order.atom_value(result) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => Ok((v, budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub(crate) fn evaluate_word<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
) -> core::result::Result<(u64, u64), Outcome> {
    let (result, budget) = evaluate(order, object, formula, budget, hints, tracer, depth)?;
    match order.atom_value(result) {
        Some((v, Tag::Word)) => Ok((v.as_u64(), budget)),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Ok((v.as_u64(), budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub(crate) fn make_field<const N: usize>(order: &mut Order<N>, v: Goldilocks, budget: u64) -> Outcome {
    match order.atom(v, Tag::Field) {
        Some(r) => Outcome::Ok(r, budget),
        None => Outcome::Error(ErrorKind::Unavailable),
    }
}

pub(crate) fn field_binary_op<const N: usize, T: Tracer>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64,
    hints: &dyn CallProvider<N>, tracer: &mut T, depth: u64,
    row: &mut TraceRow,
    op: fn(Goldilocks, Goldilocks) -> Goldilocks,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) {
        Some(p) => p,
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let (va, budget) = match evaluate_field(order, object, a, budget, hints, tracer, depth) {
        Ok(v) => v, Err(o) => return o,
    };
    let (vb, budget) = match evaluate_field(order, object, b, budget, hints, tracer, depth) {
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

// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! reduce — dispatch + budget metering + shared helpers
//! pattern implementations live in patterns/

use nebu::Goldilocks;
use crate::noun::{Order, NounId, Noun, Tag};
use crate::call::CallProvider;
use crate::patterns;

#[derive(Debug)]
pub enum Outcome {
    Ok(NounId, u64),
    Halt(u64),
    Error(ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TypeError,
    AxisError,
    InvZero,
    Unavailable,
    Malformed,
    CallRejected,
}

fn cost(tag: u64) -> u64 {
    match tag { 8 => 64, 15 => 300, _ => 1 }
}

pub fn reduce<const N: usize>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> Outcome {
    let (tag_ref, body) = match order.get(formula).inner {
        Noun::Cell { left, right } => (left, right),
        Noun::Atom { .. } => return Outcome::Error(ErrorKind::Malformed),
    };
    let tag = match order.atom_value(tag_ref) {
        Some((v, _)) => v.as_u64(),
        None => return Outcome::Error(ErrorKind::Malformed),
    };
    let c = cost(tag);
    if budget < c { return Outcome::Halt(budget); }
    let budget = budget - c;
    match tag {
        0  => patterns::axis::axis(order, object, body, budget),
        1  => patterns::quote::quote(body, budget),
        2  => patterns::compose::compose(order, object, body, budget, hints),
        3  => patterns::cons::cons(order, object, body, budget, hints),
        4  => patterns::branch::branch(order, object, body, budget, hints),
        5  => patterns::add::add(order, object, body, budget, hints),
        6  => patterns::sub::sub(order, object, body, budget, hints),
        7  => patterns::mul::mul(order, object, body, budget, hints),
        8  => patterns::inv::inv(order, object, body, budget, hints),
        9  => patterns::eq::eq(order, object, body, budget, hints),
        10 => patterns::lt::lt(order, object, body, budget, hints),
        11 => patterns::xor::xor(order, object, body, budget, hints),
        12 => patterns::and::and(order, object, body, budget, hints),
        13 => patterns::not::not(order, object, body, budget, hints),
        14 => patterns::shl::shl(order, object, body, budget, hints),
        15 => patterns::hash::hash(order, object, body, budget, hints),
        16 => patterns::call::call_witness(order, object, body, budget, hints),
        17 => patterns::look::look(order, object, body, budget, hints),
        _  => Outcome::Error(ErrorKind::Malformed),
    }
}

// === public helpers used by pattern implementations ===

pub fn cell_pair<const N: usize>(order: &Order<N>, r: NounId) -> Option<(NounId, NounId)> {
    match order.get(r).inner {
        Noun::Cell { left, right } => Some((left, right)),
        _ => None,
    }
}

pub fn evaluate<const N: usize>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> core::result::Result<(NounId, u64), Outcome> {
    match reduce(order, object, formula, budget, hints) {
        Outcome::Ok(r, b) => Ok((r, b)),
        other => Err(other),
    }
}

pub fn evaluate_field<const N: usize>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> core::result::Result<(Goldilocks, u64), Outcome> {
    let (result, budget) = evaluate(order, object, formula, budget, hints)?;
    match order.atom_value(result) {
        Some((v, Tag::Field)) | Some((v, Tag::Word)) => Ok((v, budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub fn evaluate_word<const N: usize>(
    order: &mut Order<N>, object: NounId, formula: NounId, budget: u64, hints: &dyn CallProvider<N>,
) -> core::result::Result<(u64, u64), Outcome> {
    let (result, budget) = evaluate(order, object, formula, budget, hints)?;
    match order.atom_value(result) {
        Some((v, Tag::Word)) => Ok((v.as_u64(), budget)),
        Some((v, Tag::Field)) if v.as_u64() < (1u64 << 32) => Ok((v.as_u64(), budget)),
        _ => Err(Outcome::Error(ErrorKind::TypeError)),
    }
}

pub fn make_field<const N: usize>(order: &mut Order<N>, v: Goldilocks, budget: u64) -> Outcome {
    match order.atom(v, Tag::Field) { Some(r) => Outcome::Ok(r, budget), None => Outcome::Error(ErrorKind::Unavailable) }
}

pub fn make_word<const N: usize>(order: &mut Order<N>, v: u64, budget: u64) -> Outcome {
    match order.atom(Goldilocks::new(v & 0xFFFF_FFFF), Tag::Word) { Some(r) => Outcome::Ok(r, budget), None => Outcome::Error(ErrorKind::Unavailable) }
}

pub fn field_binary_op<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
    op: fn(Goldilocks, Goldilocks) -> Goldilocks,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_field(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_field(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_field(order, op(va, vb), budget)
}

pub fn word_binary_op<const N: usize>(
    order: &mut Order<N>, object: NounId, body: NounId, budget: u64, hints: &dyn CallProvider<N>,
    op: fn(u64, u64) -> u64,
) -> Outcome {
    let (a, b) = match cell_pair(order, body) { Some(p) => p, None => return Outcome::Error(ErrorKind::Malformed) };
    let (va, budget) = match evaluate_word(order, object, a, budget, hints) { Ok(v) => v, Err(o) => return o };
    let (vb, budget) = match evaluate_word(order, object, b, budget, hints) { Ok(v) => v, Err(o) => return o };
    make_word(order, op(va, vb), budget)
}

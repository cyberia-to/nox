use super::*;
use crate::reduce::{cost, emit_error_row, emit_halt_row, pair_children};
use crate::{ErrorKind, NullCalls};

pub(super) fn enter<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    stack: &mut Vec<Frame>,
    tracer: &mut T,
) -> Result<Action, Error> {
    let decoded = pair_children(ar, formula)
        .and_then(|(tag, body)| ar.atom_value(tag).map(|v| (v.as_u64(), body)));
    let Some((tag, body)) = decoded else {
        emit_error_row(tracer, object, formula, 0, budget, ErrorKind::Malformed);
        return Ok(Action::Return(Outcome::Error(ErrorKind::Malformed)));
    };
    if matches!(tag, 16 | 17) {
        return Err(Error::UnsupportedService(tag));
    }
    let c = cost(tag);
    if budget < c {
        emit_halt_row(tracer, object, formula, tag, budget);
        return Ok(Action::Return(Outcome::Halt(budget)));
    }
    let mut row = TraceRow::default();
    row.r[0] = tag;
    row.r[1] = object as u64;
    row.r[2] = formula as u64;
    row.r[8] = budget;
    let budget = budget - c;
    let mut frame = Frame {
        row,
        budget,
        phase: Phase::Compose,
    };
    let outcome = match tag {
        0 => crate::patterns::axis::axis(ar, object, body, budget, &NullCalls, &mut frame.row),
        1 => crate::patterns::quote::quote(body, budget, &mut frame.row),
        8 | 13 | 15 => {
            let reservation = Reservation::unary(ar, body, budget);
            frame.phase = Phase::Unary(reservation);
            return Ok(call(stack, frame, body, reservation.child));
        }
        4 => {
            if let Some((test, rest)) = pair_children(ar, body) {
                if let Some((yes, no)) = pair_children(ar, rest) {
                    let reservation = Reservation::unary(ar, test, budget);
                    frame.phase = Phase::BranchTest {
                        yes,
                        no,
                        reservation,
                    };
                    return Ok(call(stack, frame, test, reservation.child));
                }
            }
            Outcome::Error(ErrorKind::Malformed)
        }
        2 | 3 | 5..=7 | 9..=12 | 14 => {
            if let Some((a, b)) = pair_children(ar, body) {
                let ba = crate::bound(ar, a);
                let bb = crate::bound(ar, b);
                let partition = !ba.is_dynamic()
                    && !bb.is_dynamic()
                    && ba.value().saturating_add(bb.value()) <= budget;
                let first = if partition { ba.value() } else { budget };
                let second = if partition { Some(bb.value()) } else { None };
                frame.phase = Phase::BinaryLeft {
                    a,
                    b,
                    first,
                    second,
                };
                return Ok(call(stack, frame, a, first));
            }
            Outcome::Error(ErrorKind::Malformed)
        }
        _ => Outcome::Error(ErrorKind::Malformed),
    };
    Ok(complete(frame, outcome, tracer))
}

pub(super) fn resume<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    mut frame: Frame,
    outcome: Outcome,
    stack: &mut Vec<Frame>,
    tracer: &mut T,
) -> Action {
    let (value, remaining) = match outcome {
        Outcome::Ok(value, remaining) => (value, remaining),
        failed => {
            if frame.row.r[0] == 8 {
                finish::inv_input_failure(&mut frame.row, &failed, tracer);
            }
            return complete(frame, failed, tracer);
        }
    };
    let outcome = match frame.phase {
        Phase::Unary(reservation) => finish::unary(
            ar,
            value,
            reservation.refund(remaining),
            &mut frame.row,
            tracer,
        ),
        Phase::BinaryLeft {
            a,
            b,
            first,
            second,
        } => {
            let second = second.unwrap_or(remaining);
            frame.phase = Phase::BinaryRight {
                a,
                b,
                left: value,
                used: first - remaining,
                second,
            };
            return call(stack, frame, b, second);
        }
        Phase::BinaryRight {
            a,
            b,
            left,
            used,
            second,
        } => {
            let joined = frame.budget - used - (second - remaining);
            if frame.row.r[0] == 2 {
                frame.row.r[4] = left as u64;
                frame.row.r[5] = value as u64;
                frame.row.r[6] = a as u64;
                frame.row.r[7] = b as u64;
                frame.phase = Phase::Compose;
                stack.push(frame);
                return Action::Enter {
                    object: left,
                    formula: value,
                    budget: joined,
                };
            }
            finish::binary(ar, left, value, joined, &mut frame.row, tracer)
        }
        Phase::BranchTest {
            yes,
            no,
            reservation,
        } => {
            let Some(test) = ar.atom_value(value) else {
                return complete(frame, Outcome::Error(ErrorKind::TypeError), tracer);
            };
            let v = test.as_u64();
            frame.row.r[4] = v;
            frame.row.r[5] = if v == 0 { 0 } else { test.inv().as_u64() };
            frame.row.r[10] = if v == 0 { 0 } else { 1 };
            let chosen = if v == 0 { yes } else { no };
            let next = Reservation::unary(ar, chosen, reservation.refund(remaining));
            frame.phase = Phase::BranchChosen(next);
            return call(stack, frame, chosen, next.child);
        }
        Phase::BranchChosen(reservation) => {
            frame.row.r[6] = value as u64;
            Outcome::Ok(value, reservation.refund(remaining))
        }
        Phase::Compose => Outcome::Ok(value, remaining),
    };
    complete(frame, outcome, tracer)
}

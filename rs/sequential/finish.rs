//! Operations on evaluated operands; shared multirow emitters preserve traces.
use crate::patterns;
use crate::{ErrorKind, NIL, Order, Outcome, Reduction, TraceRow, Tracer};

pub(super) fn inv_input_failure<T: Tracer>(row: &mut TraceRow, outcome: &Outcome, tracer: &mut T) {
    row.r[3] = NIL as u64;
    row.r[9] = 0;
    row.r[10] = match outcome {
        Outcome::Error(e) => *e as u64,
        _ => 0,
    };
    tracer.record(*row);
}

pub(super) fn unary<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    value: Order,
    budget: u64,
    row: &mut TraceRow,
    tracer: &mut T,
) -> Outcome {
    if row.r[0] == 15 {
        return patterns::hash::finish(ar, value, budget, row, tracer);
    }
    let field = ar.atom_value(value);
    match (row.r[0], field) {
        (8, Some(v)) => patterns::inv::finish(ar, v, budget, row, tracer),
        (13, Some(v)) if v.as_u64() < 1 << 32 => {
            patterns::not::finish(ar, v.as_u64(), budget, row, tracer)
        }
        _ => {
            let error = Outcome::Error(ErrorKind::TypeError);
            if row.r[0] == 8 {
                inv_input_failure(row, &error, tracer);
            }
            error
        }
    }
}

pub(super) fn binary<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    a: Order,
    b: Order,
    budget: u64,
    row: &mut TraceRow,
    tracer: &mut T,
) -> Outcome {
    let tag = row.r[0];
    if tag == 3 {
        row.r[4] = a as u64;
        row.r[5] = b as u64;
        return match ar.pair(a, b) {
            Some(r) => Outcome::Ok(r, budget),
            None => Outcome::Error(ErrorKind::Unavailable),
        };
    }
    if tag == 9 {
        return patterns::eq::finish(ar, a, b, budget, row);
    }
    let (Some(a), Some(b)) = (ar.atom_value(a), ar.atom_value(b)) else {
        return Outcome::Error(ErrorKind::TypeError);
    };
    match tag {
        5..=7 => {
            let c = match tag {
                5 => a + b,
                6 => a - b,
                _ => a * b,
            };
            row.r[4] = a.as_u64();
            row.r[5] = b.as_u64();
            row.r[6] = c.as_u64();
            crate::reduce::make_field(ar, c, budget)
        }
        10 => patterns::lt::finish(ar, a, b, budget, row, tracer),
        11 | 12 | 14 if a.as_u64() < 1 << 32 && b.as_u64() < 1 << 32 => match tag {
            11 => patterns::xor::finish(ar, a.as_u64(), b.as_u64(), budget, row, tracer),
            12 => patterns::and::finish(ar, a.as_u64(), b.as_u64(), budget, row, tracer),
            _ => patterns::shl::finish(ar, a.as_u64(), b.as_u64(), budget, row, tracer),
        },
        _ => Outcome::Error(ErrorKind::TypeError),
    }
}

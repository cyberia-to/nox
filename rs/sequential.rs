//! Bounded pure L1 evaluation with an explicit, sequential continuation stack.
use crate::{Order, Outcome, Reduction, TraceRow, Tracer};
use alloc::vec::Vec;

mod dispatch;
mod finish;

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Active invocations including the root, not noun depth or reduction cost.
    pub max_frames: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Frames,
    Allocation,
    Cancelled,
    UnsupportedService(u64),
}

#[derive(Debug)]
pub struct Execution {
    pub outcome: Outcome,
    pub peak_frames: u32,
}

#[derive(Clone, Copy)]
struct Reservation {
    parent: u64,
    child: u64,
}

impl Reservation {
    fn unary<const N: usize>(ar: &Reduction<N>, formula: Order, budget: u64) -> Self {
        let bound = crate::bound(ar, formula);
        let child = if !bound.is_dynamic() && bound.value() <= budget {
            bound.value()
        } else {
            budget
        };
        Self {
            parent: budget,
            child,
        }
    }

    fn refund(self, remaining: u64) -> u64 {
        self.parent - (self.child - remaining)
    }
}

#[derive(Clone, Copy)]
enum Phase {
    Unary(Reservation),
    BinaryLeft {
        a: Order,
        b: Order,
        first: u64,
        second: Option<u64>,
    },
    BinaryRight {
        a: Order,
        b: Order,
        left: Order,
        used: u64,
        second: u64,
    },
    BranchTest {
        yes: Order,
        no: Order,
        reservation: Reservation,
    },
    BranchChosen(Reservation),
    Compose,
}

struct Frame {
    row: TraceRow,
    budget: u64,
    phase: Phase,
}

enum Action {
    Enter {
        object: Order,
        formula: Order,
        budget: u64,
    },
    Return(Outcome),
}

/// Requested frame buffer bytes; excludes Vec metadata and allocator overhead.
pub fn frame_storage_bytes(limits: Limits) -> Option<usize> {
    (limits.max_frames as usize).checked_mul(core::mem::size_of::<Frame>())
}

/// Execute tags0..15 without jets, provider callbacks, host recursion or forks.
/// Profile/resource errors abort and must never be published as successful data.
pub fn reduce<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: Limits,
    tracer: &mut T,
) -> Result<Execution, Error> {
    reduce_controlled(ar, object, formula, budget, limits, tracer, &mut || false)
}

/// A host resource guard may cancel between bounded evaluator steps. It has no
/// access to VM values and must not provide guest computation or witnesses.
pub fn reduce_controlled<const N: usize, T: Tracer>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: Limits,
    tracer: &mut T,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Execution, Error> {
    if limits.max_frames == 0 {
        return Err(Error::Frames);
    }
    if cancelled() {
        return Err(Error::Cancelled);
    }
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(limits.max_frames as usize)
        .map_err(|_| Error::Allocation)?;
    let mut action = Action::Enter {
        object,
        formula,
        budget,
    };
    let mut peak_frames = 0;
    loop {
        if cancelled() {
            return Err(Error::Cancelled);
        }
        action = match action {
            Action::Enter {
                object,
                formula,
                budget,
            } => {
                if stack.len() >= limits.max_frames as usize {
                    return Err(Error::Frames);
                }
                peak_frames = peak_frames.max(stack.len() as u32 + 1);
                dispatch::enter(ar, object, formula, budget, &mut stack, tracer)?
            }
            Action::Return(outcome) => match stack.pop() {
                Some(frame) => dispatch::resume(ar, frame, outcome, &mut stack, tracer),
                None => {
                    return Ok(Execution {
                        outcome,
                        peak_frames,
                    });
                }
            },
        };
    }
}

fn call(stack: &mut Vec<Frame>, frame: Frame, formula: Order, budget: u64) -> Action {
    let object = frame.row.r[1] as Order;
    stack.push(frame);
    Action::Enter {
        object,
        formula,
        budget,
    }
}

fn complete<T: Tracer>(mut frame: Frame, outcome: Outcome, tracer: &mut T) -> Action {
    if !matches!(frame.row.r[0], 8 | 10..=15) {
        frame.row.r[3] = match &outcome {
            Outcome::Ok(r, _) => *r as u64,
            _ => crate::NIL as u64,
        };
        frame.row.r[9] = match &outcome {
            Outcome::Ok(_, b) | Outcome::Halt(b) => *b,
            Outcome::Error(_) => frame.budget,
        };
        if let Outcome::Error(kind) = &outcome {
            frame.row.r[10] = *kind as u64;
        }
        tracer.record(frame.row);
    }
    Action::Return(outcome)
}

#[cfg(test)]
mod tests;

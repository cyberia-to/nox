use super::super::{Action, Frame, Outcome, Phase, Reservation, compacting};
use super::{hook::Hook, *};
use crate::data::Data;

pub(super) struct Capture<'a, O> {
    sink: &'a mut O,
    limits: CaptureLimits,
    pub stats: CaptureStats,
    sequence: u64,
}

pub(super) struct Before {
    action: LogicalAction,
    depth: u32,
    popped: Option<LiveFrame>,
    count: u32,
}

impl<'a, O: stream::Stream> Capture<'a, O> {
    pub fn new(sink: &'a mut O, limits: CaptureLimits) -> Self {
        Self {
            sink,
            limits,
            stats: CaptureStats::default(),
            sequence: 0,
        }
    }
    fn work(
        &mut self,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), CaptureFailure<O::Error>> {
        if self.stats.work == self.limits.max_work {
            return Err(CaptureFailure::Work);
        }
        self.stats.work += 1;
        if cancelled() {
            return Err(CaptureFailure::Cancelled);
        }
        Ok(())
    }
    fn delivery(
        &mut self,
        encoded_bytes: impl FnOnce() -> u64,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), CaptureFailure<O::Error>> {
        self.work(cancelled)?;
        if self.stats.events == self.limits.max_events {
            return Err(CaptureFailure::Events);
        }
        let bytes = encoded_bytes();
        if bytes > self.limits.max_bytes - self.stats.bytes {
            return Err(CaptureFailure::Bytes);
        }
        self.stats.events += 1;
        self.stats.bytes += bytes;
        Ok(())
    }
    fn emit(
        &mut self,
        event: Event,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), CaptureFailure<O::Error>> {
        self.delivery(|| event.encode().as_bytes().len() as u64, cancelled)?;
        self.sink.record(event).map_err(CaptureFailure::Sink)
    }
    fn particle<const N: usize>(
        ar: &Reduction<N>,
        id: Order,
    ) -> Result<Particle, CaptureFailure<O::Error>> {
        ar.digest(id)
            .map(|d| d.map(|v| v.as_u64()))
            .ok_or(CaptureFailure::InvalidArena)
    }
    fn node<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        id: Order,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), CaptureFailure<O::Error>> {
        self.work(cancelled)?;
        let entry = ar.get(id).ok_or(CaptureFailure::InvalidArena)?;
        let value = match entry.inner {
            Data::Atom { value } => NodeValue::Atom(value.as_u64()),
            Data::Pair { left, right } => {
                if left >= id || right >= id {
                    return Err(CaptureFailure::InvalidArena);
                }
                NodeValue::Pair {
                    left: Self::particle(ar, left)?,
                    right: Self::particle(ar, right)?,
                }
            }
        };
        self.emit(
            Event::Node(Node {
                particle: Self::particle(ar, id)?,
                value,
                bound: entry.bound,
            }),
            cancelled,
        )
    }
    fn action<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<LogicalAction, CaptureFailure<O::Error>> {
        self.work(cancelled)?;
        Ok(match action {
            Action::Enter {
                object,
                formula,
                budget,
            } => LogicalAction::Enter {
                object: Self::particle(ar, *object)?,
                formula: Self::particle(ar, *formula)?,
                budget: *budget,
            },
            Action::Return(Outcome::Ok(value, remaining)) => LogicalAction::Return {
                value: Self::particle(ar, *value)?,
                remaining: *remaining,
            },
            Action::Return(Outcome::Halt(remaining)) => LogicalAction::Halt {
                remaining: *remaining,
            },
            Action::Return(Outcome::Error(error)) => LogicalAction::Error(*error),
        })
    }
    fn frame<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        frame: &Frame,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<LiveFrame, CaptureFailure<O::Error>> {
        self.work(cancelled)?;
        let reservation = |r: Reservation| BudgetReservation {
            parent: r.parent,
            child: r.child,
        };
        let opcode = frame.row.r[0];
        Ok(match frame.phase {
            Phase::Unary(r) => LiveFrame::Unary {
                opcode,
                reservation: reservation(r),
            },
            Phase::BinaryLeft {
                b, first, second, ..
            } => LiveFrame::BinaryLeft {
                opcode,
                object: Self::particle(ar, frame.row.r[1] as Order)?,
                right: Self::particle(ar, b)?,
                budget: frame.budget,
                first,
                second,
            },
            Phase::BinaryRight {
                left, used, second, ..
            } => LiveFrame::BinaryRight {
                opcode,
                left: Self::particle(ar, left)?,
                budget: frame.budget,
                used,
                second,
            },
            Phase::BranchTest {
                yes,
                no,
                reservation: r,
            } => LiveFrame::BranchTest {
                object: Self::particle(ar, frame.row.r[1] as Order)?,
                yes: Self::particle(ar, yes)?,
                no: Self::particle(ar, no)?,
                reservation: reservation(r),
            },
            Phase::BranchChosen(r) => LiveFrame::BranchChosen(reservation(r)),
            Phase::Compose => LiveFrame::Compose,
        })
    }
}

impl<O: stream::Stream> Hook for Capture<'_, O> {
    type Error = CaptureFailure<O::Error>;
    type Before = Before;
    fn begin<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        limits: CompactionLimits,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error> {
        let initial = self.action(ar, action, cancelled)?;
        self.emit(
            Event::Begin {
                version: O::VERSION,
                initial,
                initial_nodes: ar.count(),
                max_frames: limits.max_frames,
                max_total_allocations: limits.max_total_allocations,
                max_collection_work: limits.max_collection_work,
                resident_limit: ar.allocation_limit(),
            },
            cancelled,
        )?;
        for id in 0..ar.count() {
            self.node(ar, id, cancelled)?;
        }
        Ok(())
    }
    fn collected<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error> {
        if O::VERSION == 1 {
            return Ok(());
        }
        let reset = EventV2::ArenaReset {
            next_sequence: self.sequence,
            live_nodes: ar.count(),
        };
        self.delivery(|| reset.encode().as_bytes().len() as u64, cancelled)?;
        self.sink
            .reset(self.sequence, ar.count())
            .map_err(CaptureFailure::Sink)?;
        for id in 0..ar.count() {
            self.node(ar, id, cancelled)?;
        }
        Ok(())
    }
    fn before<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        stack: &[Frame],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Before, Self::Error> {
        let logical = self.action(ar, action, cancelled)?;
        let popped = if matches!(action, Action::Return(_)) {
            stack
                .last()
                .map(|f| self.frame(ar, f, cancelled))
                .transpose()?
        } else {
            None
        };
        Ok(Before {
            action: logical,
            depth: stack.len() as u32,
            popped,
            count: ar.count(),
        })
    }
    fn after<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        before: Before,
        step: &compacting::Step,
        stack: &[Frame],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error> {
        for id in before.count..ar.count() {
            self.node(ar, id, cancelled)?;
        }
        let depth = stack.len() as u32;
        let base = before.depth - u32::from(before.popped.is_some());
        let pushed = if depth > base {
            stack
                .last()
                .map(|f| self.frame(ar, f, cancelled))
                .transpose()?
        } else {
            None
        };
        let after = match step {
            compacting::Step::Next(action) => self.action(ar, action, cancelled)?,
            compacting::Step::Done(outcome) => {
                let action = match *outcome {
                    Outcome::Ok(value, remaining) => Action::Return(Outcome::Ok(value, remaining)),
                    Outcome::Halt(remaining) => Action::Return(Outcome::Halt(remaining)),
                    Outcome::Error(error) => Action::Return(Outcome::Error(error)),
                };
                self.action(ar, &action, cancelled)?
            }
        };
        self.emit(
            Event::Transition(Transition {
                sequence: self.sequence,
                before: before.action,
                after,
                depth_before: before.depth,
                depth_after: depth,
                popped: before.popped,
                pushed,
                fresh_nodes: ar.count() - before.count,
            }),
            cancelled,
        )?;
        self.sequence += 1; // each transition already consumed a bounded event
        if let compacting::Step::Done(Outcome::Ok(_, remaining)) = step {
            if let LogicalAction::Return { value, .. } = after {
                self.emit(
                    Event::Completed {
                        steps: self.sequence,
                        value,
                        remaining: *remaining,
                    },
                    cancelled,
                )?;
            }
        }
        Ok(())
    }
}

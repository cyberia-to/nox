//! Bounded, opt-in witness capture. Events carry host observations, not proofs.
use super::{CompactingExecution, CompactionFailureKind, CompactionLimits, CompactionStats};
use crate::{ErrorKind, Order, Reduction};

mod capture;
pub(super) mod hook;
mod wire;
pub use wire::EncodedEvent;

/// Four canonical Hemera limbs; independent of physical arena indices.
pub type Particle = [u64; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureLimits {
    pub max_events: u64,
    /// Logical bytes in Event::encode(), excluding sink storage overhead.
    pub max_bytes: u64,
    /// Node exports, action/frame captures and event delivery attempts.
    pub max_work: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaptureStats {
    pub events: u64,
    pub bytes: u64,
    pub work: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureFailure<E> {
    Events,
    Bytes,
    Work,
    Cancelled,
    InvalidArena,
    Sink(E),
}

#[derive(Debug)]
pub enum ObservedFailureKind<E> {
    Execution(CompactionFailureKind),
    Capture(CaptureFailure<E>),
}

#[derive(Debug)]
pub struct ObservedFailure<E> {
    pub kind: ObservedFailureKind<E>,
    pub stats: CompactionStats,
    pub peak_frames: u32,
    pub capture: CaptureStats,
}

#[derive(Debug)]
pub struct ObservedExecution {
    pub execution: CompactingExecution,
    pub capture: CaptureStats,
}

/// Accept each event atomically. Err must leave that event unpublished.
/// Callback storage/work is the sink's responsibility; it receives no arena.
pub trait Observer {
    type Error;
    fn record(&mut self, event: Event) -> Result<(), Self::Error>;
}

/// Legacy execution uses this hook without computing logical observations.
pub struct NoObserver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalAction {
    Enter {
        object: Particle,
        formula: Particle,
        budget: u64,
    },
    Return {
        value: Particle,
        remaining: u64,
    },
    Halt {
        remaining: u64,
    },
    Error(ErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetReservation {
    pub parent: u64,
    pub child: u64,
}

/// Only references retained by the semantic continuation survive collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveFrame {
    Unary {
        opcode: u64,
        reservation: BudgetReservation,
    },
    BinaryLeft {
        opcode: u64,
        object: Particle,
        right: Particle,
        budget: u64,
        first: u64,
        second: Option<u64>,
    },
    BinaryRight {
        opcode: u64,
        left: Particle,
        budget: u64,
        used: u64,
        second: u64,
    },
    BranchTest {
        object: Particle,
        yes: Particle,
        no: Particle,
        reservation: BudgetReservation,
    },
    BranchChosen(BudgetReservation),
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeValue {
    Atom(u64),
    Pair { left: Particle, right: Particle },
}

/// Cached identity and bound are witness metadata, independently checkable later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node {
    pub particle: Particle,
    pub value: NodeValue,
    pub bound: crate::data::Cost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub sequence: u64,
    pub before: LogicalAction,
    pub after: LogicalAction,
    pub depth_before: u32,
    pub depth_after: u32,
    pub popped: Option<LiveFrame>,
    pub pushed: Option<LiveFrame>,
    pub fresh_nodes: u32,
}

/// Encoding tags follow declaration order, beginning at zero, for each enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Begin {
        version: u64,
        initial: LogicalAction,
        initial_nodes: u32,
        max_frames: u32,
        max_total_allocations: u64,
        max_collection_work: u64,
        resident_limit: u32,
    },
    Node(Node),
    Transition(Transition),
    Completed {
        steps: u64,
        value: Particle,
        remaining: u64,
    },
}

/// Captures pure compacting execution with independent, explicit capture caps.
/// Success of the evaluator alone is insufficient if capture fails.
#[allow(clippy::too_many_arguments)]
pub fn reduce_compacting_observed_controlled<const N: usize, O: Observer>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: CompactionLimits,
    capture_limits: CaptureLimits,
    observer: &mut O,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ObservedExecution, ObservedFailure<O::Error>> {
    let mut capture = capture::Capture::new(observer, capture_limits);
    let result =
        super::compacting::run(ar, object, formula, budget, limits, cancelled, &mut capture);
    match result {
        Ok(execution) => Ok(ObservedExecution {
            execution,
            capture: capture.stats,
        }),
        Err(failure) => Err(ObservedFailure {
            kind: match failure.kind {
                hook::RunError::Execution(kind) => ObservedFailureKind::Execution(kind),
                hook::RunError::Capture(kind) => ObservedFailureKind::Capture(kind),
            },
            stats: failure.stats,
            peak_frames: failure.peak_frames,
            capture: capture.stats,
        }),
    }
}

#[cfg(test)]
mod tests;

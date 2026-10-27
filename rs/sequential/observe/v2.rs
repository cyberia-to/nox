//! Opt-in resident snapshots; ordinary events retain the version-1 encoding.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventV2 {
    Event(Event),
    /// Exactly live_nodes ordinary Node events follow before any transition.
    ArenaReset {
        next_sequence: u64,
        live_nodes: u32,
    },
}

/// Accept each event atomically. Err must leave that event unpublished.
/// Snapshot storage and callback work are the sink's bounded responsibility.
pub trait ObserverV2 {
    type Error;
    fn record(&mut self, event: EventV2) -> Result<(), Self::Error>;
}

/// Captures version-2 events, including a complete arena snapshot after GC.
/// Partial capture, including an interrupted snapshot, cannot report success.
#[allow(clippy::too_many_arguments)]
pub fn reduce_compacting_observed_v2_controlled<const N: usize, O: ObserverV2>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: CompactionLimits,
    capture_limits: CaptureLimits,
    observer: &mut O,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ObservedExecution, ObservedFailure<O::Error>> {
    run_observed(
        ar,
        object,
        formula,
        budget,
        limits,
        capture_limits,
        &mut stream::V2(observer),
        cancelled,
    )
}

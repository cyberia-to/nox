//! Opt-in resident-bounded execution. Legacy evaluation never enters this path.
use super::*;
use crate::data::reduction::compaction::{Fault, Scratch};

mod collection;
mod roots;

#[derive(Debug, Clone, Copy)]
pub struct CompactionLimits {
    pub max_frames: u32,
    /// Includes every entry present at execution entry and every fresh allocation.
    pub max_total_allocations: u64,
    /// Includes planning probes and pre-admitted linear commit visits.
    pub max_collection_work: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CompactionStats {
    pub pinned_nodes: u32,
    pub resident_nodes: u32,
    pub peak_resident_nodes: u32,
    pub total_allocations: u64,
    pub reclaimed_nodes: u64,
    pub collections: u64,
    pub collection_work: u64,
    pub scratch_bytes: usize,
    pub evaluator_checkpoints: u64,
    pub collection_checkpoints: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionFailureKind {
    Execution(Error),
    TotalAllocations,
    CollectionWork,
    InvalidArena,
}

#[derive(Debug, Clone, Copy)]
pub struct CompactionFailure {
    pub kind: CompactionFailureKind,
    pub stats: CompactionStats,
    pub peak_frames: u32,
}

#[derive(Debug)]
pub struct CompactingExecution {
    pub outcome: Outcome,
    pub peak_frames: u32,
    pub stats: CompactionStats,
}

impl From<Error> for CompactionFailureKind {
    fn from(error: Error) -> Self {
        Self::Execution(error)
    }
}

impl From<Fault> for CompactionFailureKind {
    fn from(error: Fault) -> Self {
        match error {
            Fault::Allocation => Self::Execution(Error::Allocation),
            Fault::Cancelled => Self::Execution(Error::Cancelled),
            Fault::Work => Self::CollectionWork,
            Fault::InvalidArena => Self::InvalidArena,
        }
    }
}

/// Requested scratch buffer bytes, excluding allocator/Vec metadata.
pub fn compaction_storage_bytes(physical_capacity: usize, resident_limit: u32) -> Option<usize> {
    crate::data::reduction::compaction::storage_bytes(physical_capacity, resident_limit)
}

pub fn reduce_compacting_cached<const N: usize>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: CompactionLimits,
) -> Result<CompactingExecution, CompactionFailure> {
    reduce_compacting_cached_controlled(ar, object, formula, budget, limits, &mut || false)
}

/// Caller-held pre-entry Orders stay valid; only internal new Orders may move.
/// Cancellation during a pre-admitted commit is reported after arena repair.
pub fn reduce_compacting_cached_controlled<const N: usize>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: CompactionLimits,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<CompactingExecution, CompactionFailure> {
    let mut stats = CompactionStats {
        pinned_nodes: ar.count(),
        resident_nodes: ar.count(),
        peak_resident_nodes: ar.count(),
        total_allocations: u64::from(ar.count()),
        ..CompactionStats::default()
    };
    let mut peak_frames = 0;
    let result = execute_compacting(
        ar,
        object,
        formula,
        budget,
        limits,
        cancelled,
        &mut stats,
        &mut peak_frames,
    );
    stats.resident_nodes = ar.count();
    match result {
        Ok(outcome) => Ok(CompactingExecution {
            outcome,
            peak_frames,
            stats,
        }),
        Err(kind) => Err(CompactionFailure {
            kind,
            stats,
            peak_frames,
        }),
    }
}

fn execute_compacting<const N: usize>(
    ar: &mut Reduction<N>,
    object: Order,
    formula: Order,
    budget: u64,
    limits: CompactionLimits,
    cancelled: &mut impl FnMut() -> bool,
    stats: &mut CompactionStats,
    peak_frames: &mut u32,
) -> Result<Outcome, CompactionFailureKind> {
    if limits.max_frames == 0 {
        return Err(Error::Frames.into());
    }
    checkpoint(stats, cancelled)?;
    if stats.total_allocations > limits.max_total_allocations {
        return Err(CompactionFailureKind::TotalAllocations);
    }
    let resident_limit = ar.allocation_limit();
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(limits.max_frames as usize)
        .map_err(|_| Error::Allocation)?;
    let mut cache = finalizer_cache::Cache::<true>::new()?;
    let mut scratch = Scratch::new::<N>(resident_limit)?;
    stats.scratch_bytes = compaction_storage_bytes(N, resident_limit).ok_or(Error::Allocation)?;
    let mut action = Action::Enter {
        object,
        formula,
        budget,
    };
    loop {
        checkpoint(stats, cancelled)?;
        let headroom = roots::allocation_headroom(ar, &action, &stack, limits.max_frames);
        if headroom > resident_limit - ar.count() {
            collection::collect(
                ar,
                &mut action,
                &mut stack,
                &mut cache,
                &mut scratch,
                limits.max_collection_work,
                stats,
                cancelled,
            )?;
        }
        let before = ar.count();
        let allowance = (limits.max_total_allocations - stats.total_allocations)
            .min(u64::from(resident_limit - before)) as u32;
        ar.set_step_allocation_limit(before + allowance);
        // Store the result before propagating errors so the caller's resident
        // allowance is restored even when a service/frame error is returned.
        let next = step(
            ar,
            action,
            &mut stack,
            &mut cache,
            limits.max_frames,
            peak_frames,
        );
        ar.set_step_allocation_limit(resident_limit);
        stats.total_allocations += u64::from(ar.count() - before);
        stats.peak_resident_nodes = stats.peak_resident_nodes.max(ar.count());
        match next? {
            Step::Done(outcome) => return Ok(outcome),
            Step::Next(next) => action = next,
        }
        if matches!(
            action,
            Action::Return(Outcome::Error(crate::ErrorKind::Unavailable))
        ) && headroom > 0
            && stats.total_allocations == limits.max_total_allocations
        {
            return Err(CompactionFailureKind::TotalAllocations);
        }
    }
}

fn checkpoint(
    stats: &mut CompactionStats,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), CompactionFailureKind> {
    stats.evaluator_checkpoints += 1;
    if cancelled() {
        Err(Error::Cancelled.into())
    } else {
        Ok(())
    }
}

enum Step {
    Next(Action),
    Done(Outcome),
}

fn step<const N: usize>(
    ar: &mut Reduction<N>,
    action: Action,
    stack: &mut Vec<Frame>,
    cache: &mut finalizer_cache::Cache<true>,
    max_frames: u32,
    peak_frames: &mut u32,
) -> Result<Step, Error> {
    Ok(match action {
        Action::Enter {
            object,
            formula,
            budget,
        } => {
            if stack.len() >= max_frames as usize {
                return Err(Error::Frames);
            }
            *peak_frames = (*peak_frames).max(stack.len() as u32 + 1);
            Step::Next(dispatch::enter(
                ar,
                object,
                formula,
                budget,
                stack,
                &mut crate::NoTrace,
            )?)
        }
        Action::Return(outcome) => match stack.pop() {
            Some(frame) => Step::Next(dispatch::resume(
                ar,
                frame,
                outcome,
                stack,
                &mut crate::NoTrace,
                cache,
            )),
            None => Step::Done(outcome),
        },
    })
}

#[cfg(test)]
mod tests;

use super::*;

struct Meter<'a, F> {
    limit: u64,
    stats: &'a mut CompactionStats,
    cancelled: &'a mut F,
}

impl<F: FnMut() -> bool> Meter<'_, F> {
    fn reserve(&mut self, units: u64) -> Result<(), Fault> {
        let total = self
            .stats
            .collection_work
            .checked_add(units)
            .ok_or(Fault::Work)?;
        if total > self.limit {
            return Err(Fault::Work);
        }
        self.stats.collection_work = total;
        Ok(())
    }

    fn tick(&mut self) -> Result<(), Fault> {
        self.reserve(1)?;
        if self.stats.collection_work.is_multiple_of(4096) {
            self.poll()?;
        }
        Ok(())
    }

    fn poll(&mut self) -> Result<(), Fault> {
        self.stats.collection_checkpoints += 1;
        if (self.cancelled)() {
            Err(Fault::Cancelled)
        } else {
            Ok(())
        }
    }
}

pub(super) fn collect<const N: usize>(
    ar: &mut Reduction<N>,
    action: &mut Action,
    stack: &mut [Frame],
    cache: &mut finalizer_cache::Cache<true>,
    scratch: &mut Scratch,
    work_limit: u64,
    stats: &mut CompactionStats,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(), Fault> {
    let count = ar.count();
    let pinned = stats.pinned_nodes;
    let mut meter = Meter {
        limit: work_limit,
        stats,
        cancelled,
    };
    meter.poll()?;
    scratch.prepare(ar, pinned, &mut || meter.tick())?;
    let mut invalid = false;
    let mut mark = |id| {
        invalid |= scratch.root(id, count).is_err();
        id
    };
    meter.tick()?;
    roots::action(action, &mut mark);
    for frame in stack.iter_mut() {
        meter.tick()?;
        roots::frame(frame, &mut mark);
    }
    if invalid {
        return Err(Fault::InvalidArena);
    }
    let live = scratch.plan(ar, &mut || meter.tick())?;
    // Pre-admit every metadata-copy iteration. No hashing, probing, allocation
    // or fallible callback occurs while the arena/index are being rewritten.
    let commit_work = u64::from(count) + N as u64 + 1 + stack.len() as u64 + cache.slots() as u64;
    meter.reserve(commit_work)?;
    meter.poll()?;
    cache.clear();
    scratch.commit(ar, live);
    roots::action(action, &mut |id| scratch.remap(id));
    for frame in stack {
        roots::frame(frame, &mut |id| scratch.remap(id));
    }
    meter.stats.collections += 1;
    meter.stats.reclaimed_nodes += u64::from(count - live);
    meter.stats.resident_nodes = live;
    meter.poll()?;
    Ok(())
}

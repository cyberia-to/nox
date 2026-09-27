//! Planning is cancellable and read-only; commit is a bounded linear pass.
use super::{Data, Goldilocks, NIL, Order, Reduction};
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Fault {
    Allocation,
    Work,
    Cancelled,
    InvalidArena,
}

pub(crate) fn storage_bytes(physical: usize, resident: u32) -> Option<usize> {
    physical
        .checked_add(resident as usize)?
        .checked_mul(core::mem::size_of::<Order>())
}

pub(crate) struct Scratch {
    forward: Vec<Order>,
    index: Vec<Order>,
}

impl Scratch {
    pub(crate) fn new<const N: usize>(resident: u32) -> Result<Self, Fault> {
        fn buffer(len: usize) -> Result<Vec<Order>, Fault> {
            let mut buffer = Vec::new();
            buffer
                .try_reserve_exact(len)
                .map_err(|_| Fault::Allocation)?;
            buffer.resize(len, NIL);
            Ok(buffer)
        }
        Ok(Self {
            forward: buffer(resident as usize)?,
            index: buffer(N)?,
        })
    }

    pub(crate) fn prepare<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        pinned: u32,
        work: &mut impl FnMut() -> Result<(), Fault>,
    ) -> Result<(), Fault> {
        if pinned > ar.count || ar.count as usize > self.forward.len() {
            return Err(Fault::InvalidArena);
        }
        for id in 0..ar.count {
            work()?;
            self.forward[id as usize] = if id < pinned { 0 } else { NIL };
        }
        // alloc_raw deliberately bypasses interning. Such entries must not gain
        // new interning behavior as an accidental consequence of collection.
        let mut interned = 0u32;
        for (old, next) in ar.index_vals.iter().zip(&mut self.index) {
            work()?;
            if *old != NIL {
                if *old >= ar.count {
                    return Err(Fault::InvalidArena);
                }
                interned += 1;
            }
            *next = NIL;
        }
        if interned != ar.count {
            return Err(Fault::InvalidArena);
        }
        Ok(())
    }

    pub(crate) fn root(&mut self, id: Order, count: u32) -> Result<(), Fault> {
        if id >= count {
            return Err(Fault::InvalidArena);
        }
        self.forward[id as usize] = 0;
        Ok(())
    }

    pub(crate) fn plan<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        work: &mut impl FnMut() -> Result<(), Fault>,
    ) -> Result<u32, Fault> {
        for id in (0..ar.count).rev() {
            work()?;
            if let Data::Pair { left, right } = ar.get(id).ok_or(Fault::InvalidArena)?.inner {
                if left >= id || right >= id {
                    return Err(Fault::InvalidArena);
                }
                if self.forward[id as usize] != NIL {
                    self.forward[left as usize] = 0;
                    self.forward[right as usize] = 0;
                }
            }
        }
        let mut live = 0;
        for id in 0..ar.count {
            work()?;
            if self.forward[id as usize] != NIL {
                self.forward[id as usize] = live;
                live += 1;
            }
        }
        for old in 0..ar.count {
            work()?;
            let new = self.forward[old as usize];
            if new == NIL {
                continue;
            }
            let hash = ar.get(old).ok_or(Fault::InvalidArena)?.hash;
            let mut slot = (hash[0].as_u64() as u32) & ar.index_mask;
            // The load factor is bounded. Every actual probe still consumes
            // work and may cancel, before touching the arena's existing index.
            loop {
                work()?;
                if self.index[slot as usize] == NIL {
                    self.index[slot as usize] = new;
                    break;
                }
                slot = (slot + 1) & ar.index_mask;
            }
        }
        Ok(live)
    }

    pub(crate) fn remap(&self, id: Order) -> Order {
        // All live roots and pair children were checked during planning.
        let mapped = self.forward[id as usize];
        debug_assert_ne!(mapped, NIL);
        mapped
    }

    pub(crate) fn commit<const N: usize>(&self, ar: &mut Reduction<N>, live: u32) {
        for old in 0..ar.count {
            let new = self.forward[old as usize];
            if new == NIL {
                continue;
            }
            let mut entry = *ar.get(old).expect("planned initialized entry");
            if let Data::Pair { left, right } = entry.inner {
                entry.inner = Data::Pair {
                    left: self.remap(left),
                    right: self.remap(right),
                };
            }
            // Survivor order is preserved: new <= old, so no unread entry is
            // overwritten. Pinned entries have new == old and pinned children.
            ar.entries[new as usize].write(entry);
        }
        ar.count = live;
        for (slot, id) in self.index.iter().copied().enumerate() {
            ar.index_vals[slot] = id;
            ar.index_keys[slot] = if id == NIL {
                [Goldilocks::ZERO; 4]
            } else {
                ar.get(id).expect("planned live entry").hash
            };
        }
    }
}

impl<const N: usize> Reduction<N> {
    /// Restricted to the compacting evaluator, which restores the resident
    /// limit after each step, including every error-return path.
    pub(crate) fn set_step_allocation_limit(&mut self, limit: u32) {
        debug_assert!(limit >= self.count);
        self.allocation_limit = limit;
    }
}

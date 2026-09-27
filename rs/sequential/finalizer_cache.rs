//! Fixed memory reuse of successful pure finalizers within one arena borrow.
use super::{Error, Order, Outcome};
use alloc::vec::Vec;

const ENTRIES: usize = 1 << 16;

#[derive(Clone, Copy, Default)]
struct Entry {
    operands: u64,
    result: Order,
    // Cached tags are 3 and 5..=15; zero marks an empty slot.
    tag: u32,
}

/// Requested cache buffer bytes, excluding Vec metadata and allocator overhead.
pub const fn finalizer_cache_storage_bytes() -> usize {
    ENTRIES * core::mem::size_of::<Entry>()
}

pub(super) struct Cache<const ENABLED: bool> {
    entries: Vec<Entry>,
}

impl<const ENABLED: bool> Cache<ENABLED> {
    pub(super) fn new() -> Result<Self, Error> {
        let mut entries = Vec::new();
        if ENABLED {
            entries
                .try_reserve_exact(ENTRIES)
                .map_err(|_| Error::Allocation)?;
            entries.resize(ENTRIES, Entry::default());
        }
        Ok(Self { entries })
    }

    #[inline]
    pub(super) fn finish(
        &mut self,
        tag: u64,
        a: Order,
        b: Order,
        budget: u64,
        compute: impl FnOnce() -> Outcome,
    ) -> Outcome {
        if !ENABLED {
            return compute();
        }
        let operands = (u64::from(a) << 32) | u64::from(b);
        let mut mixed = operands ^ tag.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let slot = ((mixed ^ (mixed >> 31)) as usize) & (ENTRIES - 1);
        let entry = &mut self.entries[slot];
        if entry.tag == tag as u32 && entry.operands == operands {
            return Outcome::Ok(entry.result, budget);
        }
        let outcome = compute();
        if let Outcome::Ok(result, _) = outcome {
            *entry = Entry {
                operands,
                result,
                tag: tag as u32,
            };
        }
        outcome
    }
}

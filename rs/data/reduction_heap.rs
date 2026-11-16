use super::{Digest, Goldilocks, NIL, Order, Reduction};
use alloc::{alloc::alloc, boxed::Box};
use core::{alloc::Layout, fmt, ptr};

/// Failure before a heap arena becomes available to its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    InvalidCapacity,
    OutOfMemory,
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCapacity => "arena capacity must be a power of two fitting u32",
            Self::OutOfMemory => "arena allocation failed",
        })
    }
}

impl core::error::Error for AllocationError {}

impl<const N: usize> Reduction<N> {
    /// Initialize the fixed arena directly in the heap, using constant stack
    /// space. The global allocator may return null; no allocation error handler
    /// is invoked here. Logical limits and hash-consing match `new()`.
    pub fn try_new_boxed() -> Result<Box<Self>, AllocationError> {
        if !N.is_power_of_two() || N > u32::MAX as usize {
            return Err(AllocationError::InvalidCapacity);
        }
        // SAFETY: Self has nonzero size and Layout supplies its exact alignment.
        // initialize_allocation takes ownership of this allocation (or null).
        unsafe { Self::initialize_allocation(alloc(Layout::new::<Self>()).cast()) }
    }

    /// `raw` is null or a uniquely owned global allocation for Self. N has
    /// already passed the constructor's capacity check.
    unsafe fn initialize_allocation(raw: *mut Self) -> Result<Box<Self>, AllocationError> {
        if raw.is_null() {
            return Err(AllocationError::OutOfMemory);
        }
        // SAFETY: raw points to correctly sized/aligned uninitialized storage.
        // Field addresses are formed without references to uninitialized Self.
        // entries is MaybeUninit storage and count=0 keeps every slot unread.
        // All other fields are fully initialized before Box creates a reference.
        // No whole-array value or whole-arena stack temporary is constructed.
        unsafe {
            let keys = ptr::addr_of_mut!((*raw).index_keys).cast::<Digest>();
            let values = ptr::addr_of_mut!((*raw).index_vals).cast::<Order>();
            for i in 0..N {
                keys.add(i).write([Goldilocks::ZERO; 4]);
                values.add(i).write(NIL);
            }
            ptr::addr_of_mut!((*raw).count).write(0);
            ptr::addr_of_mut!((*raw).allocation_limit).write(((N / 4) * 3) as u32);
            ptr::addr_of_mut!((*raw).index_mask).write((N as u32) - 1);
            Ok(Box::from_raw(raw))
        }
    }
}

#[cfg(test)]
#[path = "reduction_heap_tests.rs"]
mod tests;

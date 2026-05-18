// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! execution trace — 16-column register table, one row per reduce() call
//!
//! r[0]  = pattern tag (0-17)
//! r[1]  = object NounId
//! r[2]  = formula NounId
//! r[3]  = result NounId (NIL on halt/error)
//! r[4..8] = pattern-specific operands (filled by pattern; zero otherwise)
//! r[8]  = budget before step
//! r[9]  = budget after step
//! r[10] = error kind on error rows; pattern-specific witness on success rows
//! r[11..16] = pattern-specific (bit decomp, step counter, etc.)

extern crate alloc;
use alloc::vec::Vec;

pub const COLS: usize = 16;

/// One row of the execution trace = zheng witness for one reduce() call
/// (or one step of a multi-row pattern).
///
/// The internal array is `pub(crate)` so only nox can construct/mutate rows;
/// external consumers (e.g., zheng) read via [`TraceRow::r`] or [`TraceRow::col`].
#[derive(Clone, Copy, Debug, Default)]
pub struct TraceRow {
    pub(crate) r: [u64; COLS],
}

impl TraceRow {
    /// Read-only view of all 16 columns.
    #[inline]
    pub fn r(&self) -> &[u64; COLS] {
        &self.r
    }

    /// Single-column read. Returns 0 for out-of-range indices.
    #[inline]
    pub fn col(&self, i: usize) -> u64 {
        if i < COLS { self.r[i] } else { 0 }
    }
}

pub trait Tracer {
    fn record(&mut self, row: TraceRow);
}

/// zero-overhead tracer for non-proof paths (CLI, tests)
pub struct NoTrace;

impl Tracer for NoTrace {
    #[inline(always)]
    fn record(&mut self, _: TraceRow) {}
}

/// accumulating tracer — collects all rows into a Vec.
///
/// Row count is bounded in practice by the budget passed to reduce():
/// each pattern consumes at least 1 budget unit before emitting a row,
/// so the trace never exceeds `budget` rows. Callers that pass untrusted
/// budgets should cap the budget before calling reduce().
#[derive(Default)]
pub struct VecTrace(pub Vec<TraceRow>);

impl Tracer for VecTrace {
    fn record(&mut self, row: TraceRow) {
        self.0.push(row);
    }
}

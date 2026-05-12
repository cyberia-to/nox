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
//! r[10..16] = reserved (error kind in r[10] on error rows)

extern crate alloc;
use alloc::vec::Vec;

pub const COLS: usize = 16;

#[derive(Clone, Debug, Default)]
pub struct TraceRow {
    pub r: [u64; COLS],
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

/// accumulating tracer — collects all rows into a Vec
#[derive(Default)]
pub struct VecTrace(pub Vec<TraceRow>);

impl Tracer for VecTrace {
    fn record(&mut self, row: TraceRow) {
        self.0.push(row);
    }
}

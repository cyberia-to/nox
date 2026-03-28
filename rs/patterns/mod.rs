// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! 16 deterministic patterns + 1 non-deterministic call + 1 deterministic look
//! each pattern = one file, one function, one responsibility
//!
//! four groups map to algebraic domains:
//!   structural (0-4): tree operations, algebra-independent
//!   field (5-10):     arithmetic over F, parameterized by field
//!   bitwise (11-14):  boolean over W, parameterized by word width
//!   hash (15):        identity via H, parameterized by hash function
//!   call (16):        non-deterministic witness injection (Layer 2)
//!   look (17):        deterministic external lookup

// structural — tree operations, algebra-independent
pub mod axis;       // 0: navigate
pub mod quote;      // 1: literal
pub mod compose;    // 2: apply
pub mod cons;       // 3: construct
pub mod branch;     // 4: choose

// field — arithmetic over F (dispatched by algebra)
pub mod add;        // 5: a + b
pub mod sub;        // 6: a - b
pub mod mul;        // 7: a * b
pub mod inv;        // 8: a^-1
pub mod eq;         // 9: a = b ? 0 : 1
pub mod lt;         // 10: a < b ? 0 : 1

// bitwise — boolean over W (dispatched by word width)
pub mod xor;        // 11: a ^ b
pub mod and;        // 12: a & b
pub mod not;        // 13: !a
pub mod shl;        // 14: a << n

// hash — identity via H (dispatched by hash function)
pub mod hash;       // 15: H(x)

// call — non-deterministic witness (Layer 2)
pub mod call;       // 16: prover injects, Layer 1 validates

// look — deterministic external lookup
pub mod look;       // 17: deterministic lookup from external source

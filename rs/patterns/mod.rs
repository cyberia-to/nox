// ---
// tags: nox, rust
// crystal-type: source
// crystal-domain: comp
// ---
//! 16 deterministic patterns + 1 non-deterministic hint
//! each pattern = one file, one function, one responsibility
//!
//! four groups map to algebraic domains:
//!   structural (0-4): tree operations, algebra-independent
//!   field (5-10):     arithmetic over F, parameterized by field
//!   bitwise (11-14):  boolean over W, parameterized by word width
//!   hash (15):        identity via H, parameterized by hash function
//!   hint (16):        non-deterministic witness injection (Layer 2)

// structural — tree operations
pub mod axis;       // 0: navigate
pub mod quote;      // 1: literal
pub mod compose;    // 2: apply
pub mod cons;       // 3: construct
pub mod branch;     // 4: choose

// field — arithmetic over Goldilocks
pub mod add;        // 5: a + b
pub mod sub;        // 6: a - b
pub mod mul;        // 7: a × b
pub mod inv;        // 8: a⁻¹ (cost 64)
pub mod eq;         // 9: a = b ? 0 : 1
pub mod lt;         // 10: a < b ? 0 : 1

// bitwise — 32-bit word operations
pub mod xor;        // 11: a ⊕ b
pub mod and;        // 12: a ∧ b
pub mod not;        // 13: ¬a
pub mod shl;        // 14: a ≪ n

// hash — hemera identity
pub mod hash;       // 15: H(x) (cost 300)

// hint — non-deterministic witness (Layer 2)
pub mod hint;       // 16: prover injects, Layer 1 validates

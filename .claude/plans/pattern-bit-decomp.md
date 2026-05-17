# Extend nox trace for bit-decomposition patterns and multi-row hash

## Status (2026-05-14)

**All five patterns RESOLVED.** zheng can now replace `trivial_ccs()` with
real constraint systems for patterns 10, 11, 12, 13, 14, and 15.

| Pattern | Opcode | Resolution                                                                                |
|---------|--------|-------------------------------------------------------------------------------------------|
| 10      | lt     | ✅ 64-row bit decomp (r10=a_k, r11=b_k per bit of 64-bit canonical repr)                  |
| 11      | xor    | ✅ 32-row bit decomp (r10=a_k, r11=b_k, r12=c_k per bit; row count = cost)                |
| 12      | and    | ✅ 32-row bit decomp (same shape as xor; per-row gadget c_k = a_k * b_k)                  |
| 13      | not    | ✅ 32-row unary decomp (r10=a_k, r11=0, r12=1-a_k)                                        |
| 14      | shl    | ✅ 32-row with cross-row shift binding (r10=a_k, r11=src_bit, r12=c_k, r13=src_idx)       |
| 15      | hash   | ✅ 25-row Poseidon2 trace via `hemera::StepSponge` (24 rounds + 1 squeeze)                |

The user chose **option (a) — one-bit-per-row** for patterns 10–14 (not the
2-limb packing originally recommended below). Trade-off: simpler constraints,
larger trace.

zheng can now wire up pattern_lt, pattern_xor, pattern_and, pattern_not,
pattern_shl, and pattern_hash following the register layouts in
`nox/specs/trace.md` §patterns 10–15.

---

## Historical context (pre-resolution)

`zheng/src/ccs/patterns.rs` had `trivial_ccs()` for the following patterns because the
nox trace did not emit the witnesses zheng needs:

| Pattern | Opcode | Reason currently trivial |
|---------|--------|--------------------------|
| 10      | lt     | range-decomp limbs not sufficient for per-bit constraint |
| 11      | xor    | bit witnesses are packed, not individually addressable |
| 12      | and    | same as xor |
| 14      | shl    | same as xor |
| 15      | hash   | nox emits single summary row; full round progression deferred |

Pattern 8 (inv) uses a `r5 * r3 = 1` result-check shortcut in zheng — valid because
inverse is uniquely determined. The same shortcut does NOT apply to hash (pattern 15),
which requires observing intermediate round state.

`nox/specs/trace.md` notes for pattern 15: "multi-row trace requires hemera step-by-step
API. current implementation emits a single summary row with output digest."

---

## Section 1 — Multi-row hash trace (pattern 15)

### What nox must emit

For each hash opcode execution, emit ~300 consecutive `TraceRow`s, all with `r0 = 15`:

| Row index | Semantics |
|-----------|-----------|
| 0         | Absorption: r3..r6 = rate input words, r7..r11 = initial capacity |
| 1–298     | Round progression: r3..r14 = Poseidon2 state after round `row - 1` |
| 299       | Squeeze: r3 = output digest (single Goldilocks element or first word) |

The row layout must match the layout documented in `nox/specs/trace.md` pattern 15.

### Hemera dependency

This requires `hemera` (or the hemera sub-crate) to expose a step-by-step sponge API:

```rust
// hemera/src/poseidon2/sponge.rs  (new or extended)
pub struct StepSponge { /* internal round state */ }

impl StepSponge {
    pub fn absorb(rate: &[Goldilocks]) -> Self;
    /// Advance one Poseidon2 round; returns current full state.
    pub fn step(&mut self) -> [Goldilocks; STATE_WIDTH];
    /// True after all rounds complete.
    pub fn done(&self) -> bool;
    pub fn squeeze(&self) -> Goldilocks;
}
```

nox calls `StepSponge::absorb()`, loops `step()` collecting one `TraceRow` per call,
then emits the squeeze row.

### What zheng implements after this lands

Replace `trivial_ccs(15)` with `pattern_hash()`:
- ~4 degree-2 sub-constraints per S-box (`sq1`, `sq2`, `cube`, `final`)
- `w` degree-1 MDS mix constraints per round
- Total ≈ 40 constraints per round × 298 rounds → multi-row CCS with `m ≈ 40`
- Reference: `zheng/specs/constraints.md` "hash pattern (15)" section

---

## Section 2 — Bit-decomposition witnesses (patterns 10–14)

### Recommended approach: 2-limb 32-bit packing (option b)

Rather than emitting one-bit-per-row (which balloons trace size), nox should agree on a
packing convention that zheng can verify with limb-level range checks.

**Proposed register layout for xor / and / shl (patterns 11, 12, 14):**

| Register | Meaning |
|----------|---------|
| r4       | operand A (64-bit input) |
| r5       | operand B (64-bit input, or shift amount for shl) |
| r6       | result |
| r7       | low 32 bits of A (A & 0xFFFF_FFFF) |
| r10      | high 32 bits of A (A >> 32) |
| r11      | low 32 bits of B |
| r12      | high 32 bits of B |

Consistency constraints zheng will verify:
- `r7 + r10 * 2^32 = r4` (A decomposition)
- `r11 + r12 * 2^32 = r5` (B decomposition)
- range check: `r7, r11 ∈ [0, 2^32)` via the existing sumcheck range infrastructure
- per-limb bitwise: `xor(r7, r11) = low32(r6)` and `xor(r10, r12) = high32(r6)` encoded as degree-2

**Proposed register layout for lt (pattern 10):**

Current `nox/specs/trace.md` already documents `r7=limb0`, `r10=limb1`, `r11=borrow`.
Add one constraint: `r7 + r10 * 2^32 + borrow_correction = r4 - r5` (limb consistency).
zheng will verify this as a degree-1 constraint plus two range checks on r7, r10.

### Files to modify in nox

- `rs/src/trace.rs` (or equivalent TraceRow emission site) — populate r7, r10–r12 for
  patterns 10–14 using the decomposition above
- `rs/src/patterns/hash.rs` (or equivalent) — switch to `StepSponge` step-by-step loop
- `nox/specs/trace.md` — update register layout table if r11/r12 assignments change

### Files to modify in zheng after nox update

- `src/ccs/patterns.rs` — implement:
  - `pattern_hash()` (replaces `trivial_ccs` for pattern 15)
  - `pattern_lt()` (pattern 10)
  - `pattern_xor()` (pattern 11)
  - `pattern_and()` (pattern 12)
  - `pattern_shl()` (pattern 14)

---

## Coordination notes

- The hemera `StepSponge` API is the critical path for pattern 15; target that first.
- Patterns 10–14 are independent of hemera; can be implemented in parallel once register
  layout is agreed.
- zheng does not need to be updated until nox emits the new trace rows; the `trivial_ccs`
  stubs remain sound (accept-everything) in the interim.
- Add an integration test in `nox/tests/` that exercises a hash opcode and asserts the
  emitted `TraceRow` count matches the expected ~300 rows.

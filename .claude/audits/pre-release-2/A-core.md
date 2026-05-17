# pre-release-2 audit — nox VM core (reduce, trace, call, noun, lib)

scope: rs/reduce.rs, rs/trace.rs, rs/call.rs, rs/noun/{order,inner,tag,hash,mod}.rs, rs/lib.rs

## blockers

- pass 1 — rs/reduce.rs:64 — depth-exceeded path returns `Error(Malformed)` with no `tracer.record` call, contradicting the file-level invariant "every reduce() call emits one TraceRow" (reduce.rs:8); STARK witness cannot reconstruct the halted step. fix: emit a synthetic error row before returning, or document the exception in trace spec and verifier.
- pass 1 — rs/reduce.rs:79 — budget-halt path returns `Halt(budget)` with no `tracer.record`; same trace-completeness violation as above. fix: record a row with `r[0]=tag, r[8]=budget_in, r[9]=budget_in` before returning Halt.
- pass 4 — rs/noun/hash.rs:45 — `Goldilocks::new(u64::from_le_bytes(buf))` ingests raw hemera bytes; if `Goldilocks::new` does not canonicalize values >= p (= 2^64-2^32+1), two distinct byte digests can represent the same field element, breaking hash-cons identity. fix: explicitly reduce mod p (or assert canonical form) when extracting digest words.

## major

- pass 2 — rs/reduce.rs:60 — `reduce_inner` recurses up to `MAX_DEPTH=1000`; on `#![no_std]` targets with constrained stacks, 1000 frames carrying a 128-byte `TraceRow` plus locals can blow the stack before the depth guard fires. fix: lower MAX_DEPTH, document required stack size, or convert to an explicit work-stack.
- pass 7 — rs/trace.rs:41 — `VecTrace::record` pushes unbounded onto a `Vec`; an attacker who supplies a large `budget` produces a proportional trace and can OOM the prover. fix: add a max-rows cap (parameter or const) and return a halt signal when exceeded.
- pass 5 — rs/reduce.rs:73 — pattern-tag selector is accepted from `Tag::Field` *or* `Tag::Word` atoms (via `atom_value` returning both); a Field atom with value `p-1` or any non-`Word` integer can drive dispatch, violating the documented contract that tags are word integers. fix: restrict to `Tag::Word` (and value < 18) in this site.
- pass 6 — rs/reduce.rs:117 — on `Outcome::Error`, the trace row writes `r[9]=0` (budget after) and `r[10]=kind`, losing the actual remaining budget at error time; verifier cannot bind the error to a budget state. fix: record `budget` (the post-cost value) in r[9] on error paths too.
- pass 1 — rs/noun/order.rs:35 — `MaybeUninit::uninit().assume_init()` on `[MaybeUninit<NounEntry>; N]` is the deprecated pattern (UB on older toolchains; Miri-flagged). fix: use `[const { MaybeUninit::uninit() }; N]` (Rust 1.79+) or `array::from_fn(|_| MaybeUninit::uninit())`.
- pass 11 — rs/noun/order.rs:37 — `Order::new` constructs `index_keys: [[Goldilocks::ZERO; 4]; N]` and `entries: [...; N]` on the stack before returning; with large `N` this stack-copies tens of KB or more. fix: box the storage or use placement-init / `Box::new_uninit_slice`.
- pass 11 — rs/noun/order.rs:113 — `index_lookup` worst case scans `0..N` linearly when the table is dense; combined with linear probing in `index_insert` this is O(N) per allocation in pathological fill. fix: track a load-factor cap (e.g. 0.75) and refuse to insert past it, or add quadratic probing.
- pass 10/8 — rs/call.rs:30 and rs/call.rs:54 — `LookProvider` is implemented twice with identical empty bodies (`NullLooks` and `NullCalls`); duplicate logic and two "null" types complicate the public API. fix: define one `Null` provider that implements both traits, or have `NullCalls` wrap `NullLooks`.
- pass 8 — rs/lib.rs:23 — `Noun` and `NounEntry` are re-exported publicly; callers outside the crate can pattern-match on `Noun::Cell { left, right }` with arbitrary `NounId` values from a foreign `Order`, breaking the hash-cons invariant. fix: keep `Noun` crate-private or expose it through a read-only accessor.

## minor

- pass 9 — rs/reduce.rs:29 — `MAX_DEPTH: u64 = 1000` has no comment explaining the choice; the limit is a hard correctness boundary. fix: add a one-line rationale (stack budget × frame size) next to the constant.
- pass 10 — rs/reduce.rs:90 — `is_multi_row = tag == TAG_INV || tag == TAG_HASH` duplicates knowledge that lives in pattern modules; adding a new multi-row pattern requires editing reduce.rs in lockstep. fix: have pattern modules declare row count, or move the check into a dispatcher.
- pass 9 — rs/reduce.rs:48 — `cost()` only special-cases 2 of 18 tags; the default `1` for cell/branch/compose is asymmetric with documented "field arithmetic + bitwise + hash" weighting. fix: either document why other patterns are free, or move the cost table to a named const array indexed by tag.
- pass 9 — rs/trace.rs:11 — header comment says `r[4..8] = pattern-specific operands` (4 cells) but then `r[10..16] = reserved` while r[10] is documented as error-kind; reserved-vs-error overlap is ambiguous. fix: split into `r[10] = error_kind`, `r[11..16] = reserved`.
- pass 10 — rs/trace.rs:20 — `COLS = 16` but only `r[0..11]` are used; 5 columns are dead today. fix: shrink COLS until those columns have semantics, or document them as future-reserved with intent.
- pass 5 — rs/reduce.rs:115 — `*r as u64` casts a `NounId` (u32) to u64 directly; works but co-mingles ids with NIL sentinel (`u32::MAX`) which becomes `0x00000000FFFFFFFF` rather than a real "null". fix: define a `to_trace_id` helper that explicitly maps None→0 or similar, removing ambiguity in trace reads.
- pass 10 — rs/noun/order.rs:43 — `>= N - 1` wastes one entry of capacity; the comment does not justify the off-by-one. fix: change to `>= N` if the reservation is unintended, or add a comment.
- pass 11 — rs/reduce.rs:53 — `reduce_inner` is generic over `T: Tracer`; with `NoTrace` and `VecTrace` callers, the whole reducer is duplicated. acceptable, but consider an `&mut dyn Tracer` variant for non-hot paths to cut binary size.
- pass 12 — rs/noun/mod.rs:24 — tests cover atom/cell/hash-cons happy paths but not: order full (`alloc_raw` returns None), index probing under collisions, `read_hash_noun` on malformed shape, `cell` with invalid NounId. fix: add edge-case tests for these.
- pass 12 — rs/reduce.rs — no tests in this file; pattern dispatch, depth guard, budget-halt, and multi-row branch have no direct coverage in scope. fix: add tests for each Outcome variant from `reduce()` directly.
- pass 9 — rs/noun/hash.rs:5 — comment claims "128-bit collision security" but digest is 4 × ~64 bits ≈ 256 bits truncated from 512; security bound and rationale belong next to `pub type Digest`. fix: clarify the truncation argument and cite the source spec.
- pass 5 — rs/trace.rs:24 — `TraceRow.r` is `pub`; callers can construct rows that violate the layout invariants. fix: make field crate-private with typed accessors, or wrap the array in a newtype.
- pass 8 — rs/reduce.rs:166-178 — `make_field` / `make_word` return `Outcome::Error(Unavailable)` on allocator exhaustion; semantically Unavailable is also used by call-provider misses, conflating two different failure modes. fix: introduce `ErrorKind::OrderFull` for allocator-exhaustion paths.
- pass 1 — rs/noun/order.rs:111 — `index_lookup` derives slot from `hash[0].as_u64() as u32`; deterministic, but if hemera ever returns a digest whose first limb is non-canonical (>= p) the cast still works yet equality compares the raw `Goldilocks` repr. fix: ensure `Digest` words are canonical after `extract_digest` (ties to hash.rs:45).

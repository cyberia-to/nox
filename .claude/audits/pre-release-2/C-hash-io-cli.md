# Pre-release audit C — hash, call, look, CLI

scope: `nox/rs/patterns/{hash,call,look}.rs`, `nox/cli/{main.rs,Cargo.toml}`
passes: 1–12 (full release tier)

## blocker

(none)

## major

- pass 6 — `nox/rs/patterns/call.rs:40` — check-formula errors are flattened into `CallRejected`, hiding legitimate `Malformed`/`TypeError`/`Unavailable` bugs in the check formula itself. Fix: only convert `Err(Outcome::Ok(...))` impossibility aside, propagate `Malformed`/`TypeError` unchanged; reserve `CallRejected` for the check_result ≠ 0 branch.

- pass 1, 11 — `nox/rs/patterns/hash.rs:1-6` — stub charges full `COST_HASH = 300` but emits only one row, so the trace under-witnesses the ≈300 Poseidon2 sub-steps; once the multi-row hemera API lands, the row count will change without a cost change, silently shifting prover/verifier balance. Fix: pin TODO to a tracked spec item and add a regression test that asserts `rows_emitted == COST_HASH` once enabled (or change cost model when the real impl lands).

- pass 7 — `nox/cli/main.rs:84-110` — `parse_expr` is unbounded recursion over user input; an attacker can craft `[[[[[…` (millions of `[`) and overflow even the 32 MB worker stack. Fix: convert to an explicit work-stack loop, or cap nesting depth (e.g. 4096) with a clear error.

## minor

- pass 9, 5 — `nox/rs/patterns/hash.rs:1-6` — header says "multi-row, cost 300" but the body emits a single summary row; the TODO is present but does not enumerate the gap (missing per-permutation rows, missing input-absorb rows, missing squeeze rows). Fix: expand the doc comment to list exactly what changes when the hemera step API is wired in, so the reader knows the *result* is correct but the *trace* is incomplete.

- pass 6 — `nox/rs/patterns/hash.rs:22-24` — `order.digest(input)` returning `None` is treated as `Unavailable`, but after a successful `evaluate` `input` is always a valid `NounId` and a digest always exists. Fix: use `ErrorKind::Malformed` or `debug_assert!` — `Unavailable` (= "prover doesn't know") is misleading semantics.

- pass 1 — `nox/rs/patterns/hash.rs:33` — row recorded inside the pattern AFTER `reduce_inner` already skipped its post-hoc recording (via `is_multi_row`). The mechanism is correct, but the coupling between `reduce.rs:90` and `hash.rs:33` is implicit. Fix: add a one-line comment in both files cross-referencing the contract, or extract a `MultiRowPattern` marker.

- pass 12 — `nox/rs/patterns/call.rs:60-119` — no test for the "good witness passes check" success path; both existing tests cover failure modes (null halt, bad witness). Fix: add a `call_accepts_zero_check` test where `provide()` returns `0` and check is `axis(1)`.

- pass 12 — `nox/rs/patterns/call.rs` — no test exercising deep recursion (call inside call) or witness containing a `[16 …]` formula. Fix: add a property test that nested call witnesses terminate via `MAX_DEPTH`.

- pass 1, 12 — `nox/rs/patterns/look.rs:38-95` — no test asserts that two identical `(ns, key)` queries return the same value across separate orders (the determinism property the file claims). Fix: add `look_deterministic_across_orders` mirroring `hash_deterministic`.

- pass 9 — `nox/rs/patterns/look.rs:29-35` — `row.r[6]` is only set on the `Some` branch; on `None` the field stays at its default (0), which collides with a legitimate zero lookup. Fix: write a sentinel (e.g. `NIL`) on the `None` branch, mirroring the convention used in `call.rs:53`.

- pass 7 — `nox/cli/main.rs:64` — tokenizer silently skips unknown characters (`{`, `<`, `,`, …). An adversarial or typo-laden input parses successfully as a different formula. Fix: return `Err(format!("unexpected character {:?}", ch))`.

- pass 7 — `nox/cli/main.rs:97-102` — single-element brackets `[x]` silently unwrap to `x`. This is a parser invention not in noun grammar; it makes `[[1 42]]` and `[1 42]` parse to the same formula. Fix: reject `[x]` as malformed, or document the shorthand explicitly.

- pass 7 — `nox/cli/main.rs:43-67` — tokenizer accepts unbounded numeric literals (`123456…` with arbitrary length) before `u64::parse` rejects them; allocation is proportional to untrusted input length. Fix: cap digit count at 20 (max u64 decimal width) and error early.

- pass 6 — `nox/cli/main.rs:131-141` — worker thread `.join().expect("nox thread panicked")` swallows the panic payload; on panic the user sees the harness expect message, not the original panic. Fix: print the panic payload via `std::panic::catch_unwind` or `join` error formatting before exiting with code 2.

- pass 9 — `nox/cli/main.rs:162` — short flag `-s` for `--object` is non-obvious (`-o` would be expected). Fix: rename to `-o` or add a rationale comment; `-s` currently looks like a typo.

- pass 9 — `nox/cli/main.rs:245-270` — help text omits the positional `<file>` form from the flag list (it appears only in the synopsis lines), and does not mention the stdin fallback as a flag-equivalent. Minor: add a `<file>` line in the flag section.

- pass 10 — `nox/cli/main.rs:131-141` — comment says "Order<65536> is ~6 MB" but the constant `ORDER_SIZE = 1 << 16` is the source. Fix: derive the comment from `std::mem::size_of::<Order<ORDER_SIZE>>()` in a const_assert, or move the rationale to a const doc comment so it cannot drift.

- pass 8 — `nox/cli/Cargo.toml:13-15` — `[[bin]] path = "main.rs"` at crate root is unusual (default would be `src/main.rs`); the layout is fine but should be documented as intentional. Fix: add a one-line comment in Cargo.toml or move to `src/main.rs` for convention.

- pass 11 — `nox/cli/main.rs:148` — default budget `1_000_000` is reasonable for interactive use but undocumented in terms of "what computations fit". Fix: add a comment with a worked example (e.g. "≈1M default-cost patterns, ≈3300 hash ops").

- pass 9 — `nox/cli/main.rs:209` — `formula_text.trim().to_string()` allocates a fresh string only to shadow; `let formula_text = formula_text.trim();` (borrow) suffices. Trivial.

## summary

no blockers. one major correctness concern (`call.rs` flattens check-formula errors into `CallRejected`), one performance/witness-balance risk (`hash.rs` cost vs row count when the multi-row impl lands), one adversarial-input issue (CLI parser recursion). The hash stub *result* is correct (digest is taken from hash-consed structural hash, not re-computed); only the trace is summary-row. `look.rs` is deterministic by construction — provider contract is the only variable.

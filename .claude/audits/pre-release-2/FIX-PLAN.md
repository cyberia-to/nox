# pre-release-2 audit — consolidated fix plan

four agents covered: A=core dispatch+trace+noun, B=patterns 0-14, C=hash/call/look+CLI, D=spec-vs-code.

## headline counts

| severity | A core | B pat 0-14 | C hash/io/cli | D spec | total |
|----------|--------|------------|---------------|--------|-------|
| blocker  | 3      | 0          | 0             | 14     | 17    |
| major    | 9      | 5          | 3             | 24     | 41    |
| minor    | 14     | ~20        | ~14           | ~17    | ~65   |

The 14 D-blockers collapse into 3 real defects once de-duped:

1. **hash cost = 200 vs 300** scattered across 5 spec files (code = 300 is canonical)
2. **look formula shape `[17 a]` vs `[17 [ns key]]`** scattered across 3 spec files (code = two-arg is canonical)
3. **trace register layout drift** — same root cause as B's 5 majors and A's 3 blockers, reported 9 ways

## the big strategic decision

**trace register layout drift is the dominant finding.** specs/trace.md describes the *target* STARK witness layout (Lens commitments, inverse hints, bit decompositions, BBG roots, opening proofs). Code writes the *interpreter's* output (operand values, result NounIds). They diverge for every pattern.

Three ways to resolve:

- **(A) spec wins** — code populates witness columns (inverse hints, bit decomps, Lens openings). Big lift; some witnesses (Lens openings) need cross-repo plumbing.
- **(B) code wins** — rewrite trace.md to describe what the interpreter actually emits. Defer "STARK witness layout" to a separate spec consumed by zheng.
- **(C) hybrid (recommended)** — cheap witness fills now (eq inverse, branch selector, axis register renames). Mark expensive ones (bit decomps, Lens openings, BBG roots) as "prover-side extensions added by zheng witness generator, not by nox interpreter."

I recommend **(C)**. It honors the spec where possible without blocking on lens/bbg/zheng plumbing.

## phase 1 — true blockers (no design decisions needed)

P1.1 spec hash cost 200 → 300 in 4 files
- specs/patterns/15-hash.md:9
- specs/patterns/README.md:76
- specs/reduction.md:176
- specs/jets/hash.md:17

P1.2 spec look two-arg form in 3 files
- specs/encoding.md:324
- specs/noun/README.md:44
- specs/reduction.md:156, 177

P1.3 spec error-kind list adds CallRejected=5
- specs/reduction.md:262

P1.4 reduce.rs: emit synthetic trace row on depth-exceeded and budget-halt paths (reduce.rs:64, :79)

P1.5 noun/hash.rs:45 — confirm Goldilocks::new canonicalizes (read nebu source); if not, add explicit reduction before equality checks. defer until verified.

P1.6 cli/main.rs: bound parse_expr recursion (cap 4096 nesting, return error past cap)

## phase 2 — hybrid trace fixes (recommended)

P2.1 eq.rs — store operand field values in r4/r5 (not NounIds); compute inverse hint in r7 = (r4-r5)^-1 when unequal
P2.2 branch.rs — r4=test_value (not NounId), r10=selector
P2.3 axis.rs — rename: r5=addr, r6=levels (drop "evaluation point" naming in spec to match)
P2.4 inv.rs — keep code (MSB-first), rewrite specs/trace.md §pattern 8 to MSB-first with `acc=v` at row 0
P2.5 hash.rs — expand TODO comment to enumerate what's missing (per-permutation rows, absorb, squeeze); pin to a tracked spec item
P2.6 call.rs:40 — propagate Malformed/TypeError/Unavailable from check formula; only ≠0 result becomes CallRejected
P2.7 look.rs:30 — write r6=NIL on None branch (matches call.rs convention)
P2.8 specs/trace.md — add note that bit-decomp witnesses (r7/r10 for lt/xor/and/not/shl) and Lens-opening columns (axis/look) are populated by zheng witness generator, not by the interpreter

## phase 3 — quality cleanups (skip-if-tight)

P3.1 lib.rs — remove public re-export of Noun/NounEntry (A pass 8)
P3.2 call.rs — collapse NullLooks + NullCalls into one Null (A:20)
P3.3 reduce.rs — name COST table by tag, document MAX_DEPTH choice (A:25, :27)
P3.4 trace.rs — restrict TraceRow.r visibility (A:36)
P3.5 noun/order.rs — replace deprecated MaybeUninit::uninit().assume_init() (A:17)
P3.6 noun/order.rs — load-factor cap on index_insert (A:19)
P3.7 cli/main.rs — reject unknown tokenizer chars, reject `[x]` single-element brackets, cap numeric literal length (C minor cluster)
P3.8 specs/vm.md — mark non-existent regime repos (kuro/jali/trop/genies) as planned
P3.9 specs/jets/ — top-level status flag "implementation deferred post-genesis"
P3.10 cli/main.rs — rename short flag `-s` → `-o` for `--object`
P3.11 noun/hash.rs:19 — align tag placement with spec (capacity[14] only) or document dual encoding

## phase 4 — test coverage gaps

P4.1 inv: property test inv(k) * k = 1 for k ∈ {1, p-1, random}
P4.2 add/sub/mul: field-boundary tests (p-1, wraparound)
P4.3 eq: cell equality tests (digest-comparison path)
P4.4 axis: cell + addr=0 hash introspection test
P4.5 shl: tests for vn ∈ {33, 63, 64, u32::MAX}
P4.6 call: success-path test (good witness, check returns 0)
P4.7 look: determinism-across-orders test
P4.8 reduce: direct tests for each Outcome variant + depth-exceeded + budget-halt + multi-row dispatch
P4.9 noun: edge tests for order-full, malformed read_hash_noun, invalid NounId

## not in this audit (deferred)

- jets/ implementation (Layer 3)
- wire encoding round-trip (encoding.md → rs)
- memoization layer
- parallel reduction
- atom-Hash variant decision (tag 0x02) — currently dead in code; should be removed from spec or implemented

## recommendation

execute phases in order. P1 is mechanical (~30 min). P2 needs design buy-in for inv ordering and hybrid trace layout decision. P3+P4 can be parallelized across multiple sessions.

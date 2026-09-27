# Bounded compaction correctness review

Scope: nox base `13b4c2ee71b3f3e205ddc9ad30d1476b86a7ccc5` plus the
archived `implementation.patch`. Exact source digests and toolchain are in
`run-inputs.json`. The preceding liveness census remains in the isolated
`../nox-live-probe/audit/compiler-liveness/` tree with its exact measured patch.

Independent review by the root's pipeline reviewer checked dispatch reads
against roots, survivor ordering, preservation of metadata, cumulative
admission and the planning/commit boundary. It identified axis-zero Enter as
an additional allocating step. The fix adds exact tag/address decoding,
requires available guest gas and respects frame admission before considering
collection. The final tests cover this path and a completely pinned arena
whose finalizer succeeds entirely through existing interned values.

Verification questions and answers:

1. Can any caller-held Order move? Every resident entry at function entry is
   pinned, including unrelated caller data. Survivor order keeps that entire
   prefix at identical indices; children of a pinned pair precede it and are
   pinned too. A later call pins the prior returned result and all resident
   entries. The repeated-loop test checks exact entry metadata at every pinned
   index and reuses its result in a later call.
2. Can an old continuation or cache retain a stale Order? The direct phase
   matrix covers every action/phase. Dispatch reads only the remapped fields;
   unused trace registers and old a/b formulas are not read as data by NoTrace.
   Both live-but-moved and collected cache results are tested after clearing.
3. Can cancellation or quota rejection expose an invalid index? All fallible
   work and placement probing happen in scratch before arena mutation. The
   exact linear commit charge is reserved before copying any entries or index
   slots. Tests reject work just below the commit allowance, cancel during
   planning, cancel at the pre-commit poll, and cancel immediately after commit;
   each then re-interns every remaining entry and checks pinned data.
4. Can cumulative admission be bypassed by partial output or reinterning?
   Each evaluator step receives the lesser of free resident capacity and
   remaining total allocation allowance. The original resident allowance is
   restored before propagating every step result. Actual arena-count deltas
   include partial hash output. Every collection preserves
   total_allocations = resident_nodes + reclaimed_nodes for the current run.
   Recreated values count again; intern hits succeed even at zero fresh budget.
5. Can rebuilding silently change raw-entry behavior or formula costs?
   Collection checks private index coverage and rejects unindexed alloc_raw
   entries. Every pair is checked for children before parent. Surviving entries
   retain original digests and cached bounds; only pair Orders move. Tests
   compare surviving bounds to the larger baseline arena by digest.
6. Can the collector change guest execution? No guest child is retried and no
   host compiler stage or jet is introduced. Both profiles share dispatch and
   finalizers. Differential tests compare pure successes/errors, successful gas,
   propagated Halt budgets, frame errors and phase behavior. The saved full
   compiler discovery test adds a large independent artifact-level comparison.

7. Can a semantic Unavailable be mislabeled as allocation exhaustion? The
   initial classification compared only Unavailable and equality with the total
   limit. A regression using missing external axis/hash/cons/eq operands showed
   this could claim admission rejection before any allocator call. The final
   predicate excludes missing operands and requires positive allocation
   headroom for cumulative-limit classification. Tests cover spare and full
   resident arenas with zero fresh allowance and zero collection-work quota.
   `invalid-operands-before.log` preserves the reproduced pre-fix failure.

Collection work counts planning iterations/probes and the reserved bounded
commit, separately from evaluator checkpoints and guest reductions. A cancelled
pre-commit poll may leave a charged commit reservation unused; the contract
states this explicitly. Scratch is allocated fallibly from resident/physical
capacity before evaluation; no dynamic graph-walk stack is allocated.

Limits: this policy changes the interpretation of node accounting only when
explicitly selected. It is unsuitable for legacy trace Orders. The workload
measurement covers discovery, and does not certify a C2 compiler or full
self-hosting. Index compaction has a bounded non-preemptible commit interval,
so the host must retain its post-execution publication deadline check.

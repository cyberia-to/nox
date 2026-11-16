# Bounded NoTrace compaction

Status: measured component implementation. Full compiler C2 generation and
self-hosting acceptance remain separate gates. This change adds an explicit
compacting cached API while keeping legacy lifetime allocation and traced
execution paths unchanged. The contract is
[sequential-compaction.md](../../specs/sequential-compaction.md).

## Why this profile exists

The preserved [liveness census](../compiler-liveness/README.md) observed
177,780 reachable entries at a 25,165,824-entry allocation failure, including
159,037 pinned preloaded entries and the failed operation's retry operands.
Its source patch, exact commands, raw baseline failure, census/work counters,
and source/binary hashes are copied intact into `audit/compiler-liveness/`.
That observation established lifetime garbage pressure at one frontier; it
provided no whole-run maximum live-set bound.

## Saved compiler discovery comparison

The ignored diagnostic executes the original saved packed prefix artifact and
JOB1 at stage two. It performs no host source-language compiler stage. The
input digests, nox base revision, measured local patch, binary digest, toolchain
and pinned hemera/strata revisions are recorded in
[run-inputs.json](run-inputs.json). Commands are in [commands.sh](commands.sh).

| measurement | lifetime baseline | compacting diagnostic |
|---|---:|---:|
| guest reduction budget | 1,000,000,000 | 1,000,000,000 |
| successful charged reductions | 684,646,281 | 684,646,281 |
| remaining reductions | 315,353,719 | 315,353,719 |
| evaluator checkpoints | 566,412,397 | 566,412,397 |
| peak logical frames | 3,259 | 3,259 |
| physical arena entries | 33,554,432 | 4,194,304 |
| allowed resident entries | 25,165,824 | 3,145,728 |
| allocated entries / total fresh allocation events | 15,458,181 | 16,334,747 |
| final resident entries | 15,458,181 | 1,413,834 |
| peak resident entries | 15,458,181 | 3,145,728 |
| reclaimed entries | 0 | 14,920,913 |
| collection passes | 0 | 5 |
| charged collection work | 0 | 121,748,975 |
| collection checkpoints | 0 | 20,697 |
| collector scratch bytes | 0 | 29,360,128 |

Both results contain `[2, 94, 0, 0, 0]`, and their canonical DAG files have the
same SHA-256:
`2a4c546218675e02fa40aa5de1bd20a04c7d8bc3e7defe9345320bed7ab9041d`.
The test asserts exact successful remaining budget, peak frames and evaluator
checkpoints. Collector checkpoint calls are reported separately. Total callbacks
were 566,433,094 in the compacting run.

The compacting profile admitted 64,000,000 total allocation events and
2,000,000,000 collection-work units, with 65,536 maximum frames and a cooperative
1,200-second deadline. All 159,037 preloaded entries remained pinned. Recreated
values consume cumulative allowance again, explaining why total allocation
events exceed the baseline's lifetime unique-entry count. Neither number should
be substituted for the other in admission or reports.

The baseline evaluation took 117,763 milliseconds and the compacting diagnostic
117,847,288 microseconds. These are separate observations with shared-machine
load; no throughput conclusion is drawn. Raw receipts are
[baseline-discovery.json](baseline-discovery.json),
[discovery.json](discovery.json), [discovery.dag](discovery.dag) and
[discovery.log](discovery.log).

## Measured and final source

The diagnostic ran on nox base
`13b4c2ee71b3f3e205ddc9ad30d1476b86a7ccc5`, branch
`feat/0.4-bounded-compaction`, plus [implementation.patch](implementation.patch)
with SHA-256 `9539ce616c81b26a63dcc1d72513fee58a8f91792f3ca2a2fc34ebd689d30aba`.
After that binary started, root review requested narrowing the headroom frame
admission guard to Enter actions explicitly. A later public-API regression also corrected cumulative-limit reporting for
missing external operands which fail before any allocation attempt. These
production-only deltas are preserved in [post-measurement-runtime.patch](post-measurement-runtime.patch).
They change no runtime API or resource limits. The observed peak of 3,259 frames
is below the 65,536 limit, so both guard versions take the same collection
branches throughout this diagnostic. Valid decoded inputs and VM-constructed
values also satisfy the added operand checks throughout this workload. The measured time and binary identity
remain attributed to the measured patch, rather than to the later source.
The final full-source digests are in `final-files.json`.

## Verification and limits

Final default tests: 220 passed, one full compiler diagnostic ignored by default.
Final parallel-feature tests: 219 passed, the same diagnostic ignored. The full
saved discovery diagnostic passed separately. Strict all-target Clippy for the
cyber-nox package passed. Logs are retained here. The unrelated CLI warning
previously observed by the liveness probe remains outside this change's scope.

[review.md](review.md) records the independent review and correctness questions.
The seventeen new regular tests cover precise phase/action roots, repeated
collection, pinned caller Orders and later reuse, sharing and cost bounds,
weak cache invalidation, exact successful budgets and propagated Halt values,
frame/service errors, partial allocation admission, full pinned intern hits,
axis-zero allocation, missing external operand failures, work rejection and cancellation before/after commit.
The max-frame Return regression verifies a finalizer can collect while consuming
an existing frame. No tests or fixtures increase historical default caps.

The collector pins the entire pre-entry arena, so a large externally held live
set may still exhaust resident storage. It rejects unindexed raw entries before
index rebuilding. Its commit is a bounded non-preemptible copying interval;
host publication must retain the final deadline check. Collection work is
charged separately from guest reductions and includes the pre-admitted commit
reservation if a pre-mutation cancellation poll rejects that commit. Traces
continue using their original Order-lifetime contract.

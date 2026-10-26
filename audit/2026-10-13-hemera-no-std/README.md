# Hemera no_std feature boundary

Fresh CODE APPROVE with no blockers; portable Nu replay PASS. This remains a
source decision with red/unverified gates, not a clean release or registry-delivery
result. [Final review disposition](review/disposition.md) preserves the full
verdict and qualifies its unproven performance explanation.
The two reviewed changes opt Nox out of Hemera defaults and forward Nox's explicit
`std` feature to `hemera/std`, then add the named real thumb code-generation gate.
All 48 owning Rust files, dependency paths, versions, optionality and source locks
remain unchanged. Parent: `0b1407a4b855e4961ede07e1b7ea199cb8018c30`.

Kadek still resolves unchanged registry nox 0.2.0. This upstream source fix does
not close Kadek's dependency delivery or its separate alloc/math work. No version,
tag, publication or release action is included. Default-feature opt-out changes
transitive feature inheritance: Hemera `Hash::to_hex` and `std::io::Read` for
`OutputReader` require explicit std enablement. Nox does not call them; the named
consumer manifests independently enable Hemera defaults. This is a manifest/source
inventory, not successful full consumer or product compilation.

## Reproduce the evidence

From the repository root:

```sh
nu audit/2026-10-13-hemera-no-std/verify.nu audit/2026-10-13-hemera-no-std
```

Requirements: Nushell, Git (only `hash-object`), tar/gzip, mktemp and readlink.
No Cargo, network, sibling checkout, Ruby or Python is needed for this replay.
Historical Ruby capture/build/verifier programs remain raw evidence inside the
archive. Do not execute them as a substitute for this portable verifier.

The immutable base is [PLAN v2](../2026-10-13-hemera-no-std-plan-v2/README.md).
Its archive/index/verifier digests are pinned in [capsule.json](capsule.json).
This capsule adds 213 content-addressed objects and borrows 303 from that base:
526494 compressed bytes, 921 logical payloads. Both directories must travel
together. [index.json](index.json) maps every logical name to exact bytes, SHA-256
and local/base storage. It includes all 904 CODE support files, full raw streams,
source archives, the first preparation failures and independent replay receipts.
There are no loose Cargo manifests for Cargo's Git-package scanner to discover.

The verifier checks the full base, reconstructs the exact 220069-byte CODE input
`aebaed2fa934763ec3f401c0cb1414b8e6aeae7a93ef9d4c73d3d6d315377e46`, verifies every
object, all 103 command receipts/206 streams, the 536-entry source bridge with
exactly two changed files, 52 normalized-package entries and 48 unchanged Rust
files, actual compiler-root separation, and raw Criterion threshold arithmetic.
It reconstructs historical inputs and validates recorded outcomes; it does not
rerun compilation or benchmarks. The original source-freeze verifier separately
checked the live worktree diff at capture. This portable replay checks that exact
patch digest and source bodies, without pretending the historical live check is
performed again.

## Measured gates and remaining boundaries

All commands, cwd, environment, timestamps, runner hashes and streams live at
logical `code/support/execution/records/<record>/`; [execution notes](../../.claude/tasks/hemera-no-std/execution.md)
name the exact source epochs. Parent gates precede the production edit. Candidate
source is the two-file patch
`387f9ccee4480bbfeb957ee9a2cbb7d959ac808db5635a3f296ee51b001f3899`.

| Gate | Actual result and raw record |
| --- | --- |
| Real thumb code generation | Parent sibling38 and normalized-package68 fail `error[E0463]: can't find crate for std` at Hemera; candidate72/96 pass. |
| Workspace and feature tests | Candidate74/75/76: 161 passing each; parallel77 and brakedown78: 164 each. |
| Actual H264 controls | Four controls pass in each sibling debug84/release85 and normalized-package debug98/release99 run. These are bounded controls, not the complete codec-domain suite. |
| Packaging and host | Package95 contains 52 files; package97 passes161 library tests. CLI86/87 and wgpu88 pass; CLI87 has zero tests. Nightly82 passes with zero doctests. |
| Feature inheritance | Candidate90 has no Hemera/std; explicit std91 and variants92–94 do. |
| Current-parent API | Explicit JSON comparison101: 229 passing checks, 31 skips. Exact artifacts63/100 came from distinct actual compiler targets. Dependency feature inheritance is outside this tool's model. |
| Required exact last-tag API | Unmet: exact v0.2.0 locked build64 fails `error: cannot update the lock file`. [Named follow-up](../../.claude/tasks/exact-tag-api-baseline/research.md). Automatic diagnostics60/61 have shared-target ambiguity and do not replace this gate. |
| fmt | Candidate79 remains red; stdout equals parent58 after only source-prefix normalization. |
| Clippy | Candidate80 retains parent57 source error: `error: this expression creates a reference which is immediately dereferenced by the compiler`, `cli/main.rs:247:52`. Final Cargo wrapper differs, so full stream equality is not claimed. |
| Strict docs | Candidate81 retains parent47: `error: unresolved link to 11`. Exact backticks and source context are in raw stderr. |
| honeycrisp | Candidate89 retains parent56 missing `acpu::field::ntt_forward` and `multilinear_eval`. |
| Trident/product | Parent59 stops at unavailable cached clap4.5.57. Ring1 product integration remains unverified; static dependency-version findings are not substituted for the observed first error. |
| Performance | Red/unverified. Initial49/83 breaches 5 of 9 predeclared thresholds; the single planned repeat102/103 breaches 6 of 9. Cause is unestablished. |

For repeat102/103, median changes are particle +43.12%, tree +89.64%, pattern hash
+5.28%, hash chain +46.90%, inverse +21.08%, look +15.82%; add and multiply reverse
direction from the initial pair. Raw samples/estimates and both comparisons remain
in logical `code/support/execution/benchmark/`. Root paused heavy Cargo work for
the repeat; this agent compiled nothing concurrently. Other system load/frequency
was not measured. Equal 3028144-byte executable sizes but unequal file hashes do
not prove code equality, noise, a regression cause or a performance improvement.
No extra repetitions or source tuning were hidden.

## Review lineage and preparation failures

PLAN v1 was rejected. Its full input, raw verdict and disposition remain in the
[original capsule](../2026-10-13-hemera-no-std-plan/README.md) and immutable PLAN v2
support. Fresh Opus PLAN v2 approved the exact input at
`2026-10-13T02:23:59.183394Z`; [raw approval](review/plan-v2/stdout.json),
[invocation](review/plan-v2/invocation.json), [completion](review/plan-v2/completion.json).
CODE v1 was independently reconstructed by root before a fresh vendor review.
The capsule's frozen pending-review statement is historical; the later actual
CODE APPROVE is bound by the separate [outer disposition](review/disposition.md).

Records19–37 are failed Homebrew `+toolchain` dispatch, not source compilation;
record46 is mixed Clippy-driver/toolchain failure, superseded by pinned-driver57.
The first freeze helper compared UTF-8 and binary Ruby strings incorrectly even
though their digests matched; the corrected binary comparison passed without a
source change. The first execution verifier omitted `nu --stdin`; its preserved
body and failure are evidence, not a gate failure. All corrected actual outcomes
are bound to their own receipts; none overwrites an earlier epoch.

Portable verifier preparation failures and the successful later replay are
retained separately in [verification preparation](verification/preparation.md).

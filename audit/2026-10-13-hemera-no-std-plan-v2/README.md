# Revised Hemera no_std PLAN evidence

Parent: `0b1407a4b855e4961ede07e1b7ea199cb8018c30`. This frozen capsule
preserves the revised [PLAN](../../.claude/tasks/hemera-no-std/plan.md), proposed
manifest/named-gate delta, all source and parent measurements supplied for review.
At capsule freeze the revised review was pending and production unchanged. Actual
approval and subsequent implementation belong to the separate
[execution](../../.claude/tasks/hemera-no-std/execution.md) record.

Readable input: 227176 bytes; SHA-256
`1f2028db98726cea04732856de00b2b058501b930112df0811f09c83c1ac60d3`.
There are 62 sections, 378 indexed support files, 67 command records/134 streams,
six exact source archives/1287 Git entries including eight Trident symlinks, all
18 Hemera cfg/lint sites, nine benchmark estimate/sample sets and both package-H264
modes. The original REQUEST CHANGES input/raw verdict and the independently verified
v1 capsule remain unchanged inside support. Preparation failures are preserved.
Numbers and outcomes are bound to their exact commands/source in the reconstructed
input, not to a current checkout.

```sh
nu audit/2026-10-13-hemera-no-std-plan-v2/verify.nu audit/2026-10-13-hemera-no-std-plan-v2
```

The verifier uses Nu, tar/gzip, mktemp, readlink and Git's read-only blob hashing.
It needs no network, Cargo builds, live Git checkout, Python or Ruby runtime. It
checks archive/index/object identity, reconstructs the entire readable input,
checks source body/mode/link targets, exhaustive cfg inventory, captured streams,
actual paired-parent controls and retained reds. Reverse manifest bodies are hashed;
the included root replay additionally checked their exact original Git identities
and repeated source scans. This portable verifier does not claim to rerun Cargo or
perform a fresh live reverse-repository audit.

Inherited format, CLI lint, strict docs, honeycrisp symbols, Trident locked cache
prerequisite and exact-tag locked API errors are explicit. Automatic API runs 60/61
are not isolation proof; only the separately emitted parent JSON identities are
claimed here. These are ring-1 decision materials, not a clean release candidate.
Kadek still resolves unchanged registry nox 0.2.0.

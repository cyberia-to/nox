# Published-archive API companion

Base `102b9e3ee35921f2289db6e87ce3182cb06946b3`, branch
`fix/published-api-baseline`, reused owned `no-std-dependencies/nox` checkout.
The previous wrapper context is archived in
`audit/2026-10-17-published-api-baseline/task-honeycrisp-backend.json`.
Hemera `1c385e54`, Strata `56aedb2d` and Lens `8b26cef6` symlink targets/revisions
remain unchanged. Builds use an external task cache; no production inputs change.

## Authority and scope

The required exact locked v0.2.0 tag gate remains unchanged and red. Its existing
research explicitly withholds policy-change authority. This task creates an
executable companion diagnostic and a concrete policy proposal for owner review;
no successful companion result can replace, waive or mark that gate green.
No dependency/version edits, source substitutions, tags or publication actions.
Scope is Nu/Rust tooling, fixture inputs, this proposal and audit evidence.

## Evidence design

1. Fetch the official `cyber-nox 0.2.0` index record and crate archive into the
   task cache; verify archive SHA256
   `7c807c1b53e66cea79e8e130ca938450029af3f096984a2839fee6443a171b3f`.
   Record exact acquisition argv/status and the archive's `.cargo_vcs_info.json`.
   Compare owning Rust bodies with the peeled immutable v0.2.0 source, explaining
   Cargo normalization and the published lock separately. Preserve the tag and
   original locked-tag failure; do not regenerate its lock.
2. Produce baseline and current rustdoc JSON in distinct source/target roots
   with the same pinned compiler and flags. The baseline is the unmodified
   published archive. The current side is an exact normalized package of the
   committed owning source, retaining registry dependency identity; verify its
   Rust bodies equal the owner. The current occupied version is diagnostic only.
   Record all package manifests, locks, source/artifact hashes and actual commands.
3. Build default (empty crate defaults) and explicit-std API surfaces separately using
   `cargo rustdoc --locked --offline` and explicit JSON output. Use installed
   pinned rustup 1.95.0 with the documented rustdoc JSON mechanism, and installed
   `cargo-semver-checks 0.51.0` with explicit `--baseline-rustdoc` and
   `--current-rustdoc` paths. Predeclare patch-compatible comparison; report the
   actual verdict, counts and omissions. Never allow automatic shared-target
   baseline discovery. Any tool/format failure stays a failure.
4. Independently inspect resolved feature graphs: old default Hemera/std,
   repaired default without it, and explicit std forwarding in both surfaces.
   Name transitive std-only APIs and actual dependency versions. The public API
   tool does not adjudicate this compatibility change; neither does a default
   API pass determine Nox's release version. Optional backends remain outside
   this companion's acceptance claim, with their existing delivery debts named.
5. Write a portable verifier for acquisition/source/artifact/raw-stream hashes,
   exact commands/tool identity and separate roots. Keep bulky rebuildable
   artifacts in the external cache; retain the minimal reproducible source,
   artifact identity and raw verdict evidence under the owning audit.

## Proposed policy, not adopted

For a future owner decision, distinguish source-tag reproducibility from the
public compatibility of bytes actually distributed by crates.io. A published
archive comparison could become an additional required compatibility gate when
its checksum, VCS correspondence, dependency domain and feature surfaces are
verified. The exact locked-tag failure remains a separate unresolved release
gate until the owner explicitly decides its policy; no retrospective repair of
the tag is proposed. State this separation in the machine-readable outcome.

Acceptance for this task: genuine old/current artifacts, executable comparison,
independent feature-inheritance findings, preserved historical red and a reviewable
policy proposal. Root reviews this plan before tool implementation; final CODE
review and source-bound evidence precede an ordinary tooling commit.

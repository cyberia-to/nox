# Post-merge scope: C/D retains the frozen source revision

The C/D source revision remains
`64d091e5ad3f277cce77b88ae0de6eea4c49b4df`. Preparation PR
[#39](https://github.com/cyberia-to/nox/pull/39) merged at
`262de214521c47d18cd539040280f22e4b1fd8a4`, after PR
[#36](https://github.com/cyberia-to/nox/pull/36) merged at
`7a52650ab50c3804a9620172667dbdb94df28f1c`. The preparation merge therefore
contains additional production changes absent from the fixed C/D exports.
This note is outside the freeze and changes no experimental input.

[current-head-bridge.json](current-head-bridge.json) records the read-only
commands, both Git blob identities and SHA256 values for every changed file,
and the unchanged manifest, lockfile and benchmark identities. The complete
source diff is retained as [current-head-delta.patch.gz](current-head-delta.patch.gz).
The PR description and merge identity are captured in
[current-head-pr36.json](current-head-pr36.json).

`git diff --name-only 64d091e5..262de214 -- . ':!audit' ':!.claude'` identifies
the following 11 Rust paths. The same path set and source changes come from
PR #36; `git diff --exit-code 7a52650..262de214 -- cli rs Cargo.toml Cargo.lock`
passes. Full object IDs are recorded above and in the JSON receipt.

| Paths | Inspected source change |
| --- | --- |
| `cli/main.rs` | Iterative result printer and its tests. |
| `rs/data/cost.rs`, `rs/data/hash.rs`, `rs/data/reduction.rs` | Appended `#[cfg(test)]` modules; existing production bodies unchanged. |
| `rs/encode.rs` | Canonical Goldilocks limb checks for pair particles in `particle_of` and `decode`, plus tests. |
| `rs/jets/backends/honeycrisp.rs` | Reject out-of-range NTT/poly-evaluation shift parameters. |
| `rs/jets/fri_fold.rs`, `rs/jets/ntt.rs`, `rs/jets/poly_eval.rs` | Reject out-of-range shift parameters, plus tests. |
| `rs/jets/merkle_verify.rs` | Check decoded path length against charged depth before verification, plus tests. |
| `rs/reduce.rs` | Widen `COSTS` visibility to `pub(crate)` for the synchronization test; table values unchanged. |

`git diff --exit-code 64d091e5..262de214 -- Cargo.toml Cargo.lock
rs/Cargo.toml rs/benches/reduction.rs` passes. That equality is insufficient to
extend the performance scope: the unchanged `jet/merkle_verify` case directly
calls `merkle_verify_jet`, whose production body now has the additional path
length check. Static inspection shows the case constructs matching depth and
path length; neither its net timing effect nor the resulting binary identity
has been measured. No whole-program equivalence is claimed for the other cases.

The PR #36 description reports successful Clippy, workspace and release tests,
documentation, thumb and formatting commands. Those are attributed PR claims:
PR #36 adds no raw gate receipts, and this bridge reruns none of those commands.
They do not supply performance evidence or extend the earlier exact source-byte
bridge from the accepted hygiene inputs to the new master revision.

All 256 files in the frozen manifest still match their recorded hashes. The
manifest SHA256 remains
`b0dd79a69c3c9eec478984976fe65cafbc3f89cef1316674c7e39baca6425bee`.
The historical A/B exports, C/D exports, identical-A path control, fixed binaries,
tools and finite schedule remain unchanged. No builds, compiler runs or timing
were performed for this note. Root separately reports successful portable replay
after the preparation merge; that verifies retained artifacts, not current-master
compiler or performance equivalence.

Any eventual C/D result applies to the fixed `64d091e5` source revision and its
recorded sibling pins. It cannot close a performance gate for `262de214` or a
later release. The original 5/9 and 6/9 red results remain as recorded in
`original-replay.json`; timing remains unmeasured and requires the separate quiet
window authorization. A current-master performance claim would require separately
authorized, revision-specific evidence; this note introduces no replacement runs.

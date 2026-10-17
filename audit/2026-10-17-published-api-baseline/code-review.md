# Independent review

CODE APPROVE — `/root`, 2026-10-17, context `bac9dcc0bfa4444e6942b2dae273a434b444454e`.
Reviewed the eight tooling inputs at frozen digest
`18dcb9fc24f8fd80ee6d9d7ab24258b0957b5fae411a3038f8ed6343b9f898c1`,
the accepted plan, README, exact artifacts and raw verdicts. Production inputs
remain `686e179925c4956364e5e9876d51e4d1abe268ff163ea3df334080ad81546b3a`.

The first implementation checked Rust bodies against HEAD but proved only
manifest stability, so `--allow-dirty` could admit an uncommitted manifest.
The final runner rejects compiler inventory/content differences before packaging,
including ignored root Cargo configuration. Seven isolated controls exercise
that guard; earlier evidence remains separately identified. No unresolved code
finding remains in this diagnostic's scope.

Root ran `nu audit/2026-10-17-published-api-baseline/verify.nu REPO FRESH_CACHE`
with exit 0: [result](root-verification.json). This independently checks the
official archive checksum, VCS/source correspondence, current committed inputs,
tool identities, separate artifacts, exact commands/raw streams and retained
locked-tag failure without executing historical checkout paths.

Root then ran [root-replay.nu](root-replay.nu) against that extracted capsule.
Both explicit `cargo-semver-checks … --release-type patch` comparisons again
report 229 pass / 31 skip; [receipts](root-replay/result.json) bind argv,
raw output, tool source and capsule. Final mandatory thumb receipt and source
manifest were independently rehashed; command succeeded at unchanged closure
`a697240541739fc2cb0a79c451b91667a9d6865a78a78470efcc7f77fee89d37`.

Approval covers the companion tooling/evidence only. The compiled consumer's
loss of inherited Hemera/std remains a compatibility decision, and the exact
locked-tag gate remains red. This approval neither changes that policy nor
chooses a release version, merges a bump or authorizes publication.

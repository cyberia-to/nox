# Published-archive API companion — 2026-10-17

The published-package comparisons pass for Nox default and explicit `std`.
The required exact locked-tag gate remains red/unresolved. A separate compiled
consumer proves loss of inherited Hemera/std APIs under current defaults.
This diagnostic makes the release decision reviewable; it grants no release
approval and adopts no replacement gate policy.

Owning source: `102b9e3ee35921f2289db6e87ce3182cb06946b3`, unchanged compiler
inputs at plan commit `bac9dcc0bfa4444e6942b2dae273a434b444454e`.
Owner Rust/manifest inventory SHA256:
`686e179925c4956364e5e9876d51e4d1abe268ff163ea3df334080ad81546b3a`.
No production, dependency, feature, version, lock or immutable-tag edits.

## Measured results

All final commands, environment, stdout/stderr and hashes are linked below.
Rust 1.95.0 and cargo-semver-checks 0.51.0 are identified by executable hashes.

| Input / command | Actual result |
|---|---|
| [Official archive acquisition and current `cargo package --locked --offline --allow-dirty`](pair-4/identities.json) | Published SHA256 `7c807c1b53e66cea79e8e130ca938450029af3f096984a2839fee6443a171b3f`; VCS and all Rust bodies match peeled tag `6ab6e1a303355a49aaef6e069bffe5bfb3efa2a2`. Current package matches committed owning inputs. `--allow-dirty` admits audit-only work; full compiler inventory/content equality to HEAD is checked first. |
| [Explicit rustdoc JSON comparisons](api-3/result.json), `cargo-semver-checks check-release --current-rustdoc … --baseline-rustdoc … --release-type patch` | Default: 229 pass / 31 skip. Explicit std: 229 pass / 31 skip. Two separate source roots, four distinct targets/artifacts; both locks select registry Hemera 0.3.1 and strata-nebu 0.1.1. |
| [Consumer](std-surface.rs.txt), [four actual checks](features-3/result.json), `cargo check --locked --offline [--features std]` | Published Nox default/std pass; current default fails with E0277 (`OutputReader: std::io::Read`) and E0599 (`Hash::to_hex`); current std passes. The consumer requests Hemera 0.3.1 with defaults disabled, so Nox alone supplies its std feature. |
| [Committed-input guard cases](guards-1/result.json), `nu …/guards.nu REPO FRESH_CACHE` | Clean and audit-only inputs accepted; changed root/owning manifests, lock, Rust body and ignored Cargo config refused before packaging. Seven cases. |
| [Historical exact locked-tag gate](required-tag-history/receipt.json) | Actual exit 101 retained byte-for-byte with provenance. Immutable tag's workspace lock still names Nox 0.1.2 while its manifest names 0.2.0. Its lock was never regenerated. |
| [Final required thumb gate](final-thumb.json), `cargo build -p cyber-nox --no-default-features --target thumbv7em-none-eabi --locked --offline` | Pass on the retained [complete owning/sibling source closure](final-sources.json), with warnings denied. |

Rustdoc JSON extraction deliberately uses the same empty warning flags on both
sides; published-source rustdoc warnings remain in the raw streams. This is an
artifact extraction command, not a strict docs quality gate. Current compiler
inputs equal the previously tested hygiene source; broad source tests were not
rerun for this audit-only unit. The mandatory thumb build was rerun at handoff.

## Proposed owner decision — open

Keep source-tag reproducibility and distributed-package compatibility separately
named. Adopt the archive comparison only as an additional compatibility gate,
requiring official archive checksum, VCS/source correspondence, exact dependency
domain and separately generated feature surfaces. The existing locked-tag gate
stays red until the owner explicitly decides its policy. No repaired tag or
successful companion run retroactively changes that result.

Decide the inherited-feature compatibility/version policy separately. Cargo
[unifies dependency features](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification);
consumers relying on that inherited std surface demonstrably stop compiling.
Explicit `nox/std` or direct `hemera/std` is the concrete consumer migration.
The measured public-API checker pass alone does not establish a patch release
as compatible. Optional Honeycrisp delivery, the existing performance/ring
baseline debts and final bump/publication gates remain outside this comparison.
Only the owner merges a bump, tags or publishes.

## Reproduce and inspect

From this owning repository, with pinned tools already installed and fresh
cache directories (substitute paths; no historical task root is executed):

```nu
nu audit/2026-10-17-published-api-baseline/verify.nu REPO FRESH_VERIFY_CACHE
nu audit/2026-10-17-published-api-baseline/prepare.nu REPO PAIR_CACHE replay-pair
nu audit/2026-10-17-published-api-baseline/compare.nu REPO replay-pair PAIR_CACHE API_CACHE replay-api
nu audit/2026-10-17-published-api-baseline/features.nu REPO replay-pair PAIR_CACHE FEATURE_CACHE replay-features
```

[The capsule](evidence.tar.gz) retains the exact two package archives and four
compared rustdoc artifacts; [its index](bundle.json) binds their bytes. Verification
uses repository/cache arguments, compares current complete compiler inventory,
extracts the capsule into a fresh cache and checks exact commands, raw streams,
artifact hashes, installed tools, source ancestry and unresolved gate authority.
Historical absolute paths remain evidence only. The broader original audit
continues to own the original tag-failure execution, rather than a claimed rerun.

Earlier successful `pair-2`, `api-1/2`, `features-1/2` epochs remain distinct from
final `pair-4`, `api-3`, `features-3`. The final bridge adds portable package-cache
arguments and the committed-input guard. `pair-1`, `compare-parse-1`, `pair-3`
retain actual isolated reproductions and exact source of preparatory Nu errors;
none reached package/API comparison. Prior capsule is under `retention-1/`.
The [accepted plan](../../.claude/plans/published-api-baseline.md) and
[prior wrapper context](task-honeycrisp-backend.json) preserve task ownership.

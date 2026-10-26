# Feature reachability at the pinned sources

Scope: nox `0b1407a4b855e4961ede07e1b7ea199cb8018c30`, Hemera
`d5f0a09539360934fb32db9b2e15a91995607db6`. Generated evidence is
`feature-audit/{hemera-cfg-sites,reverse-feature-audit,reverse-api-scans}.json`;
`feature-audit.rb` scans the complete committed `.rs`/`.toml` inventory, retaining
full source hashes and exact line text. This is source/manifest analysis, not a
claim to have built every consumer.

## Hemera

All three `std` cfg sites are: `rs/src/lib.rs:54` (`extern crate std`),
`sponge.rs:37` (`Hash::to_hex`), and `sponge.rs:462` (`std::io::Read` for
`OutputReader`). The latter two are real dependency API changes when its std
feature is no longer unified on by another edge. `Hash`'s Display and
`OutputReader::fill` remain unconditional. Tests' explicit std imports are under
`cfg(test)` and do not affect nox's dependency build. The two async module guards
in `lib.rs:78,80` are controlled by `async`, whose feature forwards `std`; nox
never enables async. Serde is likewise unchanged and not enabled by nox.

All fourteen `hemera_portable` cfg/cfg_attr sites are architecture decisions,
independent of std: `arith.rs:61,95,131,141,178,191,220,229,261` and
`backend.rs:85,101,133,146,178`. The first arith site only permits dead code on
aarch64. The others select register-only aarch64 arithmetic or portable Rust;
`Cargo.toml:22` additionally registers the cfg name for linting. There is no
runtime CPU-feature detection in the committed Rust source. Disabling std does
not select a different arithmetic backend. `profile.rs` and `kernel.rs` have no
std/portable feature guards. Public-data and constant-time kernels remain
selected by the existing hash entry points, independent of std.

Nox uses `tree::{hash_leaf,hash_node}`, `Hash::{from_bytes,as_bytes}` and
`StepSponge`/Hemera Goldilocks. The complete `git grep` evidence covers owning
Rust, including benchmarks and tests. None calls either std-only API. Its
public digest/particle values are NeBu arrays or byte arrays, not a re-export of
Hemera's std-only methods. Architecture arithmetic is reachable through hashing,
so the named reduction benchmark still runs on both parents/candidates; source
independence alone is not used as a performance gate waiver.

## Named reverse consumers

Exact manifests and all `hemera::`, `to_hex`, `OutputReader` Rust matches are
retained. The source revisions below are historical exact inputs, not dirty
working trees. Each direct Hemera path resolves to the same sibling location
as nox's edge in its stated layout.

| Repository and revision | Manifest-level Hemera/std inheritance after the proposal |
| --- | --- |
| soft3 `44824e6cf64f5b0b1d7443eea412001e74c5f32e` | `conformance/rs/Cargo.toml` independently enables Hemera defaults; nox also requests brakedown → std. |
| zheng `5733dfdcc15cbc22fce9f5877b37515d005b8c44` | Both `rs` and `cli` independently enable Hemera defaults and request nox/brakedown → std. |
| mudra `186c0f22871e72ac5d27396d893998865fc604d4` | Root manifest independently enables Hemera defaults; optional nox/proof activation does not remove that edge. |
| bbg `441e3c9ddf85c4bee36b40ed7ae8805b1481bc27` | `rs` independently enables Hemera defaults, regardless of its default nox edge. |
| Trident `ec986d8edb926da51a70c23c97782d65a239c144` | Root manifest independently enables Hemera defaults. Standalone `silicon` declares no direct Hemera dependency and has no `hemera::`/`OutputReader`/Hemera `to_hex` use; its `compile_to_hexagon` names are unrelated. Both manifests request unavailable path nox ^0.3.0 while this nox is 0.2.0. |
| nox CLI at the owning parent | No direct Hemera dependency or affected API use; its actual build/test is included in owning workspace and separate CLI controls. |

These manifests establish the local additive-feature facts; they do not prove
complete graphs, locks, target variants or product execution. Actual build/test
of soft3, zheng, mudra and bbg, and Trident product build/test remain unverified.
The isolated locked Trident metadata probe (record 59) first fails on missing
cached clap 4.5.57; the nox version mismatch is separately source-proven, not its
observed first error. Kelvin drift/rung specification checks remain unverified.

## Ring and delivery

The pinned registry identifies nox as layer 1's machine, with Hemera and Strata
`part_of = nox`, and Trident as layer 2's direct dependent. The reverse Cargo
edges above extend the blast radius beyond this repository. This is ring 1,
with `decision` disposition because rung/product gates are not all green. The
session's existing owner instruction authorizes ordinary source work/review/
merge; it does not authorize publication, a bump, tag or release. The expected
patch result of an owning default-surface API comparison is not a declaration
that the full graph's release floor is patch.

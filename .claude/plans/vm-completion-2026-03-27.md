---
name: nox VM completion
description: all remaining work to make nox a complete, frozen VM — spec gaps, impl gaps, consistency fixes
status: draft
date: 2026-03-27
---

# nox VM completion

everything needed to freeze nox as a complete VM specification + implementation.

## 1. spec gaps — contracts not yet written

### 1.1 TCO (tail call optimization)

compose chains of unbounded depth must not overflow the stack.
specify: which patterns get TCO (compose, branch tail position),
how the order memory model supports it (trampoline vs loop),
interaction with budget metering.

files: specs/patterns/compose.md, specs/reduction.md

### 1.2 continuation (pause/resume)

large orders need cooperative scheduling. specify: how to
snapshot order state (NounId + program counter + budget remaining),
resume from snapshot, interaction with proof (partial trace).

files: new spec — specs/continuation.md

### 1.3 witness separation

hint does injection, but formal public/private/output classification
is not specified. zheng needs to know what to prove and what to hide.
specify: which nouns are public input, which are private witness,
which are output. how this maps to trace columns.

files: specs/hint.md or specs/witness.md, update specs/trace.md

### 1.4 trace format

trace.md exists but the exact register layout (r0-r15 values per step)
is not detailed enough for implementation. specify: column names,
column types, what each register holds at each pattern, AIR constraints
per pattern (updated for 1-row-per-reduce from cost-model-redesign).

files: specs/trace.md

### 1.5 streaming / BAO

large nouns verified chunk-by-chunk. specify: chunk size (4KB per
particle spec), incremental hashing, partial verification protocol.

files: new spec — specs/streaming.md or extend specs/encoding.md

### 1.6 lazy evaluation scope

branch is lazy (only evaluates chosen arm). compose and cons are eager.
this is implicit — make it explicit. specify: which patterns evaluate
which sub-expressions, and when.

files: specs/patterns.md or per-pattern specs

## 2. impl gaps — spec exists, code doesn't

### 2.1 axis = O(1) Lens opening

spec says axis navigates noun polynomial via Lens opening.
code does O(depth) tree walk. implementation requires lens crate
dependency.

priority: HIGH — spec violation.

files: rs/patterns/axis.rs, Cargo.toml (lens dep)

### 2.2 encoding (binary serialization)

specs/encoding.md exists. no implementation. needed for wire format,
content addressing, cross-host determinism.

files: new — rs/encoding.rs

### 2.3 jets (formula hash recognition)

specs/jets.md + specs/jets/*.md exist. no implementation.
dispatch mechanism: H(formula) lookup in jet registry.

files: new — rs/jets/ module

### 2.4 trace generation

reduce() must emit trace rows. currently returns only result.
each reduce() call → one row in trace table.

files: rs/reduce.rs, new — rs/trace.rs

### 2.5 memo (axon lookup)

H(formula, object) = axon. before reduce(), check if result
already known. requires bbg integration for cybergraph cache.

files: rs/reduce.rs, bbg dependency

## 3. spec consistency — stale/wrong information

### 3.1 verifier constraint numbers

nox specs say 400K/50K (WHIR+Merkle). zheng says 8K/825 (Brakedown).
zheng is canonical.

files: specs/jets/recursion.md (was nebu.md), specs/trace.md

### 3.2 WHIR → Brakedown

all references to WHIR as PCS → Brakedown. WHIR was the old design.

files: grep all specs/ for "WHIR"

### 3.3 merkle_verify, fri_fold jets

these jets exist for WHIR/FRI verifier. with Brakedown they are
not needed for the verifier. decide: keep (useful elsewhere?),
remove, or mark as non-verifier jets.

files: specs/jets/recursion.md

### 3.4 jet file renames

| current | new | reason |
|---------|-----|--------|
| nebu.md | recursion.md | domain, not crate name |
| kuro.md | binary-tower.md | domain |
| jali.md | polynomial-ring.md | domain |
| trop.md | tropical-semiring.md | domain |
| genies.md | isogeny-curves.md | domain |

state.md, decider.md, hash.md — keep as-is.

### 3.5 jet registry model

jets.md says "hardcoded protocol constant".
decided: cybergraph registry with genesis entries.
genesis = MUST. post-genesis = deferred.

files: specs/jets.md

### 3.6 subject → object in specs

code already renamed. specs may still say "subject" in places.
grep and fix.

files: all specs/*.md

### 3.7 NounRef → NounId, NounInner → Noun, Arena → Order in specs

code already renamed. specs must match.

files: all specs/*.md

## 4. restructure (from approved plan restructure-nox-2026-03-26.md)

### 4.1 roadmap/ → migrate + delete

per approved plan. transformer-jets → docs/explanation/.
implementation-audit → .claude/audits/. rest deleted.

### 4.2 update indexes

specs/README.md, docs/explanation/README.md links.

## 5. cli

### 5.1 art banner

hemera-style ASCII art for `nox` CLI. show on bare `nox` invocation
and `nox --help`.

### 5.2 cli in separate directory

cli/ separate from rs/ (lib). Cargo.toml workspace with two members.

## priority order

1. spec consistency (3.*) — fix what's wrong before building more
2. restructure (4.*) — clean layout
3. naming in specs (3.6, 3.7) — code-spec alignment
4. spec gaps (1.*) — write missing contracts
5. impl gaps (2.*) — build what specs define
6. cli (5.*) — developer experience

---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-03-20
diffusion: 0.00010722364868599256
springs: 0.00007019991600688145
heat: 0.00003419142694206788
focus: 0.00008151008453347325
gravity: 0
density: 0
---
# implementation audit — nox foundation completeness for Rs

can someone implement a working nox VM from the spec, using Rs (~/git/rs) as the implementation language?

nox has 7 canonical spec files (vm, nouns, patterns, reduction, jets, trace, encoding), 3 proposals, 18 explanation docs, and stub implementation (35 lines across 8 modules). Rs has ~6,800 LOC implementation (core + macros + compiler), complete spec, and enforces determinism by construction.

## spec completeness — what's ready

| spec file | status | implementation-ready? |
|-----------|--------|----------------------|
| nouns.md | canonical v0.2 | **yes** — atom\|cell, type tags, axis addressing, coercion rules all defined |
| patterns.md | canonical v0.2 | **yes** — 17 patterns with semantics, types, costs, test vectors |
| reduction.md | canonical v0.2 | **yes** — ask()/reduce() interface, budget metering, confluence proof, error types |
| encoding.md | canonical v0.2 | **yes** — 3 sizes (8/32/64), content addressing, canonical invariants |
| trace.md | canonical v0.2 | **yes** — 16 registers, per-pattern row layout, constraint structure |
| vm.md | canonical v0.2 | **mostly** — hemera params updated, see gaps below |
| jets.md | canonical v0.3 | **API yes, jet formulas no** — mechanism defined, actual formulas missing |

## spec gaps — what blocks implementation

### G1: noun memory representation (critical)

the spec defines `noun = atom(F) | cell(noun, noun)` but never specifies the in-memory layout. for Rs (no heap, no Box), the only option is arena + indices:

```rust
type NounRef = u32;  // arena index
enum NounInner {
    Atom(GoldilocksField),
    Cell(NounRef, NounRef),
}
```

the spec should define:
- maximum noun depth (needed for arena sizing)
- maximum noun count per computation (arena capacity)
- whether structural sharing is allowed (DAG vs tree)

**action**: specify max depth, max count, DAG vs tree policy in nox/reference/nouns.md

### G2: jet formula trees not provided (critical)

the spec says "formula hashes computed at build time from pure Layer 1 definitions" but the actual Layer 1 formulas for the 5 verifier jets are never written. without these, an implementor cannot:
- compute the correct formula hashes
- build a compatible jet registry
- interop with other nox implementations

what's needed: the nox program (as a noun) for each of: hash, poly_eval, merkle_verify, fri_fold, ntt. these are the jet's pure Layer 1 definitions.

**action**: write pure Layer 1 nox programs for all 5 jets in nox/roadmap/recursive-jets.md

### G3: hint callback interface (high)

pattern 16 (hint) says "prover injects witness" but doesn't define:
- the callback signature
- sync vs async injection
- what happens if the prover provides no hint (error? halt?)
- whether hints are validated before use or only after

for Rs this would be a trait:

```rust
trait HintProvider {
    fn provide_hint(&self, object: &Noun, tag: u64) -> Option<Noun>;
}
```

**action**: define HintProvider callback signature in nox/reference/reduction.md

### G4: test vector consistency (medium)

the cost-model-redesign plan (marked "implemented") identified TV1 and TV3 as off-by-1 (96→97). need to verify these corrections actually landed in patterns.md.

**action**: verify TV1, TV3 corrections in nox/reference/patterns.md

### G5: hemera version (resolved)

resolved: nox now targets hemera (24 rounds, 32-byte output, x⁻¹ S-box). all spec files updated in commit 2f8572f.

### G6: memoization spec (low)

the spec mentions `(H(obj), H(formula)) → H(result)` caching but doesn't specify: eviction policy, max size, persistence format, required vs optional. for Rs: BoundedMap<(Particle, Particle), Particle, N> with compile-time N.

**action**: state whether memoization affects correctness or only performance in nox/reference/reduction.md

### G7: multi-computation batch API (low)

the ask() interface handles one computation. how does the VM process a batch (e.g., a block of signals)? is budget shared across computations or per-computation?

**action**: define multi-computation interface in nox/reference/vm.md

## Rs readiness — can Rs implement nox?

| nox requirement | Rs capability | gap? |
|-----------------|---------------|------|
| Goldilocks arithmetic | nebu (path dep) | **gap**: Rs core doesn't depend on nebu |
| Hemera hash | placeholder Particle | **gap**: need real hemera integration |
| Recursive nouns | Arena<T, N> | **works**: arena + indices, no Box needed |
| Determinism | #[deterministic] | **perfect fit**: Rs enforces what nox requires |
| No heap | Rs edition restrictions | **perfect fit**: nox VM should be heap-free |
| Canonical encoding | #[derive(Addressed)] | **works**: Particle = nox identity |
| Content addressing | Particle type (32 bytes) | **needs update**: Rs Particle is 64 bytes, hemera outputs 32 |
| Focus metering | u64 or nebu field element | **works**: simple decrement counter |
| Jet dispatch | match on formula hash | **works**: no dyn dispatch needed, static table |
| Trace generation | BoundedVec<TraceRow, N> | **gap**: trace size unbounded, needs streaming |
| Bounded async | #[bounded_async(dur)] | **useful**: for timeout on ask() |

### key Rs gaps

1. **nebu dependency missing**: Rs core needs `nebu` for Goldilocks field elements
2. **hemera dependency missing**: real Hemera hash needed, not placeholder
3. **Particle size**: Rs Particle is 64 bytes, hemera outputs 32 bytes — need to update Rs
4. **trace buffer sizing**: nox traces can be arbitrarily large (2^20+ rows). streaming trace writer avoids bounded buffer
5. **no recursive types without arena**: solved by arena + NounRef pattern

## Rs ↔ nox alignment

| Rs primitive | nox concept | alignment |
|-------------|-------------|-----------|
| Particle (→ 32 bytes) | Noun identity H(noun) | **match after update** |
| Address (32 bytes) | Noun identity H(noun) | **exact match** with hemera |
| #[derive(Addressed)] | Canonical encoding | **exact match** (content-addressed) |
| #[step] | Focus step boundary | **natural fit** (reset state per computation) |
| BoundedVec<T, N> | Axis path, formula body | **works** for bounded sequences |
| BoundedMap<K,V,N> | Jet registry, memo cache | **works** for fixed-size tables |
| Arena<T, N> | Noun allocation | **works** for tree construction |
| FixedPoint<T, D> | — | **not used** by nox (field elements, not fixed-point) |

note: Rs Address (32 bytes) now matches hemera output (32 bytes). Address = Particle = H(noun). the three types converge.

## nox crate layout in Rs

```
nox/src/
├── lib.rs          mod exports
├── noun.rs         NounRef, NounInner, NounArena<N>
├── reduce.rs       reduce(object, formula, budget) → Result
├── hint.rs         HintProvider trait
├── jet.rs          JetRegistry (const table of formula_hash → fn)
├── trace.rs        TraceRow, TraceWriter (streaming)
├── encode.rs       Noun → [u8; 8|32|64], [u8] → Noun
├── memo.rs         MemoCache<N> (BoundedMap-based)
└── budget.rs       Budget type (nebu field element or u64)
```

dependencies: nebu (field arithmetic), hemera (hash)

## action items

### critical (block implementation)

| # | gap | action | where |
|---|-----|--------|-------|
| G1 | noun memory layout | specify max depth, max count, DAG vs tree | nouns.md |
| G2 | jet formula trees | write pure Layer 1 programs for 5 jets | recursive-jets.md |
| G3 | hint interface | define HintProvider callback signature | reduction.md |

### high (should fix before implementation)

| # | gap | action | where |
|---|-----|--------|-------|
| G4 | test vectors | verify TV1, TV3 corrections landed | patterns.md |
| R3 | Rs Particle size | update from 64 to 32 bytes for hemera | rs/core/src/core_types.rs |

### medium (can defer)

| # | gap | action | where |
|---|-----|--------|-------|
| G6 | memoization | state required vs optional, eviction policy | reduction.md |
| G7 | batch API | multi-computation interface | vm.md |
| R1 | nebu dependency | add nebu to Rs core Cargo.toml | rs/core/Cargo.toml |
| R2 | hemera dependency | replace Particle placeholder with real hemera | rs/core/src/core_types.rs |
| R4 | trace writer | design streaming TraceWriter | nox implementation |

## bottom line

the nox spec is 85-90% implementation-ready. three critical gaps (G1-G3) are solvable in 1-2 sessions each. Rs is a natural fit — its no-heap, deterministic, content-addressed primitives align with nox's requirements. hemera output size (32 bytes) aligns with Rs Address type. the main design decision is arena-based noun representation.

estimated implementation effort: 6-8 sessions for a working nox VM with all 17 patterns, 5 jets, streaming trace, and content-addressed memoization.
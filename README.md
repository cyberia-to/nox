# nox

proof-native virtual machine over the Goldilocks field. sixteen deterministic reduction patterns, one non-deterministic hint, five jets. every execution produces a STARK proof as a byproduct — running a program and proving it ran correctly are the same act.

## lineage

```
combinatory logic (1924)   S, K combinators            pure abstraction
  → lambda calculus (1936) Church's untyped lambda      computable functions
  → Nock (2016)            natural numbers + decrement  deterministic VM for Urbit
  → nox (2026)             field elements + inverse     proof-native VM for cyber
```

## architecture

```
reduce(subject, formula, focus) → result
```

everything is a noun — a binary tree of Goldilocks field elements. programs are nouns. data is nouns. the result is a noun.

```
Layer 1: 16 deterministic patterns (Turing-complete + field arithmetic + bitwise + hash)
Layer 2: hint (non-deterministic witness injection, verified by Layer 1 constraints)
Layer 3: 5 jets (hash, poly_eval, merkle_verify, fri_fold, ntt) — optimization only
```

## repo structure

```
reference/              canonical specifications (source of truth)
├── vm.md               overview, field (aurum), hash (hemera), dependencies, index
├── nouns.md            data model: atom, cell, type tags, coercion, structural hash
├── patterns.md         all 17 patterns: Layer 1 (0-15) + Layer 2 hint (16)
├── reduction.md        reduction semantics, confluence, parallelism, memoization
├── jets.md             Layer 3: pure equivalents, hardware mapping, verifier costs
├── trace.md            execution trace layout, AIR constraints, polynomial encoding
├── encoding.md         canonical noun serialization, wire format, content addressing
└── props/              design proposals (draft → accepted → implemented)
docs/
└── explanation/
    └── nox.md          conceptual overview — lineage, design philosophy, architecture
src/                    Rust implementation
├── lib.rs              module declarations
├── noun.rs             Atom, Cell, Noun (binary tree of field elements)
├── reduce.rs           Layer 1 reduction engine (16 patterns)
├── hint.rs             Layer 2 non-deterministic witness injection
├── jet.rs              Layer 3 jets (hash, poly_eval, merkle_verify, fri_fold, ntt)
├── trace.rs            execution trace recording (becomes the STARK witness)
├── encode.rs           canonical noun serialization (deterministic wire format)
├── memo.rs             content-addressed computation cache
└── focus.rs            resource metering (attention budget)
```

## companion repos

| repo | path | role |
|------|------|------|
| aurum | ~/git/aurum/ | Goldilocks field arithmetic |
| hemera | ~/git/hemera/ | Hemera hash (Poseidon2-Goldilocks) |
| zheng | ~/git/zheng/ | proof system (SuperSpartan + WHIR) |
| trident | ~/git/trident/ | high-level language, compiles to nox |
| mudra | ~/git/mudra/ | crypto primitives (KEM, dCTIDH, TFHE, threshold) |
| bbg | ~/git/bbg/ | authenticated state (Big Badass Graph) |

## license

Cyber License: Don't trust. Don't fear. Don't beg.

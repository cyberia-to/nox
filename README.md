# nox

proof-native virtual machine over the Goldilocks field. sixteen deterministic reduction patterns, one non-deterministic hint, five jets. every execution produces a STARK proof as a byproduct — running a program and proving it ran correctly are the same act.

## lineage

```
combinatory logic (1924)   S, K combinators            pure abstraction
  → lambda calculus (1936) Church's untyped lambda      computable functions
  → Nock (2016)            natural numbers + decrement  deterministic VM for Urbit
  → nox (2026)             field elements + inverse     proof-native VM for cyber
```

nox replaces Nock's natural numbers with Goldilocks field elements and decrement with field inverse. the consequence: every computation is native to STARK arithmetization. the execution trace IS the algebraic constraint system.

## architecture

```
reduce(subject, formula, focus) → result
```

everything is a noun — a binary tree of field elements. programs are nouns. data is nouns. the result is a noun.

three layers:

- Layer 1 — sixteen deterministic patterns (the ground truth of computation)
- Layer 2 — one non-deterministic pattern: hint (the origin of privacy and search)
- Layer 3 — five jets: hash, poly_eval, merkle_verify, fri_fold, ntt (optimization without changing meaning)

## repo structure

```
reference/         canonical specifications (vm, trace, encoding)
src/               Rust implementation
```

## specifications

- `reference/vm.md` — field, nouns, all 16+1 patterns, jets, cost table, test vectors
- `reference/trace.md` — execution trace layout, AIR constraints, polynomial encoding
- `reference/encoding.md` — canonical noun serialization, wire format, content addressing

## license

Cyber License: Don't trust. Don't fear. Don't beg.

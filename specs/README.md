# nox reference

canonical specification of the nox virtual machine. this is the source of truth — when code and reference disagree, fix reference first, then propagate to code.

## specifications

| page | scope | status |
|------|-------|--------|
| vm.md | overview, field, hash, algebra polymorphism, dependencies | canonical |
| noun/ | data model: atom, cell, type tags, order, hash, polynomial representation | canonical |
| patterns/ | all 17 patterns: structural (0-4), field (5-10), bitwise (11-14), hash (15), hint (16) | canonical |
| reduction.md | reduce(), ask(), budget, confluence, memoization, proof-carrying, signal assembly | canonical |
| jets.md | Layer 3: genesis jet registry, semantic contract, hardware mapping | canonical |
| jets/ | per-group genesis jet specs (hash, recursion, binary-tower, polynomial-ring, isogeny-curves, tropical-semiring, state, decider) | canonical |
| trace.md | execution trace layout, AIR constraints, polynomial encoding | canonical |
| encoding.md | canonical noun serialization, wire format, content addressing | canonical |

## reading order

1. vm.md — field, hash, and dependencies (what nox computes over)
2. noun/ — the data model (what nox operates on)
3. patterns/ — the instruction set (what nox can do)
4. reduction.md — the execution model (how nox evaluates)
5. jets.md + jets/ — the optimization layer (how nox goes fast)
6. trace.md — the proof witness (how nox proves)
7. encoding.md — the wire format (how nox serializes)

## dependencies

nox depends on two companion crates:

- nebu (~/git/nebu/rs) — Goldilocks field arithmetic. provides the F_p type and all field operations (add, sub, mul, inv, roots of unity). nox imports the field, it does not reimplement it.
- hemera (~/git/hemera/rs) — Hemera hash (Poseidon2-Goldilocks). provides the sponge construction, domain-separated hashing, and Merkle-compatible mode. nox imports the hash, it does not reimplement it.

zheng (~/git/zheng/) is a downstream consumer — it takes nox execution traces and produces stark proofs.

nox is frozen. no roadmap, no proposals. the specification is complete.

# nox reference

canonical specification of the nox virtual machine. this is the source of truth — when code and reference disagree, fix reference first, then propagate to code.

## specifications

| page | scope | status |
|------|-------|--------|
| vm.md | overview, field (aurum), hash (hemera), dependencies | canonical |
| nouns.md | data model: atom, cell, type tags, coercion, structural hash | canonical |
| patterns.md | all 17 patterns: Layer 1 (0-15) + Layer 2 hint (16) | canonical |
| reduction.md | reduction semantics, confluence, parallelism, memoization | canonical |
| jets.md | Layer 3: pure equivalents, hardware mapping, verifier costs | canonical |
| trace.md | execution trace layout, AIR constraints, polynomial encoding | canonical |
| encoding.md | canonical noun serialization, wire format, content addressing | canonical |

## reading order

1. vm.md — field, hash, and dependencies (what nox computes over)
2. nouns.md — the data model (what nox operates on)
3. patterns.md — the instruction set (what nox can do)
4. reduction.md — the execution model (how nox evaluates)
5. jets.md — the optimization layer (how nox goes fast)
6. trace.md — the proof witness (how nox proves)
7. encoding.md — the wire format (how nox serializes)

## dependencies

nox depends on two companion crates:

- aurum (~/git/aurum/rs) — Goldilocks field arithmetic. provides the F_p type and all field operations (add, sub, mul, inv, roots of unity). nox imports the field, it does not reimplement it.
- hemera (~/git/hemera/rs) — Hemera hash (Poseidon2-Goldilocks). provides the sponge construction, domain-separated hashing, and Merkle-compatible mode. nox imports the hash, it does not reimplement it.

zheng (~/git/zheng/) is a downstream consumer — it takes nox execution traces and produces stark proofs.

## design proposals

`props/` holds proposals for changes not yet committed to the spec. each proposal is a standalone markdown file with status frontmatter (draft, accepted, rejected, implemented). proposals document desire before commitment.

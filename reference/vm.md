# nox virtual machine specification

version: 0.1
status: canonical

## overview

nox is a proof-native virtual machine. sixteen deterministic reduction patterns over the Goldilocks field (Layer 1), one non-deterministic witness injection pattern (Layer 2), five jets for efficient recursive stark verification (Layer 3).

every nox execution produces a trace that IS the stark witness. there is no separate arithmetization step.

## field

nox operates over the Goldilocks field, provided by aurum (~/git/aurum/).

```
p = 2^64 - 2^32 + 1 = 18446744069414584321
primitive root: 7
2^32-th root of unity: 1753635133440165772
efficient reduction: a mod p = a_lo - a_hi × (2^32 - 1) + correction
```

the field choice is load-bearing: nox arithmetic IS Goldilocks arithmetic. the execution trace IS a table of Goldilocks elements. the stark proof IS over Goldilocks. there is no impedance mismatch at any layer.

aurum provides: field element type, addition, subtraction, multiplication, Fermat inversion, NTT-friendly roots of unity, Montgomery/Barrett reduction. nox imports the field — it does not reimplement it.

## hash

nox uses Hemera (~/git/hemera/) for all hashing: structural hash, Fiat-Shamir challenges, Merkle trees, content addressing.

```
HASH: Hemera (Poseidon2-Goldilocks)
  state: 12 field elements
  rate: 8 elements
  capacity: 4 elements
  rounds: 8 full + 22 partial + 8 full
  output: 4 field elements (256 bits)
  cost: ~300 stark constraints per permutation
```

hemera provides: sponge construction, domain-separated hashing, Merkle-compatible mode. nox imports the hash — it does not reimplement it.

## domain separation

```
COMMITMENT = 0x4E4F582020524543   "NOX  REC"
NULLIFIER  = 0x4E4F5820204E554C   "NOX  NUL"
MERKLE     = 0x4E4F5820204D524B   "NOX  MRK"
OWNER      = 0x4E4F5820204F574E   "NOX  OWN"
```

## three layers

```
Layer 1: 16 deterministic patterns   — the ground truth of computation
Layer 2: 1 non-deterministic hint    — the origin of privacy and search
Layer 3: 5 jets                      — optimization without changing meaning
```

remove Layer 3: identical results, ~8.5× slower. remove Layer 2: no privacy, no ZK. remove Layer 1: nothing remains.

## specification index

| page | scope |
|------|-------|
| nouns.md | data model: atom, cell, type tags, coercion, structural hash |
| patterns.md | all 17 patterns: Layer 1 (0-15) + Layer 2 hint (16) |
| reduction.md | reduction semantics, confluence, parallelism, memoization |
| jets.md | Layer 3 jets, pure equivalents, hardware mapping, verifier costs |
| trace.md | execution trace layout, AIR constraints, polynomial encoding |
| encoding.md | canonical wire format, content-addressed identity |

## dependencies

| crate | path | provides | nox uses |
|-------|------|----------|----------|
| aurum | ~/git/aurum/rs | Goldilocks field arithmetic | F_p type, add, sub, mul, inv, roots of unity |
| hemera | ~/git/hemera/rs | Hemera hash (Poseidon2-Goldilocks) | H(), domain separation, Merkle mode |
| zheng | ~/git/zheng/ | proof system (SuperSpartan + WHIR) | stark proving/verifying (downstream consumer of trace) |

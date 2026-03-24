# nox virtual machine specification

version: 0.2
status: canonical

## overview

nox is a proof-native virtual machine. sixteen deterministic reduction patterns parameterized by algebra, plus one non-deterministic witness injection pattern and five jets for efficient recursive stark verification.

every nox execution produces a trace that IS the stark witness. there is no separate arithmetization step.

## algebra polymorphism

nox is parameterised over its algebra. a nox instance is:

```
nox<F, W, H> where:
  F = field        (determines patterns 5-10: add, sub, mul, inv, eq, lt)
  W = word width   (determines patterns 11-14: xor, and, not, shl)
  H = hash function (determines pattern 15: hash)
```

key instantiations:

| instantiation | F | W | H | role |
|---------------|---|---|---|------|
| nox<Goldilocks, Z/2^32, Hemera> | F_p, p = 2^64 - 2^32 + 1 | Z/2^32 | Hemera (Poseidon2-Goldilocks) | canonical — all concrete costs in this spec |
| nox<F_2, Z/2, external> | F_2 | Z/2^1 | Grostl or external | Bt (binary world) — quantized inference, tri-kernel |
| nox<F_{p^3}, Z/2^32, Hemera> | F_{p^3} | Z/2^32 | Hemera | Tri recursion context |
| nox<F_{p^2}, Z/2^32, Hemera> | F_{p^2} | Z/2^32 | Hemera | quantum simulation context |

structural patterns (0-4) are identical across all instantiations. field patterns (5-10) dispatch to the instantiated field. bitwise patterns (11-14) dispatch to the instantiated word width. hash (15) dispatches to the instantiated hash function. jets are per-instantiation — each algebra has its own jet registry with its own formula hashes.

## algebra-polymorphic patterns

the 16 patterns are abstract operations parameterized by algebra, not tied to a specific field. the pattern semantics are universal — the algebra is a parameter.

three groups map to algebraic domains:

```
structural (0-4):   algebra-independent — tree operations work over any leaf type
field (5-10):       parameterized by F — any field
bitwise (11-14):    parameterized by W — any word width
hash (15):          parameterized by H — any hash function
```

the same `add(a, b)` means:
- in F_p context: `(a + b) mod p`
- in F_{p³} context: extension field addition (3 base additions)
- in F₂ context: XOR (binary addition)

the same `and(a, b)` means:
- in Z/2^32 context: 32-bit bitwise AND
- in F₂ context: multiplication (AND = mul in characteristic 2)
- in Z/2^64 context: 64-bit bitwise AND

the operations are identical. the algebra is a parameter. the programmer writes one tree. the prover splits it by algebra. cross-algebra boundaries are Hemera commitments.

## proof-system polymorphism

the same nox program can be verified by different proof systems:

```
nox<F_p> program     → zheng STARK (field-native, 1 constraint per field op)
nox<F₂> program      → Binius/FRI (binary-native, 1 constraint per binary op)
nox<F_{p³}> program   → zheng STARK with extension (3× wider constraints)
```

a single nox program containing both field and bitwise operations can be partitioned by the prover into algebra-specific sub-trees, with cross-algebra boundaries using Hemera commitments.

constraint costs are per-instantiation:

```
and(a, b) in nox<F_p>:   ~32 STARK constraints (bit decomposition in prime field)
and(a, b) in nox<F₂>:    1 constraint (native multiplication in binary field)
```

the cost difference is the honest algebraic distance between fields. the patterns stay. the proof system chooses the cheapest verification path.

## canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>

this specification documents one instantiation. all concrete parameters, costs, and test vectors refer to:

```
F = Goldilocks field, F_p where p = 2^64 - 2^32 + 1
W = Z/2^32 (32-bit words, fitting cleanly in [0, p))
H = Hemera (Poseidon2-Goldilocks sponge)
```

## field — Goldilocks (canonical)

nox<Goldilocks> operates over the Goldilocks field, provided by nebu (~/git/nebu/).

```
p = 2^64 - 2^32 + 1 = 18446744069414584321
primitive root: 7
2^32-th root of unity: 1753635133440165772
efficient reduction: a mod p = a_lo - a_hi × (2^32 - 1) + correction
```

in this instantiation, nox arithmetic IS Goldilocks arithmetic. the execution trace IS a table of Goldilocks elements. the stark proof IS over Goldilocks. there is no impedance mismatch at any layer.

nebu provides: field element type, addition, subtraction, multiplication, Fermat inversion, NTT-friendly roots of unity, Montgomery/Barrett reduction. nox imports the field — it does not reimplement it.

## hash — Hemera (canonical)

nox<_, _, Hemera> uses Hemera (~/git/hemera/) for all hashing: structural hash, Fiat-Shamir challenges, Merkle trees, content addressing.

```
HASH: Hemera (Poseidon2-Goldilocks, hemera-2)
  state: 16 field elements
  rate: 8 elements
  capacity: 8 elements
  rounds: 8 full (4+4) + 16 partial = 24 total
  s-box (full rounds): x^7
  s-box (partial rounds): x^{-1} (field inversion, 0^{-1} = 0)
  output: 4 field elements (32 bytes)
  cost: ~736 stark constraints per permutation
```

hemera provides: sponge construction, domain-separated hashing, Merkle-compatible mode. nox imports the hash — it does not reimplement it.

## the Hemera anchor

all algebras settle through Goldilocks via Hemera (Poseidon2 over Goldilocks). this creates a permanent F_p dependency:

- nox<F₂> programs must defer Hemera to settlement (~10 constraints deferred, ~736 at settlement)
- nox<F_{p³}> programs compute Hemera natively (Hemera operates over base field F_p)
- nox<F_p> programs compute Hemera natively (200 focus cost)

the polymorphism is real for computation but asymmetric for commitment. Goldilocks is the anchor field. all algebras settle through it.

## domain separation

```
COMMITMENT = 0x4E4F582020524543   "NOX  REC"
NULLIFIER  = 0x4E4F5820204E554C   "NOX  NUL"
MERKLE     = 0x4E4F5820204D524B   "NOX  MRK"
OWNER      = 0x4E4F5820204F574E   "NOX  OWN"
```

domain separation tags are injected into Hemera's sponge capacity[11] (the domain tag slot) before permutation. they ensure that hashes computed for different purposes are cryptographically distinct — a commitment hash cannot collide with a nullifier hash, even for identical input data.

nox programs invoke domain-separated hashing via pattern 15 (hash) with the tag as a capacity parameter. the tag is a protocol constant — it is not user-configurable. the VM sets capacity[11] based on the calling context:
- structural hash of nouns: capacity[11] = DOMAIN_HASH (Hemera default, 0x00)
- record commitment: capacity[11] = COMMITMENT
- nullifier derivation: capacity[11] = NULLIFIER
- Merkle tree operations: capacity[11] = MERKLE
- owner address derivation: capacity[11] = OWNER

## three layers

```
Layer 1: 16 deterministic patterns   — the ground truth of computation
Layer 2: 1 non-deterministic hint    — the origin of privacy and search
Layer 3: 5 jets                      — optimization without changing meaning
```

remove Layer 3: identical results, ~8.5× slower. remove Layer 2: no privacy, no ZK. remove Layer 1: nothing remains.

## other instantiations

the canonical instantiation is nox<Goldilocks, Z/2^32, Hemera>. other instantiations are possible — see the algebra polymorphism section above for the full table. the 4-bit encoding, the trace layout, the focus metering, the confluence property — all are properties of the abstract pattern set, not of a specific field. a new instantiation reuses the same spec with different F, W, H parameters.

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
| nebu | ~/git/nebu/rs | Goldilocks field arithmetic | F_p type, add, sub, mul, inv, roots of unity |
| hemera | ~/git/hemera/rs | Hemera hash (Poseidon2-Goldilocks) | H(), domain separation, Merkle mode |
| zheng | ~/git/zheng/ | proof system (SuperSpartan + WHIR) | stark proving/verifying (downstream consumer of trace) |

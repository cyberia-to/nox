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

structural patterns (0-4) are identical across all instantiations. field patterns (5-10) dispatch to the instantiated field. bitwise patterns (11-14) dispatch to the instantiated word width. hash (15) dispatches to the instantiated hash function. jets are per-instantiation — each algebra has its own jet registry with its own formula hashes.

## execution regimes

eight instantiations across five arithmetics. flat table — no hierarchy. each regime has its own field, costs, jets, and PCS backend. see [[five algebras]] for the independence criteria.

| regime | field | repo | [[lens]] | jets | mul cost | constraints/mul | role |
|--------|-------|------|----------|------|----------|-----------------|------|
| nebu | F_p scalar | [[nebu]] | Brakedown | 5 verifier | 1 | 1 | truth (canonical) |
| nebu² | F_p[u]/(u²−7) | [[nebu]]::Fp2 | Brakedown (2× wide) | fp2_mul, fp2_inv | 3 | 3 | quantum, 128-bit |
| nebu³ | F_p[t]/(t³−t−1) | [[nebu]]::Fp3 | Brakedown (3× wide) | fp3_mul, fp3_inv | 6 | 6 | recursion soundness |
| nebu⁴ | F_p[w]/(w⁴−7) | [[nebu]]::Fp4 | Brakedown (4× wide) | fp4_mul, fp4_inv | 9 | 9 | 256-bit, recursion tower |
| kuro | F₂ tower | [[kuro]] | Binius | 8 binary | 1 | 1 | efficiency |
| jali | R_q (n=1024) | [[jali]] | Ring-aware | 5 ring | 3072 | ~N (batched) | veil |
| trop | (min,+) | [[trop]] | Tropical | 6 tropical | — | O(\|witness\|) | choice |
| genies | F_q (512-bit) | [[genies]] | Isogeny | 5 isogeny | 1 F_q | 1 F_q | shadow |

mul cost = base F_p multiplications per one regime-native multiply. constraints/mul = STARK constraints per multiply with regime-native PCS (with jets). jali batching: N individual commitments → 1 batch via PCS₃.

### per-regime cost table

| pattern | nebu | nebu² | nebu³ | nebu⁴ | kuro | jali | trop | genies |
|---------|------|-------|-------|-------|------|------|------|--------|
| add | 1 | 2 | 3 | 4 | 1 | n | 1 | 1 F_q |
| sub | 1 | 2 | 3 | 4 | 1 | n | 1 | 1 F_q |
| mul | 1 | 3 | 6 | 9 | 1 | 3n | 1 | 1 F_q |
| inv | 64 | ~130 | ~200 | ~260 | 1 | — | — | ~4000 |
| eq | 1 | 2 | 3 | 4 | 1 | n | 1 | 8 |
| lt | 1 | — | — | — | 1 | — | 1 | — |
| xor | 1 | — | — | — | 1 | — | — | — |
| and | 1 | — | — | — | 1 | — | — | — |
| hash | 300 | 300 | 300 | 300 | deferred | 300 | 300 | deferred |
| STARK/mul | ~32 | 3 | 6 | 9 | 1 | ~N (batch) | O(w) | 1 F_q |

"—" = operation not defined or not meaningful for this regime. n = ring degree (1024). deferred = hemera computed at settlement boundary (~766 constraints). all costs are execution focus; STARK constraint counts are per-instantiation (see [[patterns]]).

five arithmetics (repos): nebu (4 regimes), kuro (1), jali (1), trop (1), genies (1).
five PCS backends (zheng): Brakedown (4 regimes), Binius (1), Ring-aware (1), Isogeny (1), Tropical (1).

### how the eight regimes enter nox

**nebu (F_p):** canonical instantiation. all patterns operate natively. all costs in this spec refer to this regime.

**nebu², nebu³, nebu⁴:** same nox parameterization pattern, wider field elements. one F_p² mul = 3 base muls (Karatsuba). one F_p³ mul = 6 base muls. one F_p⁴ mul = 9 base muls (tower Fp2→Fp4). extension jets (fp2_mul, fp3_mul, fp4_mul, inverses) recognize these structured compositions. all use Brakedown PCS₁ with proportionally wider columns.

**kuro (F₂):** separate instantiation. field patterns (add = XOR, mul = AND) are native binary operations at 1 constraint each (vs ~32 in F_p). bitwise patterns collapse to field operations (and = mul in characteristic 2). hash deferred to hemera at settlement boundary (~766 constraints per crossing).

**jali (R_q):** runs on the SAME nebu instantiation (F_p). R_q = F_p[x]/(x^n+1) is a polynomial RING over F_p, not a separate field. ring operations decompose to F_p operations via NTT. dedicated jets (ntt_batch, key_switch, blind_rotate) recognize structured compositions and commit them as batched ring operations in zheng PCS₃.

**trop (min,+):** runs on the SAME nebu instantiation (F_p). tropical operations decompose to existing patterns: min(a,b) = branch(lt(a,b), a, b). dedicated jets produce structured witnesses (assignment + cost + dual certificate) verified in F_p via zheng PCS₅.

**genies (F_q):** separate instantiation with a DIFFERENT prime q. the only regime with a foreign field. F_q elements are multi-limb (8 × 64-bit for CSIDH-512). patterns 5-10 dispatch to F_q arithmetic (Montgomery multiplication). hash remains hemera (at settlement boundary). dedicated PCS₄ in zheng (Brakedown over F_q).

### cross-algebra composition

a single nox program can mix algebras. the prover partitions the trace into algebra-specific sub-traces. each sub-trace proves via its native PCS backend. HyperNova folds all into one F_p accumulator. one decider, one proof.

```
boundary cost: ~766 F_p constraints per algebra crossing
               (30 field ops + 1 hemera hash)

example — FHE bootstrapping crosses three algebras:
  jali → kuro (gadget_decomp)      ~766
  kuro → jali (blind rotation)     ~766
  jali → nebu (key switching)      ~766
  total boundary:                  ~2,298
```

universal CCS with selectors enables heterogeneous folding:

```
sel_Fp:   1 for Goldilocks rows (nebu, jali, trop)
sel_F2:   1 for binary rows (kuro)
sel_ring: 1 for ring-structured rows (jali — NTT batch, automorphisms)
sel_Fq:   1 for isogeny rows (genies)
sel_trop: 1 for tropical witness-verify rows (trop)
```

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

### polynomial nouns and axis

every noun is a multilinear polynomial (see nouns.md polynomial representation). axis — the fundamental navigation operation — becomes polynomial evaluation at a binary point. a PCS opening proves the evaluation in O(1) (~75 bytes proof), replacing O(depth) tree traversal. the 16 patterns are unchanged semantically — axis still navigates nouns. the implementation changes from pointer-following to polynomial evaluation. this applies across all instantiations: the noun polynomial is over the instantiated field F, and the PCS commitment uses the same field.

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
HASH: Hemera (Poseidon2-Goldilocks, hemera)
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
Layer 1: 16 deterministic patterns     — the ground truth of computation
Layer 2: 1 non-deterministic hint      — the origin of privacy and search
Layer 3: 30+ jets across 5 algebras   — optimization without changing meaning
```

remove Layer 3: identical results, orders of magnitude slower. remove Layer 2: no privacy, no ZK. remove Layer 1: nothing remains. see [[jets/]] for the full jet registry.

## adding new instantiations

the 4-bit encoding, the trace layout, the focus metering, the confluence property — all are properties of the abstract pattern set, not of a specific field. a new instantiation reuses the same spec with different F, W, H parameters. see the five algebras section for the current instantiation table and cross-algebra composition.

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

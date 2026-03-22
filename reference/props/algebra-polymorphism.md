---
title: algebra-polymorphic patterns
status: accepted
date: 2026-03-17
diffusion: 0.00010722364868599256
springs: 0.00007019991600688145
heat: 0.00003419142694206788
focus: 0.00008151008453347325
gravity: 0
density: 0
---

# algebra-polymorphic patterns

## summary

nox's 16 deterministic patterns are not Goldilocks-specific. they are abstract operations parameterized by the algebra they execute in. the field, the word width, and the proof system are parameters — the pattern semantics are universal.

this proposal reframes nox from "a Goldilocks VM with three instruction groups" to "a universal pattern set instantiable over any algebra, with proof-system-polymorphic verification."

## the observation

nox has 16 patterns in three groups:

```
structural (0-4):   axis, quote, compose, cons, branch
field (5-10):       add, sub, mul, inv, eq, lt
bitwise (11-14):    xor, and, not, shl
hash (15):          H(x)
```

each group maps to an algebraic domain. the structural patterns are algebra-independent — tree operations work over any leaf type. the field patterns are parameterized by which field: F_p, F_{p²}, F_{p³}, F₂, any field. the bitwise patterns are parameterized by word width: Z/2^32, Z/2^64, Z/2^n. the hash pattern is parameterized by which hash function.

the same `add(a, b)` means:
- in F_p context: (a + b) mod p
- in F_{p³} context: extension field addition (3 base additions)
- in F₂ context: XOR (binary addition)

the same `and(a, b)` means:
- in Z/2^32 context: 32-bit bitwise AND
- in F₂ context: multiplication (AND = mul in characteristic 2)
- in Z/2^64 context: 64-bit bitwise AND (pair of 32-bit words)

the operations are identical. the algebra is a parameter.

## nox<F> — the instantiation model

a nox instance is parameterized by:

```
nox<F, W, H> where:
  F = field (determines patterns 5-10)
  W = word width (determines patterns 11-14)
  H = hash function (determines pattern 15)
```

the current nox spec describes one instantiation:

```
nox<Goldilocks, Z/2^32, Hemera>
  F = F_p where p = 2^64 - 2^32 + 1
  W = Z/2^32 (32-bit words, fitting cleanly in [0, p))
  H = Hemera (Poseidon2-Goldilocks sponge)
```

other instantiations exist:

```
nox<F₂, Z/2^1, Grøstl>       = Bt (binary world)
nox<F_{p³}, Z/2^32, Hemera>   = Tri recursion context
nox<F_{p²}, Z/2^32, Hemera>   = quantum simulation context
```

the structural patterns (0-4) are the same in every instantiation. they operate on trees of atoms, regardless of what the atoms are.

## proof-system polymorphism

the same nox program can be verified by different proof systems:

```
nox<F_p> program   → zheng STARK (field-native, 1 constraint per field op)
nox<F₂> program    → Binius/FRI (binary-native, 1 constraint per binary op)
nox<F_{p³}> program → zheng STARK with extension (3× wider constraints)
```

a single nox program containing both field and bitwise operations can be partitioned by the prover:

```
program = [5 [                        ← add: field op
            [12 [[0 2] [0 3]]]        ← and: bitwise op
            [7 [[0 4] [0 5]]]         ← mul: field op
          ]]

prover partitions:
  field sub-tree:   patterns 5, 7 → verify in zheng (F_p STARK)
  bitwise sub-tree: pattern 12   → verify in Binius (F₂)
  boundaries:       Hemera commitments (zero translation)
```

the programmer writes one tree. the prover splits it by algebra. cross-algebra boundaries are Hemera commitments.

## resolution of the bitwise question

the 32-bit word width is not a limitation of nox — it is a property of the Goldilocks instantiation. patterns 11-14 are the same Boolean operations in every algebra. the proof cost differs:

```
and(a, b) in nox<F_p>:   ~32 STARK constraints (bit decomposition in prime field)
and(a, b) in nox<F₂>:    1 constraint (native multiplication in binary field)
```

the 32× overhead is not a design flaw. it is the honest algebraic distance between F_p and F₂. the patterns stay. the proof system chooses the cheapest verification path.

keeping all 16 patterns:
- the encoding stays 4 bits (maximally dense)
- programs are portable across instantiations
- the prover optimizes, not the programmer
- no instruction set fragmentation

## domain-specific operations as jets

the 16 patterns are the atomic instruction set. domain-specific language operations are compositions of these patterns — recognized by formula hash, accelerated as jets.

```
language operation       nox composition              jet           GFP hardware
─────────────────────    ──────────────────────────   ──────────    ────────────
Arc: rank(g, steps)      iterated add/mul loops       matmul jet    fma array
Wav: fft(x)              butterfly add/mul network    ntt jet       ntt engine
Any: hash(x)             Poseidon2 field ops          hash jet      p2r pipeline
Ten: activation(x)       table lookup composition     lookup jet    lut engine
Geo: geometric_product   mul/add over components      geo_mul jet   fma array
Wav: polynomial_mul      NTT + pointwise + iNTT       ntt jet       ntt engine
```

the jet mechanism unifies language-specific optimization with hardware acceleration. a jet maps a formula hash to an optimized implementation — which may execute on specialized hardware. the same mechanism that speeds up the STARK verifier (the 5 existing jets) also speeds up every domain-specific language.

the GFP's four hardware primitives (fma, ntt, p2r, lut) are the physical substrate that jets map to. the chain:

```
source language → compiler → nox pattern tree → jet recognition → GFP hardware
```

every domain-specific language gets hardware acceleration through the jet mechanism. no language-specific hardware needed. the algebra determines which GFP primitive handles each jet.

## implications for the multiproof architecture

the multiproof architecture describes three tiers: execution (11 languages), proving (Tri + Hemera), composition (nox). the polymorphism insight refines this:

nox is not above the tiers — it spans all three:
- **execution:** nox patterns ARE the execution (the 16 patterns compute)
- **proving:** the proof system is a parameter of the nox instantiation
- **composition:** structural patterns (0-4) compose sub-computations across algebras

the execution languages (Bt, Rs, Arc, Geo, Ten, Wav, Seq, Ask) are type systems and compilers over nox patterns. they add domain-specific syntax, type checking, and compilation strategies — but the target is always nox pattern trees. domain-specific operations become jets.

```
Bt  = nox<F₂> + binary type system + Binius prover
Rs  = nox<F_p> + Rust-subset type system + word-range constraints
Tri = nox<F_{pⁿ}> + field-visible type system + zheng prover
Arc = nox<F_p> + graph type system + category-theoretic jets
Geo = nox<F_p> + Clifford type system + geometric product jets
Ten = nox<F_p> + tensor type system + matmul/einsum jets
Wav = nox<F_p> + signal type system + NTT/convolution jets
```

## implications for hardware

the 16 patterns decompose into compute and memory:

**compute (small, universal, same for every algebra):**
- field ALU: patterns 5-10 → GFP fma unit
- binary ALU: patterns 11-14 → simple gate array (AND/XOR)
- hash unit: pattern 15 → GFP p2r pipeline
- the ALU is tiny and fixed

**memory (large, algebra-dependent):**
- tree traversal: patterns 0-4 → content-addressed noun store
- leaf-width-adaptive: F₂ atoms = 1 bit, F_p atoms = 64 bits, hash atoms = 512 bits
- access patterns differ per algebra: dense sequential (Ten), random (Arc), compact (Bt)
- content-addressed storage IS the wiring — axis paths connect operations to data

optimize the memory system, not the ALU. the operations are universal. the storage topology is where algebras differ.

## the Hemera anchor

full field-independence is impossible. Hemera (Poseidon2 over Goldilocks) is the universal commitment scheme — every computation, in every algebra, commits via Hemera. Hemera output is 8 Goldilocks field elements. this creates a permanent F_p dependency:

- nox<F₂> programs must defer Hemera to settlement (~10 constraints deferred, ~1,200 at settlement)
- nox<F_{p³}> programs compute Hemera natively (Hemera operates over base field F_p)
- nox<F_p> programs compute Hemera natively (300 focus cost)

the Hemera invariant means the polymorphism is real for computation but asymmetric for commitment. Goldilocks is the anchor field. all algebras settle through it.

## what changes in nox spec

this proposal does not change the existing nox specification. nox<F_p, Z/2^32, Hemera> remains the canonical instantiation. what changes is the framing:

1. patterns are described as abstract operations with field-parametric semantics
2. the Goldilocks instantiation is documented as the canonical instance, not the definition
3. the cost model is explicitly per-instantiation (bitwise costs ~32 in F_p, ~1 in F₂)
4. jets are documented as the mechanism for domain-specific language acceleration
5. the proof system is documented as a parameter, not a fixed choice

the 4-bit encoding, the trace layout, the focus metering, the confluence property — all remain unchanged. they are properties of the abstract pattern set, not of a specific field.

## open questions

1. **noun model parametricity:** atoms have type tags (field 0x00, word 0x01, hash 0x02). in nox<F₂>, what are the type tags? does the three-type value tower generalize across fields?

2. **focus accounting across algebras:** focus is unified (computation and cyberlink creation share one budget). if a program spans two algebras, how is focus partitioned? the current spec threads focus sequentially — cross-algebra partitioning is unspecified.

3. **cross-algebra jet boundaries:** when the prover partitions a tree into F_p and F₂ sub-trees, the boundary requires Hemera commitments. the cost of these boundary crossings determines the optimal partition granularity. too fine-grained = boundary overhead dominates. too coarse = suboptimal proof costs.

4. **canonical formula hashes across instantiations:** the same source program compiles to different pattern trees in different instantiations (e.g., u64 in Rs becomes a pair of u32 words). do jet hashes need to be per-instantiation?
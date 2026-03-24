# jet specification

version: 0.3
status: canonical

## overview

jets are compositions of Layer 1 patterns recognized by formula hash and replaced with optimized implementations. every jet has an equivalent pure Layer 1 program producing identical output on all inputs. jets are OPTIMIZATION — semantics unchanged.

the jet mechanism is algebra-polymorphic. jet formula hashes are computed from the abstract pattern trees. the optimized implementation is per-instantiation — the same jet dispatches to different backends depending on the algebra (software, GFP hardware, or delegated to a specialized prover).

two categories of jets: **verifier jets** (proof-system specific, per-instantiation) and **domain-specific jets** (language-specific, open-ended). both use the same formula-hash recognition mechanism.

## semantic contract

every jet MUST have an equivalent pure Layer 1 expression producing identical output on all inputs.

this is testable: a harness compares jet output against pure-pattern output on random inputs. if a jet is removed, the system remains correct — only slower. jets are never load-bearing for correctness, only for performance.

## jet recognition

jets are recognized by formula hash. the VM maintains a jet registry mapping formula identities to optimized implementations.

```
jet_registry: H(formula_noun) → jet_implementation

recognition: at reduction time, before dispatching a formula,
the VM checks if H(formula) is in the jet registry.
if yes: execute the jet (same result, fewer trace rows).
if no:  dispatch normally via Layer 1 patterns.
```

the jet registry is hardcoded — it is a protocol constant, not configurable. every conforming implementation MUST recognize the same set of jets. the canonical formula trees and their hashes are computed at build time from the pure Layer 1 definitions and committed as constants.

jet registry entries are generated, not hand-written: the build system constructs each jet's pure Layer 1 formula as a noun, computes its structural hash, and emits the registry as a constant table. this ensures the hashes are correct by construction.

## per-instantiation dispatch

jet implementations are per-instantiation. the same formula hash dispatches to:
- software implementation (any instantiation)
- GFP hardware primitives (nox<Goldilocks> on GFP-equipped hardware)
- delegated prover (cross-algebra boundary jets)

each nox instantiation defines its own jet registry. formula hashes may differ across algebras if the same source operation compiles to different pattern trees.

```
nox<Goldilocks>  →  verifier jets (hash, poly_eval, merkle_verify, fri_fold, ntt)
nox<F₂>          →  binary jets (popcount, packed_inner_product, binary_matvec, ...)
```

## adding a new jet

to add a jet:

1. **write the pure Layer 1 formula** — the jet's semantics are defined by this formula. it must be a valid nox program using only patterns 0-16. no external dependencies.

2. **compute the formula hash** — build the formula as a noun, compute H(formula_noun) via hemera. this hash is the jet's identity. the build system does this automatically.

3. **implement the optimized version** — write the jet implementation for each target backend:
   - software: native Rust function with the same input/output contract
   - constraint: optimized CCS encoding (fewer rows than naive pattern trace)
   - hardware: GFP primitive mapping (if applicable)

4. **add to the jet registry** — register the (formula_hash, implementation) pair. the registry is a protocol constant — adding a jet requires a protocol upgrade.

5. **write the test harness** — property test: `∀ inputs: jet(input) == pure_formula(input)`. this is the jet's correctness proof. it runs on every build.

6. **document the jet** — specify: input types, output type, pure cost, jet cost, constraint count, which workloads it accelerates.

```
jet entry format:

  name:              human-readable identifier
  formula_hash:      H(pure Layer 1 formula noun)
  input:             type signature
  output:            type signature
  pure_cost:         Layer 1 pattern count
  jet_cost:          optimized execution cost
  constraints:       STARK constraint count
  accelerates:       list of workloads
  hardware_mapping:  GFP primitive (if applicable)
```

## jet categories

### verifier jets

proof-system-specific jets that make recursive composition practical. each nox instantiation has its own verifier jets matched to its proof system.

#### nox<Goldilocks> + WHIR verifier jets

five jets that make recursive proof composition practical. the unoptimized verifier costs ~400,000 patterns. with jets: ~50,000. this 8x reduction makes recursive composition practical.

| jet | signature | exec cost | stark constraints | pure Layer 1 cost |
|-----|-----------|-----------|-------------------|--------------------|
| 0: hash | hash(x) -> 4 x F_p | 200 | ~736 | ~1,000 |
| 1: poly_eval | poly_eval(coeffs, point) -> F_p | N | ~N | ~2N |
| 2: merkle_verify | merkle_verify(root, leaf, path, index) -> {0,1} | d x 200 | ~d x 736 | d x ~210 |
| 3: fri_fold | fri_fold(poly_layer, challenge) -> poly_layer_next | N/2 | ~N/2 | ~N |
| 4: ntt | ntt(values, direction) -> transformed values | N*log(N) | ~N*log(N) | ~2N*log(N) |

verifier cost breakdown:

```
Component               | Layer 1 only | With jets  | Reduction
------------------------+--------------+------------+----------
Parse proof             |     ~1,000   |    ~1,000  |  1x
Fiat-Shamir challenges  |    ~20,000   |    ~3,000  |  7x
Merkle verification     |   ~330,000   |   ~33,000  | 10x
Constraint evaluation   |    ~10,000   |    ~3,000  |  3x
WHIR verification       |    ~35,000   |    ~7,000  |  5x
------------------------+--------------+------------+----------
TOTAL                   |   ~400,000   |   ~50,000  | ~8x
```

#### nox<F_2> + Binius binary jets

eight jets for the Bt binary prover. base operations (XOR, AND, NOT, SHL) are already 1 constraint each in F_2 — jets target composite operations that appear millions of times in quantized inference and tri-kernel SpMV.

| jet | input | naive | jet | speedup | primary workload |
|-----|-------|-------|-----|---------|------------------|
| 0: popcount | F_2^128 -> Z | ~640 | ~128 | 5x | all accumulation |
| 1: packed_inner_product | F_2^128^2 -> Z | ~5n | ~128 | 5x | matmul kernel |
| 2: binary_matvec | F_2^{m x n} -> Z^m | m x 5n | m x 128 | 5x | inference, tri-kernel |
| 3: quantize | F_p -> F_2^k | ~k^2 | ~k | kx | F_p -> F_2 boundary |
| 4: dequantize | F_2^k -> F_p | ~k^2 | ~k | kx | F_2 -> F_p boundary |
| 5: activation_lut | F_2^k -> F_2^m | ~2^k/lookup | ~k/lookup | 2^k/kx | activation functions |
| 6: gadget_decompose | F_p -> F_2^k | ~k^2 | ~k | kx | FHE bootstrapping |
| 7: barrel_shift | F_2^n -> F_2^n | ~n^2 | ~3n*log(n) | n/3log x | crypto, permutations |

constraint count understates the jet advantage. Binius prover operates on packed u128 words — 128 F_2 elements per machine operation. popcount/packed_inner_product/binary_matvec achieve ~90x prover wall-clock speedup via SIMD packing on top of the constraint reduction.

### state jets

jets that optimize PROVING state transitions. verifier jets reduce trace length (fewer execution steps). state jets reduce CONSTRAINT COUNT (fewer CCS constraints per transition). both use the same formula-hash recognition mechanism.

a state transition is a nox program that reads polynomial state, validates changes, and writes updated values. the 5 primitive state operations (see [[state-operations]]) are nox patterns: READ/WRITE (polynomial evaluation), ASSERT_EQ (pattern 9), ADD (pattern 5), MUL (pattern 7). common compositions of these operations — TRANSFER, INSERT, UPDATE — have state jets.

#### recognition hierarchy

**level 1 (exact formula match):** H(formula) → hand-optimized CCS encoding.

the genesis table circuits (cyberlink: ~3,200 constraints) are level 1 state jets.

**level 2 (pattern match):** nox formula matches a template → parameterized CCS encoding.

| template | decomposition | jet constraints | without jet |
|---|---|---|---|
| TRANSFER(source, target, amount) | 2 READ + RANGE + 2 ADD + 2 WRITE + ASSERT_EQ | 3 | ~8,000 |
| INSERT(table, key, value) | READ(=0) + schema_check + WRITE | 5 | ~6,000 |
| UPDATE(table, key, old, new) | READ + ASSERT_EQ + WRITE | 5 | ~4,000 |
| AGGREGATE(table, key, delta) | READ + ADD + WRITE | 2 | ~4,000 |
| CONSERVE(inputs, outputs) | ADDs + ASSERT_EQ | n | ~4,000 |

level 2 jets fire AUTOMATICALLY when the formula structure matches a template. no protocol upgrade needed to recognize patterns in new user-defined tables.

**level 3 (type-based):** schema-aware generic encoding → constraints proportional to schema size.

**fallback:** full nox execution trace → SuperSpartan proof (~8 constraints per trace row).

#### interaction with proof-carrying

without state jet: each nox reduce() step folds into accumulator (~30 field ops × 500 steps = 15,000 field ops).
with state jet: pattern recognized → 1 fold of the 3-5 constraint instance = ~30 field ops total. **500× speedup.**

### domain-specific jets

language operations recognized by the same formula-hash mechanism. open-ended — any frequently-used nox composition can become a jet:

```
language operation       nox composition              jet           GFP hardware
─────────────────────    ──────────────────────────   ──────────    ────────────
Arc: rank(g, steps)      iterated add/mul loops       matmul jet    fma array
Wav: fft(x)              butterfly add/mul network    ntt jet       ntt engine
Any: hash(x)             Poseidon2 field ops          hash jet      p2r pipeline
Ten: activation(x)       table lookup composition     lookup jet    lut engine
Ren: geometric_product   mul/add over components      geo_mul jet   fma array
Wav: polynomial_mul      NTT + pointwise + iNTT       ntt jet       ntt engine
```

domain-specific jets follow the same semantic contract as verifier jets. the jet registry is per-instantiation — each algebra's jets are recognized by their own formula hashes.

### cross-algebra boundary jets

jets that handle the F_p ↔ F₂ transition:

```
quantize:    F_p value → F₂ binary representation
dequantize:  F₂ binary result → F_p value
```

these fire at algebra boundaries — when nox execution crosses from Goldilocks to binary or back. the constraint encoding spans both fields.

## hardware mapping

```
GFP primitive                    jets it accelerates
────────────────────────────     ─────────────────────────────────────
fma (field multiply-accumulate)  poly_eval (Horner = iterated FMA)
ntt (NTT butterfly)              ntt (direct correspondence)
p2r (Poseidon2 round)            hash, merkle_verify (hash-dominated)
lut (lookup table)               activation functions via Layer 1
```

the stack is continuous: nox pattern → software jet → GFP hardware primitive. the same computation, three speeds, identical semantics at every level.

## self-verification

the stark verifier for nox is itself a nox program. every operation the verifier needs — field arithmetic (patterns 5-8), hashing (jet 0), polynomial evaluation (jet 1), Merkle path checking (jet 2), FRI folding (jet 3) — is native to the sixteen patterns or their jet equivalents.

the VM can verify proofs about its own executions. a proof-of-proof is a nox program that runs the verifier on a proof. the proof-of-proof is itself provable. recursion to arbitrary depth, constant proof size at every level.

```
program → trace → stark proof → verifier (nox program) → trace → stark proof → ...
```

## proposals

jet designs originated as proposals, now merged into this spec:

- [[recursive-jets]] — Goldilocks/WHIR verifier jets (hash, poly_eval, merkle_verify, fri_fold, ntt) — accepted, merged above
- [[binary-jets]] — F₂/Binius jets (popcount, packed_inner_product, binary_matvec, quantize, dequantize, activation_lut, gadget_decompose, barrel_shift) — accepted, merged above

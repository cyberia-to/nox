---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-03-19
---
# binary jets — F₂/Binius prover acceleration

eight jets for the Bt binary prover. the base operations (XOR, AND, NOT, SHL) are already 1 constraint each in F₂ — they don't need jets. jets target composite operations that appear millions of times in quantized inference and tri-kernel SpMV.

## the core loop

both primary Bt workloads — quantized AI inference and tri-kernel SpMV — reduce to the same kernel:

```
binary matrix-vector multiply:
  for each row r of weight matrix W (binary):
    result[r] = popcount(W[r] AND input_vector)

popcount = count set bits = the accumulation step
```

XOR and AND are free in F₂ (1 constraint). popcount is the bottleneck: summing bits requires carry propagation, which is NOT free in F₂. a naive n-bit popcount requires a tree of full adders — each full adder is 5 binary constraints (2 XOR + 2 AND + 1 XOR for carry). total: ~5n constraints for n-bit popcount.

jets compress this.

## jet 0: popcount

```
popcount(x) → count
  input:  x ∈ F₂^n (packed binary vector, n ≤ 128)
  output: count ∈ Z_{log₂(n)+1} (integer, log₂(n)+1 bits wide)
```

count the number of set bits in a packed binary word.

```
algorithm: parallel prefix popcount (SWAR)
  step 1: sum pairs      (n/2 partial sums, 2 bits each)
  step 2: sum quads       (n/4 partial sums, 3 bits each)
  step 3: sum octets      (n/8 partial sums, 4 bits each)
  ...
  step k: final sum       (1 result, log₂(n)+1 bits)

  total: ~n constraints (each step is n/2 additions at growing width)
```

### constraint encoding

```
naive full-adder tree:
  n-1 full adders × 5 constraints each = ~5n constraints
  for n=128: ~640 constraints

jet parallel prefix:
  log₂(n) stages × n/2 wired additions = ~n constraints
  for n=128: ~128 constraints

  but the additions grow wider at each stage. careful encoding:
  stage 1: n/2 × 1-bit additions (XOR + AND = 2 constraints each) = n
  stage 2: n/4 × 2-bit additions (4 constraints each) = n
  ...
  stage k: 1 × (k-1)-bit addition = ~2k constraints

  total across all stages: ~n × log₂(n) / 2 constraints
  for n=128: ~448 constraints

  improvement over naive: ~640/448 ≈ 1.4× (modest)
```

the real win is not constraint count but PROVER EFFICIENCY: the parallel prefix structure maps perfectly to Binius's packed SIMD operations. the prover evaluates all n/2 partial sums in one packed u128 operation.

```
prover speedup:
  naive: 640 sequential constraint evaluations
  jet:   7 packed u128 SIMD operations (log₂(128) stages)
  prover speedup: ~90× wall-clock
```

- pure equivalent: ~640 patterns (full-adder tree)
- jet cost: ~128 (7 SIMD stages)
- binary constraints: ~448
- accelerates: every binary accumulation — matmul, hamming distance, threshold

the single most important Bt jet. every binary matmul row ends with popcount.

## jet 1: packed_inner_product

```
packed_inner_product(a, b) → count
  input:  a, b ∈ F₂^n (packed binary vectors)
  output: count = popcount(a AND b) ∈ Z_{log₂(n)+1}
```

inner product of two binary vectors. this IS the matmul kernel: one call per matrix row.

```
implementation:
  1. c = a AND b           (n constraints, but AND is free in F₂ = 0 constraints)
  2. count = popcount(c)   (jet 0: ~448 constraints for n=128)
  total: ~448 constraints
```

AND is literally free (it IS multiplication in F₂). the entire cost is popcount.

- pure equivalent: n AND + ~5n popcount = ~5n constraints
- jet cost: ~128
- binary constraints: ~448
- accelerates: binary matmul kernel, hamming distance, correlation

## jet 2: binary_matvec

```
binary_matvec(W, x) → y
  input:  W ∈ F₂^{m×n} (weight matrix), x ∈ F₂^n (input vector)
  output: y ∈ Z^m where y[i] = popcount(W[i] AND x)
```

batched matrix-vector multiply. the top-level jet for inference and tri-kernel.

```
naive: m separate packed_inner_product calls
  constraint cost: m × 448 = 448m

jet optimization — shared structure:
  1. commit x once (shared across all m rows)
  2. W rows packed into m × (n/128) u128 words
  3. AND is free → m×n binary constraints = 0
  4. m popcount calls with shared commitment

  constraint savings from shared input commitment:
    without sharing: m × (n + 448) ≈ m × 576 (for n=128)
    with sharing:    n + m × 448 ≈ m × 449 (for n=128)
    marginal savings: ~22% (from amortized input commitment)
```

the bigger win is prover efficiency:

```
4096×4096 binary matmul (one inference layer):
  naive patterns:      4096 × 4096 × 5 = ~84M binary constraints
  with popcount jet:   4096 × 448 = ~1.8M binary constraints
  with matvec jet:     4096 × 449 + 4096 = ~1.84M binary constraints

  equivalent F_p cost (without Binius): 84M × 32 = ~2.7B F_p constraints
  binary jet cost:                      ~1.84M binary constraints

  total speedup vs F_p: ~1,400×
```

- pure equivalent: m × ~5n constraints
- jet cost: m × ~128
- binary constraints: m × ~448 + n
- accelerates: quantized inference layers, tri-kernel SpMV iterations

## jet 3: quantize

```
quantize(v, k) → bits
  input:  v ∈ F_p (Goldilocks field element), k ∈ Z (bit width, k ≤ 64)
  output: bits ∈ F₂^k (binary representation)
```

the bridge from Goldilocks to binary. fires at every F_p → F₂ algebra crossing.

```
constraint encoding:
  prove: Σ_{i=0}^{k-1} bits[i] × 2^i = v    (range check in F_p)
  prove: each bits[i] ∈ {0, 1}                (free in F₂, 1 constraint each in F_p)

  the range check is an F_p constraint (weighted sum = field element)
  the bit constraints are F₂ constraints (trivially satisfied)

  cross-algebra constraint: ~k F_p constraints for the weighted sum
  + k trivial F₂ constraints for bit membership
```

the weighted sum `Σ bits[i] × 2^i = v` must be verified in F_p (where 2^i makes algebraic sense). the bits themselves live in F₂. this jet spans the algebra boundary.

- pure equivalent: ~k² patterns (bit decomposition via repeated halving)
- jet cost: ~k
- cross-algebra constraints: k F_p + k F₂
- accelerates: every Goldilocks → binary transition (model weight loading, input quantization)

## jet 4: dequantize

```
dequantize(bits, k) → v
  input:  bits ∈ F₂^k (binary representation), or accumulated integer from popcount
  output: v ∈ F_p (Goldilocks field element)
```

the bridge from binary back to Goldilocks. fires at every F₂ → F_p crossing.

```
constraint encoding:
  prove: v = Σ_{i=0}^{k-1} bits[i] × 2^i    (reconstruction in F_p)

  same cross-algebra structure as quantize, reversed direction.
  for popcount results: k = log₂(n)+1 (small, ~7 bits for 128-bit vectors)
```

- pure equivalent: ~k² patterns
- jet cost: ~k
- cross-algebra constraints: k F_p + k F₂
- accelerates: result collection after binary computation, accumulation into F_p

## jet 5: activation_lut

```
activation_lut(table, x) → table[x]
  input:  table ∈ F₂^{2^k × m} (lookup table, 2^k entries of m bits each)
          x ∈ F₂^k (k-bit index)
  output: result ∈ F₂^m
```

small lookup table for activation functions. quantized inference uses tables for nonlinearities (binary ReLU, sign, step, clipped linear).

```
mechanism: multilinear extension (MLE)
  the table is a function f: {0,1}^k → F₂^m
  MLE: f̃(x₁,...,xₖ) = Σ_{b∈{0,1}^k} f(b) × ∏ᵢ (xᵢ·bᵢ + (1-xᵢ)(1-bᵢ))

  evaluation at binary point x: f̃(x) = f(x)  (MLE agrees on Boolean hypercube)

  constraint encoding:
    commit table as MLE polynomial          (one-time, 2^k constraints)
    each lookup = evaluate MLE at x         (k multiplications = k constraints)
    prove evaluation matches claimed output (1 equality constraint)

  per-lookup cost after table commitment: ~k+1 constraints
```

```
example: 8-bit activation table (k=8, 256 entries)
  table commitment: 256 constraints (one-time per table)
  per-lookup: 9 constraints
  1000 lookups in an inference layer: 256 + 9000 = 9,256 constraints
  naive (enumerate all entries): 1000 × 256 = 256,000 constraints
  speedup: ~28×
```

- pure equivalent: ~2^k × m patterns per lookup (enumeration)
- jet cost: ~k+1 per lookup (after table commitment)
- binary constraints: 2^k (table) + (k+1) per lookup
- accelerates: activation functions, S-boxes, small nonlinearities

## jet 6: gadget_decompose

```
gadget_decompose(a, base, digits) → (a₀, a₁, ..., a_{digits-1})
  input:  a ∈ R_q coefficient (Goldilocks field element)
          base ∈ Z (decomposition base, typically 2)
          digits ∈ Z (number of digits)
  output: digit sequence where a = Σ aᵢ × base^i
```

the first phase of TFHE bootstrapping. decomposes each ring coefficient into binary digits for processing in the binary domain.

```
for base=2 (binary decomposition):
  identical to quantize but semantically distinct:
  quantize converts a value for binary computation
  gadget_decompose prepares FHE ciphertext for bootstrapping

  constraint encoding: same as quantize (~k cross-algebra constraints)

for base=4 or base=8 (multi-bit decomposition):
  each digit is 2 or 3 bits
  k/2 or k/3 digits instead of k
  per-digit range check: prove 0 ≤ aᵢ < base
  total: ~(k/log₂(base)) × log₂(base) = ~k constraints
```

the jet recognizes the gadget decomposition pattern (specific to FHE) and applies the optimized constraint encoding. the prover knows the decomposition base from the formula hash.

- pure equivalent: ~k² patterns (repeated modular reduction)
- jet cost: ~k
- cross-algebra constraints: k F_p + k F₂
- accelerates: TFHE blind rotation, key switching, all FHE bootstrapping phases

## jet 7: barrel_shift

```
barrel_shift(x, amount, direction) → shifted
  input:  x ∈ F₂^n (packed word)
          amount ∈ F₂^{log₂(n)} (shift amount, variable)
          direction ∈ F₂ (0=left, 1=right)
  output: shifted ∈ F₂^n
```

variable-amount shift or rotate. pattern 14 (shl) is fixed-amount only. barrel shift handles the general case.

```
algorithm: log₂(n) conditional shift stages
  stage i: if amount[i] = 1, shift by 2^i positions
  each stage: n MUX operations (select original or shifted)
  each MUX: 3 binary constraints (select = a·s + b·(1-s))

  total: log₂(n) × n × 3 constraints
  for n=128, log₂(n)=7: 7 × 128 × 3 = 2,688 constraints

  naive enumeration of all shift amounts:
    n possible amounts × n constraints per shift = n² constraints
    for n=128: 16,384 constraints

  speedup: ~6×
```

- pure equivalent: ~n² patterns (enumerate shift amounts)
- jet cost: ~3n·log₂(n)
- binary constraints: ~3n·log₂(n)
- accelerates: cipher round functions, permutation-based crypto, binary NTT butterfly

## summary

```
jet                   input           naive         jet          speedup   primary workload
─────────────────────────────────────────────────────────────────────────────────────────
0: popcount           F₂^128 → Z     ~640          ~128         5×        all accumulation
1: packed_inner_prod  F₂^128² → Z    ~5n           ~128         5×        matmul kernel
2: binary_matvec      F₂^{m×n} → Z^m m×5n          m×128        5×        inference, tri-kernel
3: quantize           F_p → F₂^k     ~k²           ~k           k×        F_p → F₂ boundary
4: dequantize         F₂^k → F_p     ~k²           ~k           k×        F₂ → F_p boundary
5: activation_lut     F₂^k → F₂^m   ~2^k/lookup   ~k/lookup    2^k/k×    activation functions
6: gadget_decompose   F_p → F₂^k     ~k²           ~k           k×        FHE bootstrapping
7: barrel_shift       F₂^n → F₂^n   ~n²           ~3n·log(n)   n/3log×   crypto, permutations
```

### constraint savings for primary workloads

```
workload: 4096×4096 quantized inference layer (BitNet 1-bit)
  naive F₂ patterns:    ~84M constraints
  with Bt jets:         ~1.84M constraints (binary_matvec jet)
  vs F_p (no Binius):   ~2.7B constraints
  total speedup:        ~1,400× over F_p

workload: tri-kernel SpMV (quantized 4-bit, 10^6 × 10^6 sparse)
  per iteration (1% density): ~10^6 × 128 × 4 = ~512M binary constraints (naive)
  with Bt jets:               ~10^6 × 449 × 4 = ~1.8M constraints
  vs F_p: ~16B constraints
  speedup: ~8,900× over F_p

workload: FHE gadget decomposition (n=1024 coefficients, 64-bit)
  naive: 1024 × 64² = ~4.2M constraints
  with gadget_decompose jet: 1024 × 64 = ~65K constraints
  speedup: ~64×
```

### prover efficiency (wall-clock)

constraint count understates the jet advantage. Binius prover operates on packed u128 words — 128 F₂ elements per machine operation. jets are designed to exploit packing:

```
popcount:           7 SIMD stages (vs 640 sequential)     ~90× prover speedup
packed_inner_prod:  7 SIMD stages (AND is free)            ~90× prover speedup
binary_matvec:      m × 7 SIMD stages (shared commitment)  ~90× prover speedup
```

constraint count × prover speedup = effective acceleration:

```
4096×4096 inference layer:
  constraint reduction: 46× (84M → 1.84M)
  prover speedup:       90× (SIMD packing)
  effective:            ~4,100× faster to prove than naive binary
  vs F_p prover:        ~1,400× × 90× = ~126,000× total advantage
```

## open questions

1. **popcount output width**: popcount of n bits produces a log₂(n)+1 bit result. this result typically crosses to F_p for accumulation (matrix row results are summed as integers). is the dequantize cost per row acceptable, or should popcount output directly to F_p?
2. **table size limit for activation_lut**: k=8 (256 entries) is practical. k=16 (65K entries) requires 65K constraints for table commitment — may need hierarchical decomposition
3. **multi-bit quantization**: quantize/dequantize handle arbitrary k. for 4-bit quantized models (GPTQ-style), k=4 per weight. should there be a specialized 4-bit jet?
4. **cross-algebra constraint accounting**: jets 3, 4, 6 span both F_p and F₂. how does the universal CCS handle constraints that reference both algebras? does each cross-algebra constraint count as one F_p constraint?

see [[jets]] for API and recognition mechanism, [[verifier-jets]] for Goldilocks jets, [[binius-pcs]] for PCS backend, [[zheng-2]] for cross-algebra composition

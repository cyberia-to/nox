---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: accepted
date: 2026-03-17
---
# verifier jets — Goldilocks/WHIR recursive composition

five jets that make recursive proof composition practical for nox<Goldilocks>. the unoptimized verifier costs ~600,000 patterns. with jets: ~70,000. this 8.5× reduction makes recursive composition practical.

## the five jets

### jet 0: hash

```
hash(x) → 8 × F_p (64-byte Hemera digest)
```

computes Hemera(x) — the Poseidon2-Goldilocks sponge over the input noun.

- pure equivalent: ~2,800 field ops (full Poseidon2 permutation as Layer 1 patterns)
- jet cost: 300
- stark constraints: ~300
- accelerates: Fiat-Shamir challenges, Merkle tree construction, content addressing

hash is simultaneously Layer 1 pattern 15 and Layer 3 jet 0. the pattern defines the semantics; the jet provides the optimized constraint layout.

### jet 1: poly_eval

```
poly_eval(coeffs, point) → F_p
```

Horner evaluation of a degree-N polynomial at a single point.

```
poly_eval([c_0, c_1, ..., c_N], z) = c_0 + c_1·z + c_2·z² + ... + c_N·z^N
```

- pure equivalent: ~2N patterns (N multiplications + N additions)
- jet cost: N
- stark constraints: ~N
- accelerates: WHIR query verification, constraint evaluation at random points

Horner's method is iterated FMA (fused multiply-accumulate), mapping directly to the nebu field's multiply-add sequence.

### jet 2: merkle_verify

```
merkle_verify(root, leaf, path, index) → {0, 1}
```

verify a Merkle authentication path of depth d. returns 0 if the path is valid (leaf hashes up to root), 1 otherwise.

```
for each level i from 0 to d-1:
  if bit i of index = 0:
    current = Hemera(current ‖ path[i])
  else:
    current = Hemera(path[i] ‖ current)
assert current = root
```

- pure equivalent: d × ~310 patterns (hash + conditional per level)
- jet cost: d × 300
- stark constraints: ~d × 300
- accelerates: stark proof checking (500K → 50K of unjetted verifier cost)

Merkle verification is the single largest cost in the unjetted verifier — 83% of total cost is hash operations for Merkle paths and Fiat-Shamir.

### jet 3: fri_fold

```
fri_fold(poly_layer, challenge) → poly_layer_next
```

one round of FRI folding: split the polynomial by parity of exponent, combine with the random challenge.

```
given f(x) = f_even(x²) + x·f_odd(x²)
fri_fold(f, α) = f_even + α·f_odd
```

- pure equivalent: ~N patterns (N/2 multiplications + N/2 additions + restructuring)
- jet cost: N/2
- stark constraints: ~N/2
- accelerates: WHIR verification (log(N) folding rounds per proof)

### jet 4: ntt

```
ntt(values, direction) → transformed values
```

Number Theoretic Transform (forward or inverse) over F_p. the algebraic analogue of FFT.

```
forward: coefficient representation → evaluation representation
inverse: evaluation representation → coefficient representation
```

uses the 2^32-th root of unity (1753635133440165772) from the Goldilocks field, provided by nebu.

- pure equivalent: ~2N·log(N) patterns (butterfly operations)
- jet cost: N·log(N)
- stark constraints: ~N·log(N)
- accelerates: polynomial multiplication, WHIR commitment computation, proof aggregation

## verifier cost analysis

```
Component               │ Layer 1 only │ With jets  │ Reduction
────────────────────────┼──────────────┼────────────┼──────────
Parse proof             │     ~1,000   │    ~1,000  │  1×
Fiat-Shamir challenges  │    ~30,000   │    ~5,000  │  6×
Merkle verification     │   ~500,000   │   ~50,000  │ 10×
Constraint evaluation   │    ~10,000   │    ~3,000  │  3×
WHIR verification       │    ~50,000   │   ~10,000  │  5×
────────────────────────┼──────────────┼────────────┼──────────
TOTAL                   │   ~600,000   │   ~70,000  │ ~8.5×
```

## hardware mapping

```
GFP primitive                    jets it accelerates
────────────────────────────     ─────────────────────────────────────
fma (field multiply-accumulate)  poly_eval (Horner = iterated FMA)
ntt (NTT butterfly)              ntt (direct correspondence)
p2r (Poseidon2 round)            hash, merkle_verify (hash-dominated)
lut (lookup table)               activation functions via Layer 1
```

## jet cost table

```
jet              │ exec cost    │ stark constraints │ pure Layer 1 cost
─────────────────┼──────────────┼───────────────────┼──────────────────
hash             │ 300          │ ~300              │ ~2,800
poly_eval(N)     │ N            │ ~N                │ ~2N
merkle_verify(d) │ d × 300      │ ~d × 300          │ d × ~310
fri_fold(N)      │ N/2          │ ~N/2              │ ~N
ntt(N)           │ N·log(N)     │ ~N·log(N)         │ ~2N·log(N)
```

## cost examples

```
Hemera hash: 300 (jet) or ~2800 (pure Layer 1)
  [15 [0 1]]
  jet cost: 300

Merkle verification (32 levels): ~9,600 (jet) or ~9,920 (pure Layer 1)
  merkle_verify(root, leaf, path, 32)
  jet cost: 32 × 300 = 9,600

stark verifier (one recursion level): ~70,000 (with jets)
  without jets: ~600,000 Layer 1 patterns

recursive composition (2 levels): ~140,000 (with jets)
  proof-of-proof: verify a proof that itself verified a proof
```

see [[jets]] for API and recognition mechanism, [[binary-jets]] for F₂ jets

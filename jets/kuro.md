---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: binary jets, kuro jets, F2 jets, Bt jets
---
# kuro jets — binary (F₂)

eight jets for nox<F₂> + Binius. quantized inference and tri-kernel SpMV at native binary cost. base operations (XOR, AND, NOT, SHL) already 1 constraint in F₂ — jets target composite operations that appear millions of times.

## jets

| # | name | input → output | naive | jet | speedup | primary workload |
|---|------|----------------|-------|-----|---------|------------------|
| 0 | popcount | F₂^128 → Z | ~640 | ~128 | 5× | all accumulation |
| 1 | packed_inner_product | F₂^128² → Z | ~5n | ~128 | 5× | matmul kernel |
| 2 | binary_matvec | F₂^{m×n} → Z^m | m×5n | m×128 | 5× | inference, tri-kernel |
| 3 | quantize | F_p → F₂^k | ~k² | ~k | k× | F_p → F₂ boundary |
| 4 | dequantize | F₂^k → F_p | ~k² | ~k | k× | F₂ → F_p boundary |
| 5 | activation_lut | F₂^k → F₂^m | ~2^k/lookup | ~k/lookup | 2^k/k× | activation functions |
| 6 | gadget_decompose | F_p → F₂^k | ~k² | ~k | k× | FHE bootstrapping |
| 7 | barrel_shift | F₂^n → F₂^n | ~n² | ~3n×log(n) | n/3log× | crypto, permutations |

## SIMD advantage

constraint count understates the jet advantage. Binius prover operates on packed u128 words — 128 F₂ elements per machine operation. popcount/packed_inner_product/binary_matvec achieve ~90× prover wall-clock speedup via SIMD packing on top of the constraint reduction.

## cross-algebra

quantize (jet 3) and dequantize (jet 4) handle the nebu ↔ kuro boundary. gadget_decompose (jet 6) handles the jali → kuro boundary for FHE bootstrapping.

## hardware mapping

- all jets → lut (lookup table) engine, SIMD packed
- quantize/dequantize → fma + lut (boundary conversion)

## PCS backend

PCS₂: Binius (binary Reed-Solomon over F₂ tower via kuro)

---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: verifier jets, recursion jets, proof composition jets
---
# recursion jets — F_p genesis jets

four genesis jets for nox<Goldilocks>. committed in genesis BBG state. make recursive proof composition and general-purpose polynomial computation practical.

hash is a separate genesis jet — see [[hash]].

## jets

| # | name | signature | exec cost | constraints | pure cost |
|---|------|-----------|-----------|-------------|-----------|
| 0 | poly_eval | poly_eval(coeffs, point) → F_p | N | ~N | ~2N |
| 1 | merkle_verify | merkle_verify(root, leaf, path, index) → {0,1} | d×200 | ~d×736 | d×~210 |
| 2 | fri_fold | fri_fold(poly_layer, challenge) → poly_layer_next | N/2 | ~N/2 | ~N |
| 3 | ntt | ntt(values, direction) → transformed values | N×log(N) | ~N×log(N) | ~2N×log(N) |

## role in current architecture

with Brakedown (Merkle-free PCS), the zheng verifier is pure field arithmetic. canonical verifier cost is ~825 constraints (CCS jet + batch) — see zheng/specs/verifier.md.

| jet | verifier role | general role |
|-----|--------------|--------------|
| poly_eval | Brakedown opening verification | any polynomial evaluation (Horner) |
| merkle_verify | not used by Brakedown verifier | cross-system interop, legacy content proofs |
| fri_fold | not used by Brakedown verifier | FRI-based protocols, cross-system interop |
| ntt | not used by Brakedown verifier | polynomial multiplication, ring operations |

poly_eval is verifier-critical. merkle_verify, fri_fold, and ntt are general-purpose computation jets retained for cross-system interoperability and domain-specific acceleration.

## hardware mapping

- poly_eval → fma (field multiply-accumulate, Horner = iterated FMA)
- merkle_verify → p2r (Poseidon2 round) via hash jet
- ntt → ntt (direct correspondence)
- fri_fold → ntt + fma

## PCS

Brakedown (expander-graph linear codes, Merkle-free) via Lens trait

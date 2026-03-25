# jets

optimization without compromise — from sixteen patterns to silicon, preserving meaning at every level.

## the problem

the [[stark]] verifier for nox is itself a nox program. to achieve recursive proof composition — proving that a proof is valid — the network runs the verifier inside the VM and proves THAT execution.

without optimization, the verifier costs ~600,000 Layer 1 patterns. at current proving speeds, this makes recursive composition impractical for block production. the verifier is dominated by three operations: hashing (~83% of cost), Merkle path verification, and polynomial evaluation. all three involve repeated field arithmetic — thousands of multiplications and additions in tight loops.

the sixteen patterns can express these operations. the patterns are Turing-complete. but expressing a Poseidon2 permutation as ~2,800 individual add/mul patterns produces ~2,800 rows in the trace, each requiring separate constraint verification. the proof is correct but enormous.

## the solution

five jets: optimized implementations of specific Layer 1 compositions.

```
hash             Poseidon2 permutation        ~2,800 patterns → 300 cost
poly_eval        Horner polynomial evaluation  ~2N patterns → N cost
merkle_verify    Merkle authentication path    ~310d patterns → 300d cost
fri_fold         FRI folding round             ~N patterns → N/2 cost
ntt              Number Theoretic Transform    ~2N·log(N) patterns → N·log(N) cost
```

each jet compresses many trace rows into fewer, more efficient constraint polynomials. the constraint logic is different (optimized), but the input-output behavior is identical.

## the semantic contract

every jet MUST have an equivalent pure Layer 1 expression producing identical output on all inputs.

this is not a guideline. it is a testable, enforceable invariant. the test suite runs both versions — jet and pure-pattern — on random inputs and checks equality. any divergence means the jet is buggy. the pure version is always the ground truth.

```
for all inputs x:
  jet_hash(x) == layer1_hash(x)
  jet_poly_eval(coeffs, point) == layer1_poly_eval(coeffs, point)
  jet_merkle_verify(root, leaf, path, index) == layer1_merkle_verify(root, leaf, path, index)
  jet_fri_fold(poly, challenge) == layer1_fri_fold(poly, challenge)
  jet_ntt(values, direction) == layer1_ntt(values, direction)
```

remove all jets → identical results, ~8.5× slower. this is the test of the contract: the system must function correctly with no jets at all.

## why these five

the selection is driven by one analysis: profile the stark verifier, find the bottlenecks, jet those.

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

Merkle verification alone accounts for 83% of the unjetted verifier cost, dominated by hash operations. the hash jet provides the largest single improvement. poly_eval and fri_fold target the WHIR protocol's polynomial operations. ntt handles polynomial multiplication for commitment and aggregation.

the jets are not general-purpose optimizations. they are surgical: five operations that together reduce the recursive verifier cost from "impractical" to "routine."

## from patterns to silicon

the five jets map to four [[Goldilocks field processor]] hardware primitives:

```
GFP primitive                    jets it accelerates
────────────────────────────     ─────────────────────────────────────
fma (field multiply-accumulate)  poly_eval (Horner = iterated FMA)
ntt (NTT butterfly)              ntt (direct correspondence)
p2r (Poseidon2 round)            hash, merkle_verify (hash-dominated)
lut (lookup table)               activation functions via Layer 1
```

the stack is continuous:

```
nox pattern    →  Layer 1         (semantics, ~2800 patterns for one hash)
software jet   →  Layer 3         (optimized constraint layout, 300 cost)
GFP primitive  →  hardware        (single-cycle Poseidon2 round)
```

the same computation at three speeds. identical results at every level. a nox program written today runs on pure Layer 1 patterns. when jets are available, it runs ~8.5× faster with the same semantics. when GFP hardware exists, the jets map directly to silicon primitives for another order-of-magnitude improvement.

this continuity is by design. the jet selection was guided by the hardware architecture, and the hardware architecture was guided by the jet requirements. they co-evolved to ensure that the optimization path from VM instruction to silicon gate has no semantic gaps.

## jets and proofs

jets change the constraint layout but preserve the input-output behavior. the [[stark]] proof system sees different constraint polynomials (more efficient ones) but verifies the same logical properties.

the prover can use jets — producing a proof with the optimized constraint layout. the verifier checks the same constraints either way. whether the prover used jets or pure patterns is invisible to the verifier. the proof says "this computation was correct," and correctness is defined by Layer 1 semantics regardless of which layer produced the trace.

this means jet adoption is purely a prover-side optimization. provers who use jets produce proofs faster and cheaper. provers who do not still produce valid proofs. the network does not require jets — it benefits from them. this is the meaning of "optimization without compromise."

## the Nock precedent

nox inherits the jet concept from [[Nock]]/[[Urbit]], where jets are called "arms" — optimized C implementations of computationally expensive Nock expressions. the principle is the same: semantic equivalence between the slow pure version and the fast optimized version.

nox's innovation is the selection criterion. Nock jets are chosen by practical utility (what Urbit applications need to be fast). nox jets are chosen by proof-system necessity (what the stark verifier needs to be fast). the difference reflects the systems' purposes: Nock powers a personal computing environment, nox powers a planetary verification machine.

## beyond the verifier: 33 jets across five algebras

the five verifier jets described above cover nebu (F_p). four more algebras contribute their own jet families:

- **kuro** (F₂): 8 binary jets — popcount, binary_matvec, quantize, dequantize, activation_lut, gadget_decompose, barrel_shift. 32× constraint reduction for bitwise workloads.
- **jali** (R_q): 5 ring jets — ntt_batch, key_switch, gadget_decomp, noise_track, blind_rotate. 3072× gap motivates ring-aware proving for FHE.
- **trop** (min,+): 5 tropical jets — trop_matmul, trop_shortest, trop_hungarian, trop_viterbi, trop_transport. optimization witnesses verified in F_p.
- **genies** (F_q): 4 isogeny jets — group_action, isogeny_walk, vrf_eval, vdf_step. privacy primitives over a foreign field.
- **decider**: 1 jet — all-history verification in 89 constraints, less than one hemera hash.

5 boundary jets handle algebra crossings (quantize, dequantize, gadget_decomp, secret_hash, witness_commit).

33 jets total. the same principle at every level: semantic equivalence, optimization without compromise.

see [[five-algebras]] for why five algebras are irreducible, [[decider]] for the 89-constraint all-history verification.

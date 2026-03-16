# the three layers

the ontological separation — truth, possibility, speed. nox is organized into three layers that are not implementation detail but a statement about the nature of computation.

## the architecture

```
┌────────────────────────────────────────────────────────────────────┐
│  LAYER 1: WHAT IS                                                  │
│  sixteen deterministic patterns                                    │
│  confluent, memoizable, parallel-safe                              │
│  both prover and verifier execute identically                      │
│                                                                    │
│  the ground truth of computation                                   │
├────────────────────────────────────────────────────────────────────┤
│  LAYER 2: WHAT MIGHT BE                                            │
│  one non-deterministic pattern: hint                               │
│  prover injects, Layer 1 constraints verify                        │
│  verifier never executes hint — checks via stark                   │
│                                                                    │
│  the origin of privacy and search                                  │
├────────────────────────────────────────────────────────────────────┤
│  LAYER 3: HOW FAST                                                 │
│  five jets: hash, poly_eval, merkle_verify, fri_fold, ntt          │
│  observationally equivalent to Layer 1 compositions                │
│  optimization without changing meaning                             │
│                                                                    │
│  the pragmatics of recursive verification                          │
└────────────────────────────────────────────────────────────────────┘
```

## Layer 1: what is

sixteen deterministic patterns define the irreducible semantics of computation. given the same object, formula, and focus, Layer 1 always produces the same result. the prover and verifier execute identically — there is no information asymmetry at this layer.

five structural patterns (axis, quote, compose, cons, branch) make the system Turing-complete. six field arithmetic patterns (add, sub, mul, inv, eq, lt) give it algebraic power native to the [[stark]] proof system. four bitwise patterns (xor, and, not, shl) handle binary data. one hash pattern (hash) provides cryptographic identity.

the patterns form an orthogonal rewrite system: unique tags, linear left-hand sides, no overlap. by the Huet-Levy theorem (1980), this guarantees confluence — the result is the same regardless of evaluation order. this is the mathematical foundation of content-addressed computation: if the result is independent of how you compute it, then `(H(object), H(formula))` is a canonical, permanent, universal identifier for that computation's result.

Layer 1 is the ground truth. if you stripped away Layers 2 and 3, you would still have a complete, deterministic, provable virtual machine. slower and less private, but functionally identical. everything builds on this foundation.

## Layer 2: what might be

one pattern: hint. the prover injects a value from outside the VM. Layer 1 constraints verify it. the verifier never executes hint — it checks the [[stark]] proof instead.

this single pattern is the entire mechanism of privacy, search, and oracle access. it is the boundary between the prover's knowledge and the verifier's knowledge. what the prover knows but the verifier does not crosses this boundary.

Layer 2 breaks confluence intentionally. two different provers may inject different valid witnesses for the same constraint. this is the non-determinism that makes [[zero knowledge proofs]] possible — the prover demonstrates knowledge without revealing it. soundness is preserved: any witness that fails the Layer 1 constraint check is rejected.

the ontological status of Layer 2 is "possibility." Layer 1 says "this IS the result." Layer 2 says "there EXISTS a witness such that the constraints are satisfied." the shift from deterministic to existential is what enables privacy — proving that you know something without revealing what you know.

without Layer 2, nox would be fully transparent. every computation would be publicly reproducible. there would be no private transactions, no ZK proofs, no ability to prove identity without revealing secrets. hint is the minimum necessary non-determinism — one instruction that opens the door to an entire cryptographic universe.

## Layer 3: how fast

five jets: hash, poly_eval, merkle_verify, fri_fold, ntt. each is observationally equivalent to a composition of Layer 1 patterns — same input, same output, different speed.

the selection criterion is specific: recursive [[stark]] verification. running the [[stark]] verifier inside nox (to produce a proof-of-proof) costs ~600,000 Layer 1 patterns without jets, ~70,000 with jets. this 8.5× reduction is what makes recursive proof composition practical.

Layer 3 preserves confluence. a jet is a semantic no-op — replacing it with its Layer 1 expansion produces identical results on all inputs. the test is automated: a harness runs both versions on random inputs and checks equality. if any jet ever disagrees with its pure equivalent, the jet is buggy and must be fixed or removed.

the ontological status of Layer 3 is "pragmatics." Layer 1 defines what is true. Layer 2 defines what is possible. Layer 3 defines what is fast. speed matters for engineering — recursive verification at 600K patterns per level is too expensive for practical block production. but speed does not affect meaning. remove all jets and every program still produces the same result.

## the removal test

the three layers are distinguished by what happens when you remove them:

remove Layer 3 (jets): every program still runs, every proof still verifies. the system is approximately 8.5× slower for recursive verification. no functionality is lost. this is why jets are "optimization without compromise."

remove Layer 2 (hint): the system loses privacy and search. every computation becomes fully transparent — no ZK proofs, no private transactions, no ability to prove without revealing. but the deterministic core remains complete. Layer 1 is still Turing-complete, still provable.

remove Layer 1 (patterns): nothing remains. the patterns ARE the computation. there is no nox without them.

this asymmetry reveals the architecture: Layer 1 is necessary and sufficient for computation. Layer 2 adds the qualitative capability of privacy. Layer 3 adds the quantitative improvement of speed. each layer is optional only if the layers below it remain.

## why this separation matters

in most VMs, optimization is entangled with semantics. a JIT compiler changes how code runs — and sometimes changes what it does (optimization bugs). a GPU kernel is semantically different from its CPU equivalent in edge cases. the boundary between "what the program means" and "how fast it runs" is blurry.

in nox, the boundary is clean. Layer 1 defines meaning. Layer 3 defines speed. they are connected by a testable contract: same input → same output. this contract can be verified by exhaustive testing, by proof, or both. if the contract holds, the jet is correct. if it fails, the jet is wrong. there is no middle ground.

this cleanliness propagates to the [[stark]] proof system. the proof verifier checks Layer 1 constraints — the semantics. jets change the constraint layout (more efficient polynomials for the same computation) but the constraints are equivalent. the proof says "this computation was correct" regardless of whether jets were used. the prover can use jets; the verifier checks the same constraints either way.

## the stack

```
nox pattern    → Layer 1 semantics     (what the computation means)
software jet   → Layer 3 optimization  (how the software runs it)
GFP primitive  → hardware acceleration (how the silicon runs it)
```

the three layers extend beyond software. the five jets map to four [[Goldilocks field processor]] hardware primitives: fma (fused multiply-accumulate), ntt (Number Theoretic Transform butterfly), p2r (Poseidon2 round), lut (lookup table). the same computation at three speeds — pattern, jet, silicon — with identical semantics at every level.

this continuity from VM instruction to hardware gate is the deepest expression of the three-layer architecture. the ontological separation (truth, possibility, speed) maps cleanly to the implementation hierarchy (pattern, jet, primitive). the structure of the ideas and the structure of the machine converge.

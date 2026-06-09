# hash pattern (15)


parameterized by H.

```
reduce(o, [15 a], f) →
  let (v_a, f1) = reduce(o, a, f - 25)
  (H(v_a), f1)
```

computes the identity of the evaluated operand. with polynomial data, this is hemera(Lens.commit(input_polynomial) ‖ domain_tag). result is a 4-element hash — 32 bytes, materialized as a pair-tree of four atoms `pair(pair(h0,h1), pair(h2,h3))`. there is no tag byte.

hash CAN be expressed as pure Layer 1 patterns (~1000 field ops for the Poseidon2 permutation with 24 rounds, x⁻¹ S-box in partial rounds). pattern 15 is simultaneously a Layer 1 pattern and the first Layer 3 jet. the jet accelerates; semantics unchanged.

cost: 25. multi-row pattern: 24 Poseidon2 round rows + 1 squeeze row, driven by `hemera::StepSponge`. row count = cost. constraints: ~736 zheng constraints (per-round S-box + MDS), specified in specs/trace.md §pattern 15.

see jets/hash.md for the hash jet specification.

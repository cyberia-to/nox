# hash pattern (15)


parameterized by H.

```
reduce(o, [15 a], f) →
  let (v_a, f1) = reduce(o, a, f - 200)
  (H(v_a), f1)
```

computes the identity of the evaluated operand. with polynomial nouns, this is hemera(Lens.commit(input_polynomial) ‖ domain_tag). result is a 4-element hash (32 bytes, type tag 0x02).

hash CAN be expressed as pure Layer 1 patterns (~1000 field ops for the Poseidon2 permutation with 24 rounds, x⁻¹ S-box in partial rounds). pattern 15 is simultaneously a Layer 1 pattern and the first Layer 3 jet. the jet accelerates; semantics unchanged.

cost: 200. constraints: ~736.

see jets/hash.md for the hash jet specification.

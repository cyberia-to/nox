# hash pattern (15)


parameterized by H.

```
reduce(o, [15 a], f) →
  let (v_a, f1) = reduce(o, a, f - 25)
  (H(v_a), f1)
```

computes the identity of the evaluated operand. with polynomial data, this is hemera(Lens.commit(input_polynomial) ‖ domain_tag). result is a 4-element hash — 32 bytes, materialized as a pair-tree of four atoms `pair(pair(h0,h1), pair(h2,h3))`. there is no tag byte.

hash CAN be expressed as pure Layer 1 patterns (~3,000 field ops for the Poseidon2 permutation of hemera 0.4: 60 rounds, x⁷ S-box in every round, 1,728 multiplications). pattern 15 is simultaneously a Layer 1 pattern and the first Layer 3 jet. the jet accelerates; semantics unchanged.

cost: 25. multi-row pattern: 24 Poseidon2 round rows + 1 squeeze row under hemera 0.3, driven by `hemera::StepSponge`; hemera 0.4 (frozen 2026-10-16) has 60 rounds, so 61 rows — the budget decision is pending (hemera `audit/profile-0.4-2026-10.md` §5). row count = cost. constraints: ~1,440 zheng constraints (per-round S-box + MDS), specified in specs/trace.md §pattern 15.

see jets/hash.md for the hash jet specification.

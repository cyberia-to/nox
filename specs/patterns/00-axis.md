# pattern 0: axis

algebra-independent.

```
reduce(o, [0 a], f) = (axis(s, eval(a)), f - 1)
```

the evaluated axis index must be a field-type or word-type atom, interpreted as an integer. if eval(a) produces a cell or hash-type atom → ⊥_error.

with polynomial nouns, axis is O(1) via Lens opening: the binary encoding of the axis address is the evaluation point. this replaces O(depth) tree traversal with a single polynomial evaluation.

cost: 1. constraints: 1 (budget) + 4 (commitment binding, when lens hints provided).

## register layout

```
r0  = 0
r1  = object NounId
r2  = formula NounId
r3  = result NounId               — NounId of the value at the addressed position
r4  = object NounId               — repeated for Lens-opening binding in zheng
r5  = axis address                — evaluated axis address as raw u64
r6  = depth traversed             — number of tree levels descended (0 for addr ≤ 1)
r7  = result NounId               — NounId of the value at the addressed position
r8  = budget_in
r9  = budget_out                  — r8 - 1
r10 = 0 (success) / error kind
r11 = commitment_bytes[0..8]      — first  8 bytes of Lens commitment (LE u64); 0 if hints absent
r12 = commitment_bytes[8..16]     — second 8 bytes
r13 = commitment_bytes[16..24]    — third  8 bytes
r14 = commitment_bytes[24..32]    — fourth 8 bytes
r15 = 0 (reserved)
```

r11-r14 are populated when the executor's CallProvider implements axis_commitment().
when absent (NullCalls, interpreter mode) r11-r14 are zero; zheng skips commitment binding.

## constraints (zheng)

inline (circuit-level, always):
  r9 = r8 - 1  (budget decrement, degree 1)

commitment binding (when r11-r14 ≠ 0, via axis_acc fold):
  verifier_steps(commitment_from_r11_r14, eval_point, r7_value, opening) all satisfied

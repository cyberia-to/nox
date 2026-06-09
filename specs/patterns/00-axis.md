# pattern 0: axis

algebra-independent.

```
reduce(o, [0 a], f) = (axis(s, a), f - 1)
```

the address `a` is a literal atom in the formula structure — it is NOT reduced before use. axis reads the address directly from the formula data body. the address must be a field-type or word-type atom, interpreted as an integer. if `a` is a pair or hash-type atom → ⊥_error.

with polynomial data, axis is O(1) via Lens opening: the binary encoding of the axis address is the evaluation point. this replaces O(depth) tree traversal with a single polynomial evaluation.

cost: 1. constraints: 1 (budget) + 4 (commitment binding, when lens hints provided).

## register layout

```
r0  = 0
r1  = object particle
r2  = formula particle
r3  = result particle               — particle of the value at the addressed position
r4  = commitment[0] from axis_commitment() when prover active, 0 otherwise
      (equals r11: first 8-byte LE limb of the 32-byte Lens commitment)
r5  = axis address (raw u64)      — literal address from formula body
r6  = levels descended            — number of tree levels descended (0 for addr ≤ 1)
r7  = result particle               — particle of the value at the addressed position
r8  = budget_in
r9  = budget_out                  — r8 - 1
r10 = 0 (success) / error kind
r11 = commitment[0] (bytes 0..8, LE u64)   — same as r4; 0 if hints absent
r12 = commitment[1] (bytes 8..16)
r13 = commitment[2] (bytes 16..24)
r14 = commitment[3] (bytes 24..32)
r15 = 0 (reserved)
```

r4 and r11-r14 are populated when the executor's CallProvider implements axis_commitment().
when absent (NullCalls, interpreter mode) r4 and r11-r14 are zero; zheng skips commitment binding.

## constraints (zheng)

inline (circuit-level, always):
  r9 = r8 - 1  (budget decrement, degree 1)

commitment binding (when r11-r14 ≠ 0, via axis_acc fold):
  verifier_steps(commitment_from_r11_r14, eval_point, r7_value, opening) all satisfied

# pattern 11: xor


parameterized by W. valid on word type only. bitwise on hash → ⊥_error.

```
abstract:   xor_W(a, b) → bitwise exclusive-or over W bits
canonical:  v_a ⊕ v_b (32-bit XOR)
```

cost: 1. constraints: ~32 (bit decomposition in F_p). in F₂: 1 constraint.

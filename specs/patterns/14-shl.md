# pattern 14: shl


parameterized by W. valid on word type only.

```
abstract:   shl_W(a, n) → left shift over W bits, n must be in [0, W)
canonical:  (v_a << v_n) mod 2^32, shifts ≥ 32 produce 0
```

right shift is expressible as `shl(a, W-n)` followed by `and` with a mask.

cost: 1. constraints: ~32.

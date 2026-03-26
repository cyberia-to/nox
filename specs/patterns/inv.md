# pattern 8: inv

rs module: patterns/inv.rs

parameterized by F.

```
abstract:   inv_F(a) → a⁻¹ in F, or ⊥_error if a = 0
canonical:  v_a^(p-2) mod p    (Fermat's little theorem)
```

execution cost: 64 (reflects ~64 multiplications in square-and-multiply for Goldilocks).
stark verification cost: 1 constraint (verifier checks a × a⁻¹ = 1).

the asymmetry between execution cost and verification cost is fundamental: inversion is expensive to compute but cheap to verify.

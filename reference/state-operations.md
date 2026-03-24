# state operations

version: 0.1
status: canonical

## overview

state operations are patterns for modifying [[BBG]] polynomial state. they are NOT a separate instruction set — they are compositions of the 16 nox patterns (primarily field arithmetic: add, mul, eq) applied to polynomial evaluation points. state jets optimize their proof encoding.

five irreducible operations. three groups. derived from CCS structure.

## derivation from CCS

a CCS satisfaction equation:

$$\sum_j c_j \cdot \bigodot_{i \in S_j} M_i \cdot z = 0$$

decomposes into five nox-native activities:
- z contains state values → accessed by **READ** / **WRITE** (polynomial evaluation = field ops)
- $M_i \cdot z$ computes linear combinations → **ADD** (pattern 5)
- $\bigodot$ computes Hadamard products → **MUL** (pattern 7)
- $= 0$ asserts equality → **ASSERT_EQ** (pattern 9)

these are nox patterns operating on polynomial state. not new instructions.

## the five operations

### group 1: state access

**READ(dimension, key) → value**

evaluate BBG_poly at a point. implemented as: nox field operations that compute the polynomial evaluation, producing a field element result.

```
nox decomposition:  poly_eval jet over BBG_poly evaluation table
constraints:        1 (PCS evaluation binding)
```

**WRITE(dimension, key, value)**

update BBG_poly at an evaluation point. implemented as: mark the point dirty, store the new value, include in batch recommit.

```
nox decomposition:  field assignment to evaluation table
constraints:        1 (updated polynomial binding)
```

### group 2: verification

**ASSERT_EQ(a, b)**

assert two field values are equal. IS pattern 9 (eq).

```
nox decomposition:  [9 [a b]] → 0 (equal) or 1 (not equal)
constraints:        1
```

### group 3: arithmetic

**ADD(a, b) → c**

field addition. IS pattern 5 (add).

```
nox decomposition:  [5 [a b]] → a + b mod p
constraints:        1
```

**MUL(a, b) → c**

field multiplication. IS pattern 7 (mul).

```
nox decomposition:  [7 [a b]] → a × b mod p
constraints:        1
```

## irreducibility

| removed | what breaks |
|---|---|
| READ | can't access polynomial state |
| WRITE | can't modify polynomial state |
| ASSERT_EQ | can't verify any constraint |
| ADD | can't compute sums (ring incomplete) |
| MUL | can't compute products (ring incomplete) |

## derived operations

compositions of the five primitives. these are the natural targets for state jets:

| operation | decomposition | constraints |
|---|---|---|
| EXTEND(dim, key, val) | READ(=0) + WRITE | 3 |
| AUTHORIZE(caller, owner) | READ + ASSERT_EQ | 2 |
| ASSERT_NEQ(a, 0) | MUL(a, witness) + ASSERT_EQ(=1) | 2 |
| RANGE(val, 0, 2^k) | k × (MUL + ASSERT_EQ) + ADD | 2k+1 |
| CONSERVE(ins, outs) | ADD chain + ASSERT_EQ | n |

HASH is not a state operation — it is nox pattern 15. hash results enter state transitions as proven values from the zheng proof.

see [[jets]] for state jets (the third jet category), [[BBG]] architecture for polynomial state, [[patterns]] for the 16 base patterns

# state operations

version: 0.1
status: canonical

## overview

state operations are patterns for modifying [[BBG]] polynomial state. they are NOT a separate instruction set — they are compositions of the 16 nox patterns (primarily field arithmetic: add, mul, eq) applied to polynomial evaluation points. state jets optimize their proof encoding.

five named compositions of the 16 nox patterns. not new instructions — abbreviations for common pattern sequences applied to the polynomial evaluation table noun.

## derivation from the 16 patterns

the [[BBG]] polynomial evaluation table is a NOUN — a binary tree of [[Goldilocks field]] elements. state operations are standard nox patterns operating on this noun:

| state operation | nox patterns used | what it does |
|---|---|---|
| READ | Lens.open (polynomial evaluation) | O(1) evaluation at (dimension, key) via Lens opening |
| WRITE | pattern 0 (axis) + pattern 3 (cons) | navigate to position, build updated tree |
| ASSERT_EQ | pattern 9 (eq) | check two field values are equal |
| ADD | pattern 5 (add) | field addition on state values |
| MUL | pattern 7 (mul) | field multiplication on state values |

Lens verification (proving READ is correct) decomposes into: field arithmetic (patterns 5-8) + hemera hash (pattern 15). also from the 16.

**16 patterns are the axiom. state operations are derived. no new instructions.**

## the five operations

### group 1: state access

**READ(dimension, key) → value**

O(1) polynomial evaluation via Lens opening. with polynomial nouns, the evaluation table is a multilinear polynomial — READ evaluates it at a binary point corresponding to (dimension, key). the Lens opening proof (~75 bytes) certifies the result. this is direct polynomial evaluation, not axis-on-evaluation-table-noun (tree walk).

```
nox decomposition:  Lens.open(BBG_poly, (dimension, key)) → value + proof
constraints:        1 (Lens evaluation binding)
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

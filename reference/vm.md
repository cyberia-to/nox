# nox virtual machine specification

version: 0.1
status: canonical

## overview

nox is a proof-native virtual machine. sixteen deterministic reduction patterns over the Goldilocks field (Layer 1), one non-deterministic witness injection pattern (Layer 2), five jets for efficient recursive stark verification (Layer 3).

every nox execution produces a trace that IS the stark witness. there is no separate arithmetization step.

## field

```
p = 2^64 - 2^32 + 1 = 18446744069414584321
primitive root: 7
2^32-th root of unity: 1753635133440165772
reduction: a mod p = a_lo - a_hi × (2^32 - 1) + correction
```

## nouns

everything is a noun.

```
noun = atom(F_p)
     | cell(noun, noun)
```

atom: single Goldilocks field element.
cell: ordered pair of two nouns (binary tree).

### type tags

```
0x00  field   [0, p)         arithmetic operations
0x01  word    [0, 2^64)      bitwise operations
0x02  hash    4 × F_p        identity (256-bit Hemera digest)
```

coercion: field → word always valid (Goldilocks fits in u64). word → field always valid. hash → field extracts first element (lossy). field → hash forbidden.

type errors: bitwise on hash → error. arithmetic on hash (except eq) → error.

## reduction

```
reduce(subject, formula, focus) → (result, focus') | halt | error
```

subject: environment (data). formula: code (noun of form [tag body]). focus: resource budget (decremented per pattern).

## structural hash

```
H(atom a)     = Hemera(0x00 ‖ type_tag(a) ‖ encode(a))
H(cell(l, r)) = Hemera(0x01 ‖ H(l) ‖ H(r))
```

domain separation constants:
```
COMMITMENT = 0x4E4F582020524543   "NOX  REC"
NULLIFIER  = 0x4E4F5820204E554C   "NOX  NUL"
MERKLE     = 0x4E4F5820204D524B   "NOX  MRK"
OWNER      = 0x4E4F5820204F574E   "NOX  OWN"
```

## Layer 1: sixteen deterministic patterns

### structural (0-4)

```
0  axis    reduce(s, [0 a], f) = (axis(s, eval(a)), f - 1 - depth)
             axis(s, 0)   = H(s)
             axis(s, 1)   = s
             axis(s, 2)   = head(s)
             axis(s, 3)   = tail(s)
             axis(s, 2n)  = axis(axis(s,n), 2)
             axis(s, 2n+1)= axis(axis(s,n), 3)

1  quote   reduce(s, [1 c], f) = (c, f - 1)

2  compose reduce(s, [2 [x y]], f) =
             let (rx, f1) = reduce(s, x, f - 2)
             let (ry, f2) = reduce(s, y, f1)
             reduce(rx, ry, f2)

3  cons    reduce(s, [3 [a b]], f) =
             let (ra, f1) = reduce(s, a, f - 2)
             let (rb, f2) = reduce(s, b, f1)
             (cell(ra, rb), f2)

4  branch  reduce(s, [4 [test [yes no]]], f) =
             let (t, f1) = reduce(s, test, f - 2)
             if t = 0 then reduce(s, yes, f1)
                      else reduce(s, no, f1)
```

parallel: patterns 2, 3 evaluate both sub-expressions independently. pattern 4 evaluates only ONE branch (lazy).

### field arithmetic (5-10)

```
5  add     (a + b) mod p                    cost: 1
6  sub     (a - b) mod p                    cost: 1
7  mul     (a × b) mod p                    cost: 1
8  inv     a^(p-2) mod p  (error if a = 0)  cost: 64
9  eq      0 if a = b, else 1               cost: 1
10 lt      0 if a < b, else 1               cost: 1
```

all binary patterns: reduce(s, [op [a b]], f) evaluates both operands (parallelizable), then applies op.

### bitwise (11-14)

```
11 xor     a ⊕ b                            cost: 1
12 and     a ∧ b                            cost: 1
13 not     a ⊕ (2^64 - 1)                  cost: 1
14 shl     (a << n) mod 2^64               cost: 1
```

valid on word type only. bitwise on hash → error.

### hash (15)

```
15 hash    H(a) → 4 × F_p (256-bit digest)  cost: 300
```

## Layer 2: hint (pattern 16)

```
16 hint    reduce(s, [16 constraint], f) =
             let (check, f1) = reduce(s, constraint, f - 1)
             let w = PROVER_INJECT()
             assert check(w) = 0
             (w, f1)
```

prover injects witness value. Layer 1 constraint verifies it. verifier checks via stark proof, never executes hint directly.

hint is NOT memoizable. pure sub-expressions within hint-containing computations remain memoizable.

## Layer 3: jets

five operations semantically equivalent to Layer 1 compositions, optimized for the stark verifier bottleneck.

```
jet 0: hash(x)                        → 4 × F_p          cost: 300
jet 1: poly_eval(coeffs, point)       → F_p               cost: N
jet 2: merkle_verify(root,leaf,path,i)→ {0,1}             cost: d × 300
jet 3: fri_fold(poly_layer, challenge)→ poly_layer_next    cost: N/2
jet 4: ntt(values, direction)         → transformed        cost: N·log(N)
```

jet semantic contract: every jet MUST have an equivalent pure Layer 1 expression producing identical output on all inputs. remove all jets → identical results, ~8.5× slower.

## cost table

```
pattern     exec cost       stark constraints
─────────   ──────────────  ─────────────────
0  axis     1 + depth       ~depth
1  quote    1               1
2  compose  2               2
3  cons     2               2
4  branch   2               2
5  add      1               1
6  sub      1               1
7  mul      1               1
8  inv      64              1
9  eq       1               1
10 lt       1               ~64
11-14 bit   1               ~64 each
15 hash     300             ~300
16 hint     1 + constraint  constraint rows
```

## confluence

Layer 1 patterns form an orthogonal rewrite system (unique tags, linear left-hand sides, non-overlapping). by Huet-Levy (1980), confluent without requiring termination. parallel, lazy, and eager reduction all produce identical results.

consequence: content-addressed memoization is sound.
```
cache key:   (H(subject), H(formula))
cache value: H(result)
```

## test vectors

```
add(1, 2) = 3
mul(p-1, p-1) = 1
inv(2) = 9223372034707292161

reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 96)
```

## stark integration

the execution trace (register states per step) IS the AIR witness.
- pattern tag → constraint selector
- pattern semantics → transition constraint polynomial
- 16 register columns × 2^n rows

trace encodes as one multilinear polynomial. WHIR commits it. SuperSpartan sumcheck verifies. output: stark proof (~60-157 KiB).

see zheng for proof system details.

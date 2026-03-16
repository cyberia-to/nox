# pattern specification

version: 0.1
status: canonical

## overview

seventeen patterns: sixteen deterministic (Layer 1), one non-deterministic (Layer 2). four bits index the Layer 1 patterns (0-15). pattern 16 (hint) is Layer 2.

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                       LAYER 1: REDUCTION PATTERNS                         ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  STRUCTURAL (5)              FIELD ARITHMETIC (6)                         ║
║  0: axis — navigate          5: add — (a + b) mod p                       ║
║  1: quote — literal          6: sub — (a - b) mod p                       ║
║  2: compose — recursion      7: mul — (a × b) mod p                       ║
║  3: cons — build cell        8: inv — a^(p-2) mod p                       ║
║  4: branch — conditional     9: eq  — equality test                       ║
║                              10: lt — less-than                           ║
║                                                                           ║
║  BITWISE (4)                 HASH (1)                                     ║
║  11: xor    12: and          15: hash — structural H(x)                   ║
║  13: not    14: shl                                                       ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  LAYER 2                                                                  ║
║  16: hint — non-deterministic witness injection                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

## structural patterns (0-4)

### pattern 0: axis

```
reduce(s, [0 a], f) = (axis(s, eval(a)), f - depth)

  axis(s, 0)   = H(s)           ; hash introspection
  axis(s, 1)   = s              ; identity
  axis(s, 2)   = head(s)        ; left child (⊥_error if atom)
  axis(s, 3)   = tail(s)        ; right child (⊥_error if atom)
  axis(s, 2n)  = axis(axis(s,n), 2)
  axis(s, 2n+1)= axis(axis(s,n), 3)
```

the evaluated axis index must be a field-type or word-type atom, interpreted as an integer. if eval(a) produces a cell or hash-type atom → ⊥_error.

cost: depth (number of tree traversal steps). axis 0 and 1 cost 1. axis 2 and 3 cost 1. axis 4-7 cost 2. stark constraints: ~depth.

### pattern 1: quote

```
reduce(s, [1 c], f) = (c, f - 1)
```

returns c literally, unevaluated. the only pattern that ignores the object.

cost: 1. stark constraints: 1.

### pattern 2: compose

```
reduce(s, [2 [x y]], f) =
  let (rx, f1) = reduce(s, x, f - 2)
  let (ry, f2) = reduce(s, y, f1)
  reduce(rx, ry, f2)
```

evaluate x to get a new object, evaluate y to get a formula, then apply. this is the recursion mechanism — all control flow, looping, and function application reduce to compose.

PARALLELISM: reduce(s,x) and reduce(s,y) are INDEPENDENT — safe to evaluate in parallel.

cost: 2. stark constraints: 2.

### pattern 3: cons

```
reduce(s, [3 [a b]], f) =
  let (ra, f1) = reduce(s, a, f - 2)
  let (rb, f2) = reduce(s, b, f1)
  (cell(ra, rb), f2)
```

build a cell from two evaluated sub-expressions. the data construction primitive.

PARALLELISM: reduce(s,a) and reduce(s,b) are INDEPENDENT.

cost: 2. stark constraints: 2.

### pattern 4: branch

```
reduce(s, [4 [test [yes no]]], f) =
  let (t, f1) = reduce(s, test, f - 2)
  if t = 0 then reduce(s, yes, f1)
           else reduce(s, no, f1)
```

CRITICAL: only ONE branch is evaluated. this is lazy evaluation — prevents infinite recursion DoS where both branches would diverge. the untaken branch is never touched.

NOT parallel — must evaluate test before choosing a branch.

cost: 2. stark constraints: 2.

## field arithmetic patterns (5-10)

all binary arithmetic patterns follow the same structure: evaluate both operands (parallelizable), then apply the operation.

```
reduce(s, [op [a b]], f) =
  let (v_a, f1) = reduce(s, a, f - cost)
  let (v_b, f2) = reduce(s, b, f1)
  (op(v_a, v_b), f2)
```

### pattern 5: add

```
reduce(s, [5 [a b]], f) → ((v_a + v_b) mod p, f2)
```

Goldilocks addition. provided by aurum. cost: 1. stark constraints: 1.

### pattern 6: sub

```
reduce(s, [6 [a b]], f) → ((v_a - v_b) mod p, f2)
```

Goldilocks subtraction. provided by aurum. cost: 1. stark constraints: 1.

### pattern 7: mul

```
reduce(s, [7 [a b]], f) → ((v_a × v_b) mod p, f2)
```

Goldilocks multiplication. provided by aurum. cost: 1. stark constraints: 1.

### pattern 8: inv

```
reduce(s, [8 a], f) →
  let (v_a, f1) = reduce(s, a, f - 64)
  if v_a = 0 then ⊥_error
  (v_a^(p-2) mod p, f1)
```

Goldilocks field inverse via Fermat's little theorem. provided by aurum.

execution cost: 64 (reflects ~64 multiplications in square-and-multiply).
stark verification cost: 1 constraint (verifier checks a × a⁻¹ = 1).

the asymmetry between execution cost and verification cost is fundamental: inversion is expensive to compute but cheap to verify.

### pattern 9: eq

```
reduce(s, [9 [a b]], f) → (0 if v_a = v_b else 1, f2)
```

equality test across all types (field, word, hash). returns 0 for true (consistent with branch: 0 = take yes-branch).

cost: 1. stark constraints: 1.

### pattern 10: lt

```
reduce(s, [10 [a b]], f) → (0 if v_a < v_b else 1, f2)
```

less-than comparison. on field elements, compares canonical representatives.

cost: 1. stark constraints: ~64 (range decomposition for non-native comparison).

## bitwise patterns (11-14)

valid on word type [0, 2^64) only. bitwise on hash → ⊥_error.

stark constraints: ~64 each (bit decomposition required for algebraic verification).

### pattern 11: xor

```
reduce(s, [11 [a b]], f) → (v_a ⊕ v_b, f2)
```

cost: 1.

### pattern 12: and

```
reduce(s, [12 [a b]], f) → (v_a ∧ v_b, f2)
```

cost: 1.

### pattern 13: not

```
reduce(s, [13 a], f) → (v_a ⊕ (2^64 - 1), f1)
```

bitwise complement. unary — single operand.

cost: 1.

### pattern 14: shl

```
reduce(s, [14 [a n]], f) → ((v_a << v_n) mod 2^64, f2)
```

left shift. right shift is expressible as `shl(a, 64-n)` followed by `and` with a mask.

cost: 1.

## hash pattern (15)

```
reduce(s, [15 a], f) →
  let (v_a, f1) = reduce(s, a, f - 300)
  (H(v_a), f1)
```

computes the structural hash of the evaluated operand using Hemera. result is an 8-element hash (64 bytes, type tag 0x02).

hash CAN be expressed as pure Layer 1 patterns (~2800 field ops for the Poseidon2 permutation). pattern 15 is simultaneously a Layer 1 pattern and the first Layer 3 jet. the jet accelerates; semantics unchanged.

cost: 300. stark constraints: ~300.

## hint pattern (16) — Layer 2

```
reduce(s, [16 constraint], f) =
  let (check, f1) = reduce(s, constraint, f - 1)
  let w = PROVER_INJECT()
  assert check(w) = 0          — Layer 1 verifies the constraint
  (w, f1)
```

the single non-deterministic pattern. the prover injects a witness value from outside the VM. Layer 1 constraints verify it. the verifier NEVER executes hint directly — it checks constraint satisfaction via the stark proof.

```
PROVER_INJECT: → Noun
  source:   external to the VM, prover-only
  verifier: checks via stark (multilinear trace + sumcheck)
  cost:     1 + cost(constraint). witness search is external
  memo:     NOT memoizable (different provers may inject different valid witnesses)
```

hint is the entire mechanism of privacy, search, and oracle access:

```
identity:         hint injects the secret behind a neuron address
                  Layer 1 checks: Hemera(secret) = address

private transfer: hint injects record details (owner, value, nonce)
                  Layer 1 checks: conservation, ownership, nullifier freshness

AI inference:     hint injects neural network weights
                  Layer 1 checks: forward pass produces claimed output

optimization:     hint injects an optimal solution
                  Layer 1 checks: solution satisfies constraints AND is optimal
```

## cost table

```
Layer │ Pattern      │ Exec Cost      │ stark Constraints
──────┼──────────────┼────────────────┼───────────────────
  1   │ 0 axis       │ depth           │ ~depth
  1   │ 1 quote      │ 1              │ 1
  1   │ 2 compose    │ 2              │ 2
  1   │ 3 cons       │ 2              │ 2
  1   │ 4 branch     │ 2              │ 2
  1   │ 5 add        │ 1              │ 1
  1   │ 6 sub        │ 1              │ 1
  1   │ 7 mul        │ 1              │ 1
  1   │ 8 inv        │ 64             │ 1
  1   │ 9 eq         │ 1              │ 1
  1   │ 10 lt        │ 1              │ ~64
  1   │ 11-14 bit    │ 1              │ ~64 each
  1   │ 15 hash      │ 300            │ ~300
  2   │ 16 hint      │ 1 + constraint │ constraint rows
```

## test vectors

```
add(1, 2) = 3
mul(p-1, p-1) = 1
inv(2) = 9223372034707292161
inv(0) = ⊥_error

reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 96)
  // object = cell(1,2)
  // formula = add(axis 2, axis 3) = add(1, 2) = 3
  // cost: add(1) + axis(1) + axis(1) + dispatch(1) = 4

reduce(42, [1 7], 10) = (7, 9)
  // quote returns 7 literally, ignoring object 42

reduce([1,2], [3 [[0 2] [0 3]]], 100) = (cell(1, 2), 96)
  // cons(axis 2, axis 3) = cons(1, 2)

reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100)
  = (200, 95)
  // branch: eq(1,2)=1 (false), take no-branch → quote 200
```

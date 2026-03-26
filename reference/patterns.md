# pattern specification

version: 0.2
status: canonical

## overview

seventeen patterns: sixteen deterministic (Layer 1), one non-deterministic (Layer 2). four bits index the Layer 1 patterns (0-15). pattern 16 (hint) is Layer 2.

the 16 deterministic patterns are algebra-polymorphic — parameterized by field F, word width W, and hash function H. structural patterns (0-4) are algebra-independent. field patterns (5-10) are parameterized by F. bitwise patterns (11-14) are parameterized by W. hash (15) is parameterized by H. see vm.md for the instantiation model.

all concrete costs and constraint counts below refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                       LAYER 1: REDUCTION PATTERNS                         ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  STRUCTURAL (5)              FIELD ARITHMETIC (6)                         ║
║  0: axis — navigate          5: add — F-addition                          ║
║  1: quote — literal          6: sub — F-subtraction                       ║
║  2: compose — recursion      7: mul — F-multiplication                    ║
║  3: cons — build cell        8: inv — F-inverse                           ║
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

## structural patterns (0-4) — algebra-independent

structural patterns operate on the tree structure of nouns. they work identically over any leaf type, any field, any instantiation.

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

with polynomial nouns (see nouns.md polynomial representation), axis is O(1) via Lens opening: the binary encoding of the axis address is the evaluation point, and the Lens opening proof (~75 bytes) certifies the result. this replaces O(depth) tree traversal with a single polynomial evaluation. the semantic definition (recursive tree walk) is unchanged — the Lens evaluation computes the same value.

cost: 1 (polynomial evaluation via Lens opening). stark constraints: 1 (Lens evaluation binding). legacy cost model (tree traversal): depth.

### pattern 1: quote

```
reduce(s, [1 c], f) = (c, f - 1)
```

returns c literally, unevaluated. the only pattern that ignores the object.

cost: 1. stark constraints: 1.

### pattern 2: compose

```
reduce(s, [2 [x y]], f) =
  let (rx, f1) = reduce(s, x, f - 1)
  let (ry, f2) = reduce(s, y, f1)
  reduce(rx, ry, f2)
```

evaluate x to get a new object, evaluate y to get a formula, then apply. this is the recursion mechanism — all control flow, looping, and function application reduce to compose.

PARALLELISM: reduce(s,x) and reduce(s,y) are INDEPENDENT — safe to evaluate in parallel.

cost: 1. stark constraints: 1.

### pattern 3: cons

```
reduce(s, [3 [a b]], f) =
  let (ra, f1) = reduce(s, a, f - 1)
  let (rb, f2) = reduce(s, b, f1)
  (cell(ra, rb), f2)
```

build a cell from two evaluated sub-expressions. the data construction primitive.

PARALLELISM: reduce(s,a) and reduce(s,b) are INDEPENDENT.

cost: 1. stark constraints: 1.

### pattern 4: branch

```
reduce(s, [4 [test [yes no]]], f) =
  let (t, f1) = reduce(s, test, f - 1)
  if t = 0 then reduce(s, yes, f1)
           else reduce(s, no, f1)
```

CRITICAL: only ONE branch is evaluated. this is lazy evaluation — prevents infinite recursion DoS where both branches would diverge. the untaken branch is never touched.

NOT parallel — must evaluate test before choosing a branch.

cost: 1. stark constraints: 1.

## field arithmetic patterns (5-10) — parameterized by F

field patterns compute over the instantiated field F. the abstract semantics are universal: add computes F-addition, mul computes F-multiplication, inv computes F-inverse. the concrete reduction rules, costs, and constraint counts depend on F.

all binary arithmetic patterns follow the same structure: evaluate both operands (parallelizable), then apply the operation.

```
reduce(s, [op [a b]], f) =
  let (v_a, f1) = reduce(s, a, f - 1)
  let (v_b, f2) = reduce(s, b, f1)
  (op_F(v_a, v_b), f2)
```

### pattern 5: add

```
abstract:   add_F(a, b) → a + b in F
canonical:  (v_a + v_b) mod p              Goldilocks addition, provided by nebu
```

cost: 1. stark constraints: 1.

### pattern 6: sub

```
abstract:   sub_F(a, b) → a - b in F
canonical:  (v_a - v_b) mod p              Goldilocks subtraction, provided by nebu
```

cost: 1. stark constraints: 1.

### pattern 7: mul

```
abstract:   mul_F(a, b) → a × b in F
canonical:  (v_a × v_b) mod p              Goldilocks multiplication, provided by nebu
```

cost: 1. stark constraints: 1.

### pattern 8: inv

```
abstract:   inv_F(a) → a⁻¹ in F, or ⊥_error if a = 0
canonical:  v_a^(p-2) mod p                Fermat's little theorem, provided by nebu
```

execution cost: 64 (reflects ~64 multiplications in square-and-multiply for Goldilocks).
stark verification cost: 1 constraint (verifier checks a × a⁻¹ = 1).

the asymmetry between execution cost and verification cost is fundamental: inversion is expensive to compute but cheap to verify. the execution cost is per-instantiation (different fields have different inversion algorithms). the verification cost (1 constraint) is universal.

### pattern 9: eq

```
abstract:   eq(a, b) → 0 if a = b in F, else 1
canonical:  0 if v_a = v_b else 1
```

equality test across all types (field, word, hash). returns 0 for true (consistent with branch: 0 = take yes-branch).

cost: 1. stark constraints: 1.

### pattern 10: lt

```
abstract:   lt(a, b) → 0 if a < b under F's canonical ordering, else 1
canonical:  0 if v_a < v_b else 1 (comparing canonical representatives in [0, p))
```

cost: 1. stark constraints: ~64 (range decomposition for non-native comparison in Goldilocks). the constraint count is per-instantiation — in F₂, lt is trivial (1 constraint).

## bitwise patterns (11-14) — parameterized by W

bitwise patterns compute over W-bit words. the abstract semantics are Boolean operations on bit vectors. the word width W determines the range of valid operands and the proof cost.

valid on word type [0, W) only. bitwise on hash → ⊥_error. bitwise on field → ⊥_error (coerce to word first).

stark constraints in the canonical instantiation: ~32 each (bit decomposition required for algebraic verification in a prime field). in nox<F₂>, these same operations cost 1 constraint each (native in characteristic 2). the cost ratio is the honest algebraic distance between F_p and Z/2^W.

### pattern 11: xor

```
abstract:   xor_W(a, b) → bitwise exclusive-or over W bits
canonical:  v_a ⊕ v_b (32-bit XOR)
```

cost: 1.

### pattern 12: and

```
abstract:   and_W(a, b) → bitwise conjunction over W bits
canonical:  v_a ∧ v_b (32-bit AND)
```

cost: 1.

### pattern 13: not

```
abstract:   not_W(a) → bitwise complement over W bits
canonical:  v_a ⊕ (2^32 - 1)
```

bitwise complement. unary — single operand.

cost: 1.

### pattern 14: shl

```
abstract:   shl_W(a, n) → left shift over W bits, n must be in [0, W)
canonical:  (v_a << v_n) mod 2^32, shifts ≥ 32 produce 0
```

right shift is expressible as `shl(a, W-n)` followed by `and` with a mask.

cost: 1.

## hash pattern (15) — parameterized by H

```
reduce(s, [15 a], f) →
  let (v_a, f1) = reduce(s, a, f - cost_H)
  (H(v_a), f1)
```

computes the identity of the evaluated operand. with polynomial nouns, this is hemera(Lens.commit(input_polynomial) ‖ domain_tag) — the Lens commitment wrapped with domain separation. result type depends on H's output size.

### canonical (Hemera-2)

result is a 4-element hash (32 bytes, type tag 0x02).

the hash pattern computes: Lens.commit the input noun's polynomial (O(N) field ops), then hemera-wrap the commitment with the domain tag (1 hemera call). for small inputs this is comparable to a direct hemera permutation. for large inputs it is cheaper than recursive hemera hashing.

hash CAN be expressed as pure Layer 1 patterns (~1000 field ops for the Poseidon2 permutation with 24 rounds, x⁻¹ S-box in partial rounds). pattern 15 is simultaneously a Layer 1 pattern and the first Layer 3 jet. the jet accelerates; semantics unchanged.

cost: 200. stark constraints: ~736.

the cost is per-instantiation. a different hash function H would have different cost.

## hint pattern (16) — Layer 2

```
reduce(s, [16 constraint], f) =
  let (check, f1) = reduce(s, constraint, f - 1)
  let w = PROVER_INJECT()                         — prover supplies witness noun
  let (v, f2) = reduce(w, check, f1)              — apply check as formula to w as object
  assert v = 0                                     — constraint must produce zero (field element)
  (w, f2)
```

the single non-deterministic pattern. the prover injects a witness noun `w` from outside the VM. the constraint formula is evaluated with `s` as object to produce `check` — a formula. then `check` is applied to `w` as object via standard reduction. the result must be the field element 0 (success, same convention as branch/eq). if `reduce(w, check, f1)` produces a non-zero value, halts, or errors, the hint fails and the proof is invalid.

the verifier NEVER executes hint directly — it checks constraint satisfaction via the stark proof. the trace includes the rows for both `reduce(s, constraint, ...)` and `reduce(w, check, ...)`. the witness `w` appears in the trace as the object of the constraint-check rows.

```
PROVER_INJECT: → Noun
  source:   external to the VM, prover-only
  verifier: checks via stark (multilinear trace + sumcheck)
  cost:     hint dispatch costs 1. the two sub-reductions (constraint
            evaluation and check application) cost whatever they cost.
            witness search (PROVER_INJECT) is external — zero focus cost.
  memo:     NOT memoizable (different provers may inject different valid witnesses)
```

hint is the entire mechanism of privacy, search, and oracle access:

```
identity:         hint injects the secret behind a neuron address
                  Layer 1 checks: H(secret) = address

private transfer: hint injects record details (owner, value, nonce)
                  Layer 1 checks: conservation, ownership, nullifier freshness

AI inference:     hint injects neural network weights
                  Layer 1 checks: forward pass produces claimed output

optimization:     hint injects an optimal solution
                  Layer 1 checks: solution satisfies constraints AND is optimal
```

## cost table (canonical: nox<Goldilocks, Z/2^32, Hemera>)

```
Layer │ Pattern      │ Exec Cost      │ STARK Constraints │ Rationale
──────┼──────────────┼────────────────┼───────────────────┼─────────────────────
  1   │ 0 axis       │ 1              │ 1                 │ O(1) Lens opening (polynomial evaluation)
  1   │ 1 quote      │ 1              │ 1                 │ literal return
  1   │ 2 compose    │ 1              │ 1                 │ dispatch only
  1   │ 3 cons       │ 1              │ 1                 │ cell construction
  1   │ 4 branch     │ 1              │ 1                 │ test + select
  1   │ 5 add        │ 1              │ 1                 │ F-addition
  1   │ 6 sub        │ 1              │ 1                 │ F-subtraction
  1   │ 7 mul        │ 1              │ 1                 │ F-multiplication
  1   │ 8 inv        │ 64             │ 1                 │ F-inverse (Goldilocks)
  1   │ 9 eq         │ 1              │ 1                 │ equality comparison
  1   │ 10 lt        │ 1              │ ~64               │ range decomposition (Goldilocks)
  1   │ 11-14 bit    │ 1              │ ~32 each          │ bit decomposition (Z/2^32 in F_p)
  1   │ 15 hash      │ 200            │ ~736              │ Hemera-2 permutation
  2   │ 16 hint      │ 1              │ 1                 │ inject + dispatch
```

the exec cost is the focus deducted for THIS pattern's dispatch. sub-expression
reduce() calls deduct their own costs separately. three patterns have multi-step
overhead (axis: depth traversal steps, inv: 64 sequential multiplications,
hash: 200 Poseidon2 round rows). all other patterns cost exactly 1.

per-instantiation costs that change across algebras: inv (execution cost depends on inversion algorithm), lt (constraint count depends on field size), bitwise (constraint count depends on whether the field is binary-native), hash (cost depends on H).

## test vectors (canonical: nox<Goldilocks>)

```
add(1, 2) = 3
mul(p-1, p-1) = 1
inv(2) = 9223372034707292161
inv(0) = ⊥_error

reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 97)
  // add(1) + axis(1) + axis(1) = 3 reduce() calls, 3 focus

reduce(42, [1 7], 10) = (7, 9)
  // quote(1) = 1 reduce() call, 1 focus

reduce([1,2], [3 [[0 2] [0 3]]], 100) = (cell(1, 2), 97)
  // cons(1) + axis(1) + axis(1) = 3 reduce() calls, 3 focus

reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100)
  = (200, 95)
  // branch(1) + eq(1) + axis(1) + axis(1) + quote(1) = 5 reduce() calls, 5 focus
```

# nox virtual machine specification

version: 0.1
status: canonical

## overview

nox is a proof-native virtual machine. sixteen deterministic reduction patterns over the Goldilocks field (Layer 1), one non-deterministic witness injection pattern (Layer 2), five jets for efficient recursive stark verification (Layer 3).

every nox execution produces a trace that IS the stark witness. there is no separate arithmetization step.

## field

```
FIELD: Goldilocks
  p = 2^64 - 2^32 + 1 = 18446744069414584321
  Primitive root: 7
  2^32-th root of unity: 1753635133440165772

  Efficient reduction:
    a mod p = a_lo - a_hi × (2^32 - 1) + correction

HASH: Hemera (Poseidon2-Goldilocks)
  State: 12 field elements, Rate: 8 elements
  Rounds: 8 full + 22 partial + 8 full
  Cost: ~300 stark constraints per permutation
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
┌───────────────────────────────────────────────────────────────────────┐
│  TYPE TAG    │  REPRESENTATION     │  VALID RANGE    │  USE          │
├──────────────┼─────────────────────┼─────────────────┼───────────────┤
│  0x00: field │  Single F_p element │  [0, p)         │  Arithmetic   │
│  0x01: word  │  Single F_p element │  [0, 2^64)      │  Bitwise      │
│  0x02: hash  │  4 × F_p elements   │  256-bit digest │  Identity     │
└───────────────────────────────────────────────────────────────────────┘
```

### coercion rules

```
field → word:  always valid (Goldilocks element fits in u64)
word → field:  always valid (injection)
hash → field:  extract first element (lossy, for compatibility only)
field → hash:  forbidden (use HASH pattern)
```

### type errors

```
bitwise op on hash → ⊥_error
arithmetic on hash (except equality) → ⊥_error
```

## reduction

```
reduce : (Subject, Formula, Focus) → Result

Result = (Noun, Focus')     — success with remaining focus
       | Halt               — focus exhausted
       | ⊥_error            — type/semantic error
       | ⊥_unavailable      — referenced content not retrievable
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

## pattern reference

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
╚═══════════════════════════════════════════════════════════════════════════╝
```

## Layer 1: sixteen deterministic patterns

both prover and verifier execute these identically. confluent by Huet-Levy (1980): orthogonal rewrite system, any evaluation order yields the same result.

### structural (0-4)

```
PATTERN 0: AXIS
reduce(s, [0 a], f) = (axis(s, eval(a)), f - 1 - depth)

  axis(s, 0)   = H(s)           ; hash introspection
  axis(s, 1)   = s              ; identity
  axis(s, 2)   = head(s)        ; left child (⊥_error if atom)
  axis(s, 3)   = tail(s)        ; right child (⊥_error if atom)
  axis(s, 2n)  = axis(axis(s,n), 2)
  axis(s, 2n+1)= axis(axis(s,n), 3)


PATTERN 1: QUOTE
reduce(s, [1 c], f) = (c, f - 1)

  returns c literally, unevaluated.


PATTERN 2: COMPOSE
reduce(s, [2 [x y]], f) =
  let (rx, f1) = reduce(s, x, f - 2)
  let (ry, f2) = reduce(s, y, f1)
  reduce(rx, ry, f2)

  PARALLELISM: reduce(s,x) and reduce(s,y) are INDEPENDENT.


PATTERN 3: CONS
reduce(s, [3 [a b]], f) =
  let (ra, f1) = reduce(s, a, f - 2)
  let (rb, f2) = reduce(s, b, f1)
  (cell(ra, rb), f2)

  PARALLELISM: reduce(s,a) and reduce(s,b) are INDEPENDENT.


PATTERN 4: BRANCH (lazy)
reduce(s, [4 [test [yes no]]], f) =
  let (t, f1) = reduce(s, test, f - 2)
  if t = 0 then reduce(s, yes, f1)
           else reduce(s, no, f1)

  CRITICAL: only ONE branch evaluated. prevents infinite recursion DoS.
```

### field arithmetic (5-10)

```
PATTERN 5: ADD
reduce(s, [5 [a b]], f) →
  let (v_a, f1) = reduce(s, a, f - 1)
  let (v_b, f2) = reduce(s, b, f1)
  ((v_a + v_b) mod p, f2)


PATTERN 6: SUB
reduce(s, [6 [a b]], f) → ((v_a - v_b) mod p, f2)


PATTERN 7: MUL
reduce(s, [7 [a b]], f) → ((v_a × v_b) mod p, f2)


PATTERN 8: INV
reduce(s, [8 a], f) →
  let (v_a, f1) = reduce(s, a, f - 64)
  if v_a = 0 then ⊥_error
  (v_a^(p-2) mod p, f1)

  Execution cost reflects real work (~64 multiplications via
  square-and-multiply for Fermat's little theorem).
  stark verification cost = 1 constraint (verifier checks a × a⁻¹ = 1).


PATTERN 9: EQ
reduce(s, [9 [a b]], f) → (0 if v_a = v_b else 1, f2)


PATTERN 10: LT
reduce(s, [10 [a b]], f) → (0 if v_a < v_b else 1, f2)
```

all binary patterns: reduce(s, [op [a b]], f) evaluates both operands (parallelizable), then applies op.

### bitwise (11-14)

```
valid on word type [0, 2^64). bitwise on hash → ⊥_error.

PATTERN 11: XOR
reduce(s, [11 [a b]], f) → (v_a ⊕ v_b, f2)

PATTERN 12: AND
reduce(s, [12 [a b]], f) → (v_a ∧ v_b, f2)

PATTERN 13: NOT
reduce(s, [13 a], f) → (v_a ⊕ (2^64 - 1), f1)

PATTERN 14: SHL
reduce(s, [14 [a n]], f) → ((v_a << v_n) mod 2^64, f2)
```

### hash (15)

```
PATTERN 15: HASH
reduce(s, [15 a], f) →
  let (v_a, f1) = reduce(s, a, f - 300)
  (H(v_a), f1)

result is 4-element hash (256 bits).
hash CAN be expressed as pure Layer 1 patterns (~2800 field ops for Poseidon2).
pattern 15 is also the first Layer 3 jet. jets accelerate; semantics unchanged.
```

## Layer 2: hint (pattern 16)

```
PATTERN 16: HINT
reduce(s, [16 constraint], f) =
  let (check, f1) = reduce(s, constraint, f - 1)
  let w = PROVER_INJECT()
  assert check(w) = 0            — Layer 1 verifies the constraint
  (w, f1)

PROVER_INJECT: → Noun
  Source:   external to the VM. prover-only.
  Verifier: NEVER executes hint directly.
             checks constraint satisfaction via stark (multilinear trace + sumcheck).
  Cost:     1 + cost(constraint). witness search is external.
  Memo:     NOT memoizable (different provers, different valid witnesses).
```

## Layer 3: jets

five jets selected by analyzing the stark verifier bottleneck. every jet has an equivalent Layer 1 program producing identical output on all inputs.

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                          LAYER 3: JETS                                    ║
╠═══════════════════════════════════════════════════════════════════════════╣
║                                                                           ║
║  JET 0: HASH                                                              ║
║  hash(x) → 4 × F_p digest                                                ║
║  Pure equivalent: ~2,800 field ops (Poseidon2 permutation)                ║
║  Jet cost: 300                                                            ║
║  Accelerates: Fiat-Shamir challenges, Merkle tree construction            ║
║                                                                           ║
║  JET 1: POLY_EVAL                                                         ║
║  poly_eval(coeffs, point) → F_p                                           ║
║  Horner evaluation of degree-N polynomial at a single point               ║
║  Pure equivalent: ~2N patterns (N muls + N adds)                          ║
║  Jet cost: N                                                              ║
║  Accelerates: WHIR query verification, constraint evaluation              ║
║                                                                           ║
║  JET 2: MERKLE_VERIFY                                                     ║
║  merkle_verify(root, leaf, path, index) → {0, 1}                         ║
║  Verify authentication path of depth d                                    ║
║  Pure equivalent: d × ~310 patterns (hash + conditional per level)        ║
║  Jet cost: d × 300                                                        ║
║  Accelerates: stark proof checking (500K → 50K of verifier cost)          ║
║                                                                           ║
║  JET 3: FRI_FOLD                                                          ║
║  fri_fold(poly_layer, challenge) → poly_layer_next                        ║
║  One round of FRI folding: split by parity, combine with challenge        ║
║  Pure equivalent: ~N patterns (N/2 muls + N/2 adds + restructuring)       ║
║  Jet cost: N/2                                                            ║
║  Accelerates: WHIR verification (log(N) folding rounds)                   ║
║                                                                           ║
║  JET 4: NTT                                                               ║
║  ntt(values, direction) → transformed values                              ║
║  Number Theoretic Transform (forward or inverse) over F_p                 ║
║  Pure equivalent: ~2N·log(N) patterns (butterfly operations)              ║
║  Jet cost: N·log(N)                                                       ║
║  Accelerates: polynomial multiplication, WHIR commitment, aggregation     ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

jet semantic contract: every jet MUST have an equivalent pure Layer 1 expression producing identical output on all inputs. jets are OPTIMIZATION. semantics unchanged.

### hardware mapping

```
GFP primitive                jets it accelerates
───────────────────────────  ──────────────────────────────────────────
fma (field multiply-accumulate)  poly_eval (Horner's method = iterated FMA)
ntt (NTT butterfly)              ntt (direct correspondence)
p2r (Poseidon2 round)            hash, merkle_verify (hash-dominated)
lut (lookup table)               activation functions via Layer 1 patterns
```

## cost table

```
Layer │ Pattern      │ Exec Cost      │ stark Constraints
──────┼──────────────┼────────────────┼───────────────────
  1   │ 0 axis       │ 1 + depth      │ ~depth
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
  2   │ 16 hint      │ 1 + constraint │ constraint rows
  3   │ hash         │ 300            │ ~300
  3   │ poly_eval(N) │ N              │ ~N
  3   │ merkle_v(d)  │ d × 300        │ ~d × 300
  3   │ fri_fold(N)  │ N/2            │ ~N/2
  3   │ ntt(N)       │ N·log(N)       │ ~N·log(N)
```

### stark verifier cost with jets

```
Component               │ Layer 1 only │ With jets  │ Reduction
────────────────────────┼──────────────┼────────────┼──────────
Parse proof             │     ~1,000   │    ~1,000  │  1×
Fiat-Shamir challenges  │    ~30,000   │    ~5,000  │  6×
Merkle verification     │   ~500,000   │   ~50,000  │ 10×
Constraint evaluation   │    ~10,000   │    ~3,000  │  3×
WHIR verification       │    ~50,000   │   ~10,000  │  5×
────────────────────────┼──────────────┼────────────┼──────────
TOTAL                   │   ~600,000   │   ~70,000  │ ~8.5×
```

### cost examples

```
simple addition: 4 patterns
  [5 [[0 2] [0 3]]]
  cost: 1 (add) + 2 (axis) + overhead = ~4

Poseidon hash: 300 (jet) or ~2800 (pure Layer 1)
  [15 [0 1]]
  jet cost: 300

Merkle verification (32 levels): ~9,600 (jet) or ~10,000 (pure Layer 1)
  merkle_verify(root, leaf, path, 32)
  jet cost: 32 × 300 = 9,600

stark verifier (one recursion level): ~70,000 (with jets)
  without jets: ~600,000 Layer 1 patterns
```

## parallel reduction

```
PATTERN PARALLELISM
───────────────────

Pattern 2 (compose):  [2 [x y]]
  reduce(s,x) ∥ reduce(s,y)  — INDEPENDENT
  Then: reduce(result_x, result_y)

Pattern 3 (cons):     [3 [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: Cell(result_a, result_b)

Patterns 5-7, 9-12:   [op [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: apply op

Pattern 4 (branch):   [4 [t [c d]]]
  reduce(s,t) first
  Then: ONE of reduce(s,c) or reduce(s,d)  — NOT parallel (lazy)
```

## confluence

Layer 1 patterns form an orthogonal rewrite system (unique tags, linear left-hand sides, non-overlapping). by Huet-Levy (1980), confluent without requiring termination. parallel, lazy, and eager reduction all produce identical results.

Layer 2 (`hint`) breaks confluence intentionally — multiple valid witnesses may satisfy the same constraints. soundness is preserved: no invalid witness passes the constraint check.

Layer 3 (jets) preserves confluence — jets are observationally equivalent to their Layer 1 expansions.

## global memoization

```
GLOBAL CACHE
────────────
Key:   (H(subject), H(formula))
Value: H(result)

Properties:
- Universal: any node can contribute/consume
- Permanent: results never change (determinism)
- Verifiable: result hash checkable against proof

LAYER SCOPE:
- Layer 1: fully memoizable (deterministic)
- Layer 2: NOT memoizable (hint results are prover-specific)
- Layer 3: fully memoizable (jets are deterministic)

computations containing hint are excluded from the global cache.
pure subexpressions within a hint-containing computation remain memoizable.
```

## test vectors

```
add(1, 2) = 3
mul(p-1, p-1) = 1
inv(2) = 9223372034707292161

reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 96)
  // add(axis 2, axis 3) = add(1, 2) = 3
```

## stark integration

the execution trace (register states per step) IS the AIR witness.
- pattern tag → constraint selector
- pattern semantics → transition constraint polynomial
- 16 register columns × 2^n rows

trace encodes as one multilinear polynomial. WHIR commits it. SuperSpartan sumcheck verifies. output: stark proof (~60-157 KiB).

see reference/trace.md for trace layout and AIR constraints. see zheng for proof system details.

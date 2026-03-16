# reduction specification

version: 0.1
status: canonical

## overview

reduction is the execution model of nox. a formula is applied to an object under a focus budget, producing a result.

## reduction signature

```
reduce : (Object, Formula, Focus) → Result

  Object   : Noun    — the environment, the data, the context
  Formula  : Noun    — the code (cell of form [tag body])
  Focus    : F_p     — resource budget, decremented per pattern
                       comparison (f < cost) uses integer ordering on canonical [0, p)
                       the Halt guard prevents subtraction from ever wrapping

Result = (Noun, Focus')     — success with remaining focus
       | Halt               — focus exhausted (f < cost of next pattern)
       | ⊥_error            — type/semantic error (bitwise on hash, inv(0), axis on atom)
       | ⊥_unavailable      — referenced content not retrievable (network partition)
```

## focus metering

every pattern costs focus. the cost is deducted before the pattern executes. if remaining focus is less than the pattern's cost, reduction halts.

```
reduce(s, [5 [a b]], f) =
  if f < 1 then Halt
  let (v_a, f1) = reduce(s, a, f - 1)
  ...
```

focus is the same resource that weights cyberlinks in the cybergraph. a neuron spends focus to think (run nox programs) and to speak (create cyberlinks). the budget is unified — attention and computation are the same currency.

## evaluation order

formulas are evaluated recursively. the tag determines which pattern fires. the body structure determines the operands.

```
dispatch(s, formula, f) =
  let (tag, body) = formula        — formula must be a cell, else ⊥_error
  match tag:
    0  → axis(s, body, f)
    1  → quote(body, f)
    2  → compose(s, body, f)
    3  → cons(s, body, f)
    4  → branch(s, body, f)
    5  → add(s, body, f)
    ...
    15 → hash(s, body, f)
    16 → hint(s, body, f)
    _  → ⊥_error                   — unknown pattern tag
```

if formula is an atom (not a cell), reduction produces ⊥_error.

## confluence

Layer 1 patterns form an orthogonal rewrite system:
- each pattern has a unique tag (non-overlapping left-hand sides)
- left-hand sides are linear (no variable appears twice)
- patterns are non-overlapping (tag uniquely determines the rule)

by the Huet-Levy theorem (1980), orthogonal term rewriting systems are confluent without requiring termination.

consequence: the result depends only on what the program IS, never on how it was evaluated. parallel reduction, lazy reduction, eager reduction, any mixture — the answer is the same.

consequence: content-addressed memoization is sound. `(H(object), H(formula))` uniquely determines `H(result)`.

Layer 2 (`hint`) breaks confluence intentionally — multiple valid witnesses may satisfy the same constraints. soundness is preserved: any witness that passes the Layer 1 constraint check is valid. hint is the deliberate injection point for non-determinism.

Layer 3 (jets) preserves confluence — jets are observationally equivalent to their Layer 1 expansions. replacing a jet with its pure equivalent produces identical results.

## parallel reduction

confluence enables safe parallelism. specific patterns have independent sub-computations:

```
Pattern 2 (compose):  [2 [x y]]
  reduce(s,x) ∥ reduce(s,y)  — INDEPENDENT
  Then: reduce(result_x, result_y)

Pattern 3 (cons):     [3 [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: cell(result_a, result_b)

Patterns 5-7, 9-12:   [op [a b]]
  reduce(s,a) ∥ reduce(s,b)  — INDEPENDENT
  Then: apply op

Pattern 4 (branch):   [4 [t [c d]]]
  reduce(s,t) first — MUST evaluate test before choosing
  Then: ONE of reduce(s,c) or reduce(s,d)  — NOT parallel (lazy)
```

all binary arithmetic and bitwise patterns can evaluate both operands in parallel. branch is the only pattern that enforces sequential evaluation (test before choice).

## global memoization

```
Key:   (H(object), H(formula))
Value: H(result)
```

properties:
- universal: any node in the network can contribute and consume
- permanent: results never change (confluence guarantees determinism)
- verifiable: result hash is checkable against the stark proof

layer scope:
- Layer 1: fully memoizable (deterministic)
- Layer 2: NOT memoizable (hint results are prover-specific)
- Layer 3: fully memoizable (jets are deterministic)

computations containing hint anywhere in their reduction tree are excluded from the global cache. pure sub-expressions within a hint-containing computation remain memoizable — the exclusion applies to the hint-tainted root, not to its pure children.

## error propagation

errors propagate upward through the reduction tree. if any sub-expression produces ⊥_error or ⊥_unavailable, the parent expression produces the same error.

```
reduce(s, [5 [a b]], f) =
  let (v_a, f1) = reduce(s, a, f - 1)
  if v_a is error → return error
  let (v_b, f2) = reduce(s, b, f1)
  if v_b is error → return error
  ((v_a + v_b) mod p, f2)
```

Halt propagates identically — if a sub-expression exhausts focus, the parent halts.

## stark integration

the reduction trace (sequence of pattern applications with register states) IS the stark witness. see trace.md for the register layout and AIR constraints. see jets.md for optimized verification.

# cost model redesign

status: implemented

## problem

the current cost table and test vectors are inconsistent. working backwards
from the four test vectors, no single rule produces all four results:

```
TV1: reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 96)   → cost 4
TV2: reduce(42, [1 7], 10)                  = (7, 9)    → cost 1
TV3: reduce([1,2], [3 [[0 2] [0 3]]], 100)  = (cell, 96) → cost 4
TV4: reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100)
                                             = (200, 95)  → cost 5
```

patterns dispatched per TV:
- TV1: add, axis, axis         → 3 dispatches, cost 4 (off by +1)
- TV2: quote                   → 1 dispatch,  cost 1 (matches)
- TV3: cons, axis, axis        → 3 dispatches, cost 4 (off by +1)
- TV4: branch, eq, axis, axis, quote → 5 dispatches, cost 5 (matches)

TV2 and TV4 follow "1 per dispatch." TV1 and TV3 have 1 extra.

## analysis

the extra cost in TV1/TV3 can be explained by add/cons requiring a
separate result row in the trace (dispatch row + result row = 2 rows
for the pattern itself). but if ALL compound patterns cost 2, then TV4
gives: branch(2) + eq(2) + axis(1) + axis(1) + quote(1) = 7. wrong.

the only cost assignment that fits all four test vectors:

| pattern | cost | TV verification |
|---------|------|-----------------|
| add     |  2   | TV1: 2+1+1 = 4 ✓ |
| cons    |  2   | TV3: 2+1+1 = 4 ✓ |
| branch  |  1   | TV4: 1+1+1+1+1 = 5 ✓ |
| eq      |  1   | TV4: see above ✓ |
| axis    |  1   | all TVs ✓ |
| quote   |  1   | TV2: 1 = 1 ✓ |

## proposed model

**cost = 1 per reduce() call (dispatch cost)**

this is the base rule. every time reduce(s, formula, f) is entered,
1 focus is deducted for dispatch.

**multi-step patterns add overhead on top of the dispatch cost:**

| pattern | dispatch | overhead | total | rationale |
|---------|----------|----------|-------|-----------|
| axis    | 1        | depth-1  | depth | tree traversal: 1 step per level |
| quote   | 1        | 0        | 1     | literal return |
| compose | 1        | 0        | 1     | delegates to 3 sub-reduce calls |
| cons    | 1        | 0        | 1     | cell construction |
| branch  | 1        | 0        | 1     | test + mux |
| add     | 1        | 0        | 1     | field addition |
| sub     | 1        | 0        | 1     | field subtraction |
| mul     | 1        | 0        | 1     | field multiplication |
| inv     | 1        | 63       | 64    | square-and-multiply chain |
| eq      | 1        | 0        | 1     | equality comparison |
| lt      | 1        | 0        | 1     | less-than comparison |
| 11-14   | 1        | 0        | 1     | bitwise operation |
| hash    | 1        | 299      | 300   | Poseidon2 permutation |
| hint    | 1        | 0        | 1     | inject (sub-reductions separate) |

total focus = sum of pattern costs across all reduce() calls.

## test vector reconciliation

with this model:

```
TV1: add(1) + axis(1) + axis(1) = 3 → (3, 97)       CHANGED from 96
TV2: quote(1) = 1               → (7, 9)             unchanged
TV3: cons(1) + axis(1) + axis(1) = 3 → (cell, 97)    CHANGED from 96
TV4: branch(1) + eq(1) + axis(1) + axis(1) + quote(1) = 5 → (200, 95)  unchanged
```

TV1 and TV3 change. TV2 and TV4 stay.

## alternative: keep TV1/TV3, change TV4

to keep TV1 (cost=4) and TV3 (cost=4), compound patterns must cost 2.
then TV4 should be: branch(2) + eq(1) + axis(1) + axis(1) + quote(1) = 6 → 94.

this means TV4 changes from 95 to 94.

principle: patterns with sub-expressions that they evaluate cost 2
(dispatch + result). patterns without (quote, axis) cost 1.

| pattern | cost |
|---------|------|
| axis    | depth |
| quote   | 1 |
| compose | 2 (evaluates x and y) |
| cons    | 2 (evaluates a and b) |
| branch  | 2 (evaluates test, then chosen branch) |
| add-mul | 2 (evaluates a and b) |
| inv     | 64 |
| eq/lt   | 1 ← but eq evaluates two sub-expressions too? |
| 11-14   | 1 ← but these evaluate two sub-expressions too? |
| not     | 1 |
| hash    | 300 |
| hint    | 1 |

problem: eq/lt/xor/and/shl also evaluate sub-expressions but would
need to cost 1, not 2, for this to work. the only pattern that
evaluates sub-expressions AND costs 1 is eq. this breaks the
principle. so TV4 (95) is probably wrong → should be 94.

but then eq would also cost 2, giving TV4 = 2+2+1+1+1 = 7 → 93.
even worse. to get 94: branch(2) + eq(1) + axis(1) + axis(1) + quote(1) = 6.
so eq must cost 1 while branch costs 2. the distinction: branch has
a result row (selecting the branch output), eq does not (comparison
is pure constraint, no separate result). but then add should also be
pure constraint (r7 = r5 + r6) — why does add cost 2?

this alternative has more unexplained distinctions than the proposed model.

## recommendation

go with the proposed model: **1 per reduce() call, multi-step
overhead only for axis/inv/hash.**

- maximally simple: one rule covers everything
- multi-step overhead has a clear physical reason (sequential computation
  that cannot be expressed in 1 constraint row)
- test vectors change: TV1 96→97, TV3 96→97
- cost table simplifies: only 3 special cases (axis=depth, inv=64, hash=300)

## trace implications

the 1-row-per-reduce model means compound patterns don't need separate
dispatch and result rows. the trace layout changes:

current: compound patterns have dispatch row + result row (2 rows).
transition constraints connect them locally.

proposed: compound patterns have 1 row. operand values flow from
sub-expression result rows through CCS wiring constraints. SuperSpartan
handles cross-row constraints natively (trace.md:83 already states this).

trace.md AIR constraints change from transition form:
  r7_{t+1} = r5_t + r6_t
to in-row form:
  r7_t = r5_t + r6_t
with wiring constraints connecting r5_t and r6_t to sub-expression results.

multi-row patterns section updates:
- compose: 2 → 1 row (was: dispatch x, dispatch y)
- cons: 2 → 1 row (was: dispatch a, dispatch b + result)
- inv: 64 rows (unchanged)
- hash: 300 rows (unchanged)

## files to change

1. patterns.md — cost table: compose 2→1, cons 2→1, branch 2→1
2. patterns.md — test vectors: TV1 96→97, TV3 96→97
3. patterns.md — individual pattern descriptions: update cost values
4. reduction.md — focus accounting: rewrite with clean 1-per-reduce rule
5. reduction.md — focus metering example: update
6. reduction.md — remove destructuring cost discussion
7. trace.md — AIR constraints: transition → in-row for simple patterns
8. trace.md — multi-row patterns: update compose/cons descriptions
9. jets.md — cost examples: update simple addition cost 4→3

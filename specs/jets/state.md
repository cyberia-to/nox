---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: state jets, state operations, state transition jets
---
# state jets — polynomial state transitions

jets that optimize proving state transitions on BBG polynomial state. committed in genesis BBG state.

state operations are compositions of the 16 nox patterns applied to the polynomial evaluation table. state jets recognize these compositions by formula hash and replace them with optimized CCS encodings.

## five primitive operations

the BBG polynomial evaluation table is a noun. state operations are standard nox patterns operating on this noun:

| operation | nox patterns | what it does | constraints |
|-----------|-------------|--------------|-------------|
| READ(dimension, key) → value | Lens.open (polynomial evaluation) | O(1) evaluation at (dimension, key) | 1 |
| WRITE(dimension, key, value) | axis (0) + cons (3) | navigate to position, build updated tree | 1 |
| ASSERT_EQ(a, b) | eq (9) | check two field values are equal | 1 |
| ADD(a, b) → c | add (5) | field addition on state values | 1 |
| MUL(a, b) → c | mul (7) | field multiplication on state values | 1 |

18 patterns are the axiom. state operations are derived. no new instructions.

### irreducibility

| removed | what breaks |
|---------|-------------|
| READ | can't access polynomial state |
| WRITE | can't modify polynomial state |
| ASSERT_EQ | can't verify any constraint |
| ADD | can't compute sums (ring incomplete) |
| MUL | can't compute products (ring incomplete) |

## six genesis state jets

### cyberlink circuit (exact match)

H(formula) → hand-optimized CCS encoding for the cyberlink transaction.

| jet | decomposition | jet constraints | without jet |
|-----|--------------|-----------------|-------------|
| CYBERLINK | full cyberlink validation circuit | ~3,200 | ~25,000 |

### five templates (pattern match)

formula structure matches a template → parameterized CCS encoding. fires automatically — no protocol upgrade needed.

| jet | decomposition | jet constraints | without jet |
|-----|--------------|-----------------|-------------|
| TRANSFER(source, target, amount) | 2 READ + RANGE + 2 ADD + 2 WRITE + ASSERT_EQ | 3 | ~8,000 |
| INSERT(table, key, value) | READ(=0) + schema_check + WRITE | 5 | ~6,000 |
| UPDATE(table, key, old, new) | READ + ASSERT_EQ + WRITE | 5 | ~4,000 |
| AGGREGATE(table, key, delta) | READ + ADD + WRITE | 2 | ~4,000 |
| CONSERVE(inputs, outputs) | ADD chain + ASSERT_EQ | n | ~4,000 |

### fallback

unrecognized formulas: full nox execution trace → SuperSpartan proof (~8 constraints per trace row).

## proof-carrying interaction

without state jet: each nox reduce() step folds into accumulator (~30 field ops × 500 steps = 15,000 field ops).
with state jet: pattern recognized → 1 fold of the 3-5 constraint instance = ~30 field ops total. 500× speedup.

## hardware mapping

state jets map to fma (field multiply-accumulate) — the dominant GFP primitive for polynomial evaluation and field arithmetic.

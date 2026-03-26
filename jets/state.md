---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: state jets, CCS jets, state transition jets
---
# state jets — polynomial state transitions

jets that optimize PROVING state transitions. verifier jets reduce trace length (fewer execution steps). state jets reduce CONSTRAINT COUNT (fewer CCS constraints per transition). both use the same formula-hash recognition mechanism.

a state transition is a nox program that reads polynomial state, validates changes, and writes updated values. the 5 primitive state operations (see [[state-operations]]) are nox patterns: READ/WRITE (polynomial evaluation), ASSERT_EQ (pattern 9), ADD (pattern 5), MUL (pattern 7).

with polynomial nouns, READ is O(1) polynomial evaluation via Lens opening — every table lookup is O(1) regardless of table size.

## recognition hierarchy

**level 1 (exact formula match):** H(formula) → hand-optimized CCS encoding.

the genesis table circuits (cyberlink: ~3,200 constraints) are level 1 state jets.

**level 2 (pattern match):** nox formula matches a template → parameterized CCS encoding.

| template | decomposition | jet constraints | without jet |
|---|---|---|---|
| TRANSFER(source, target, amount) | 2 READ + RANGE + 2 ADD + 2 WRITE + ASSERT_EQ | 3 | ~8,000 |
| INSERT(table, key, value) | READ(=0) + schema_check + WRITE | 5 | ~6,000 |
| UPDATE(table, key, old, new) | READ + ASSERT_EQ + WRITE | 5 | ~4,000 |
| AGGREGATE(table, key, delta) | READ + ADD + WRITE | 2 | ~4,000 |
| CONSERVE(inputs, outputs) | ADDs + ASSERT_EQ | n | ~4,000 |

level 2 jets fire AUTOMATICALLY when formula structure matches a template. no protocol upgrade needed to recognize patterns in new user-defined tables.

**level 3 (type-based):** schema-aware generic encoding → constraints proportional to schema size.

**fallback:** full nox execution trace → SuperSpartan proof (~8 constraints per trace row).

## interaction with proof-carrying

without state jet: each nox reduce() step folds into accumulator (~30 field ops × 500 steps = 15,000 field ops).
with state jet: pattern recognized → 1 fold of the 3-5 constraint instance = ~30 field ops total. **500× speedup.**

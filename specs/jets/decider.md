---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: decider jet, 89-constraint verifier, all-history verification
---
# decider jet — 89 constraints

verifies ALL accumulated chain history in one step. the HyperNova decider checks the universal accumulator (~200 bytes) that folds every block from genesis.

generic cost: ~8K constraints. with three stacked optimizations: 89 constraints — less than one hemera permutation (736).

## specification

```
name:          decider
recognition:   level 1 CCS jet (exact formula match)
input:         HyperNova accumulator (~200 bytes) + Lens commitments
output:        accept / reject
constraints:   89 (optimistic) or 825 (conservative)
```

## three optimizations

| # | optimization | what it does | before → after |
|---|---|---|---|
| 1 | CCS jet encoding | direct CCS matrix instead of nox trace | ~8K → ~2,070 (4×) |
| 2 | batched spot-checks | 640 Brakedown checks → 1 batched sumcheck | ~1,280 → ~35 (36×) |
| 3 | algebraic Fiat-Shamir | derive challenges from Lens commitments, zero hemera | ~736 → 0 |

## constraint breakdown

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints
hemera:                   0 constraints
────────────────         ──────────────
total:                   89 constraints
```

## conservative tier

algebraic Fiat-Shamir (optimization 3) reuses hemera hash already inside Brakedown commitments for challenge derivation. if independent randomness is required, one hemera call remains:

```
conservative: 89 + 736 = 825 constraints
```

825 is still 10× better than generic 8K. the system ships with 825 and transitions to 89 when algebraic FS soundness is formally verified.

## what 89 constraints means

```
verify ALL chain history:       ~89 constraints ≈ ~100 nanoseconds
89 constraints < 1 hemera call:  736 constraints
verifying all history < hashing 56 bytes

light client join:
  download checkpoint: 240 bytes
  verify: 89 constraints ≈ 100 ns
  trust: mathematical certainty of all history from genesis
```

## open questions

1. algebraic Fiat-Shamir soundness — formal proof that challenge derivation from Lens commitments satisfies FS security model
2. batched sumcheck interaction — can it share challenges with the main SuperSpartan sumcheck?
3. decider jet equivalence — one-time formal verification that 89-constraint CCS = full decider

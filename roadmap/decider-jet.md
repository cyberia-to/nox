---
tags: cyber, cip
crystal-type: process
crystal-domain: cyber
status: draft
date: 2026-03-25
---
# decider jet — 89-constraint verification of all history

## the problem

the [[HyperNova]] decider is the ONE verification that runs after all folding. it checks the accumulated instance by running [[SuperSpartan]] + [[sumcheck]] + [[Brakedown]] once. current cost: ~8K constraints. this is the floor — every chain verification, every light client join, every epoch finalization pays it.

## the solution: three stacked optimizations

### optimization 1: decider CCS jet (level 1)

the decider is a FIXED computation — always the same operations in the same order. express it as a direct CCS encoding (level 1 state jet) instead of a nox execution trace:

```
component               generic nox trace    CCS jet encoding
────────────            ─────────────────    ────────────────
sumcheck replay         ~2K constraints      20 ASSERT_EQ
CCS evaluation          ~1K constraints      34 MUL + ASSERT_EQ
Brakedown spot-checks   ~3K constraints      1,280 MUL + ASSERT_EQ
Fiat-Shamir hemera      ~2K constraints      736 (1 hemera call)
────────────            ─────────────────    ────────────────
total                   ~8K                  ~2,070
```

the jet recognises the decider formula by hash and emits the direct CCS encoding. 4× reduction.

### optimization 2: batched spot-checks

the 1,280 Brakedown spot-check constraints are 640 independent multiplications (MUL + ASSERT_EQ each). same operation, different inputs. batch them via ONE sumcheck:

```
640 independent checks → 1 batched sumcheck
  rounds: log₂(640) ≈ 10
  per round: 3 field elements
  final assertion: 5 constraints
  total: ~35 constraints (was 1,280)
```

the sumcheck PROVES all 640 multiplications are correct in 35 constraints. soundness: each round catches cheating with probability ≥ 1 - d/|F_p| ≈ 1 - 2⁻⁶³. over 10 rounds: negligible error.

### optimization 3: algebraic Fiat-Shamir (zero hemera)

the [[Brakedown]] commitment contains a [[hemera]] binding hash (internal to PCS.commit). derive Fiat-Shamir challenges from this existing hash:

```
challenge_i = algebraic_derivation(PCS.commit, i)
  no additional hemera call
  challenges are deterministic from the commitment
  binding: the commitment is already hemera-bound
```

the decider needs ZERO hemera calls. all challenges derive from the existing PCS commitment. the hemera that's already inside Brakedown is the trust anchor.

## the result

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints
hemera:                   0 constraints
────────────────         ──────────────
total:                   89 constraints
```

~89 constraints. cheaper than ONE hemera permutation (736). 90× reduction from the generic ~8K.

## what 89 constraints means

```
verify ALL chain history:      ~89 constraints ≈ ~100 nanoseconds
verify one epoch (1000 blocks): 1000 × 30 + 89 = ~30,089 field ops
verify one block (1000 tx):     1000 × 30 + 89 = ~30,089 field ops

light client join:
  download checkpoint: 240 bytes
  verify: 89 constraints ≈ 100 ns
  total: 240 bytes + 100 ns

cost comparison:
  89 constraints < 1 hemera permutation (736)
  verifying ALL history < hashing 56 bytes
```

the universal accumulator (~200 bytes) proves everything from genesis. the cost of checking it is less than the cost of one hash call. verification becomes effectively FREE.

## the jet specification

```
jet name:         decider
formula hash:     H(decider_formula)
recognition:      level 1 CCS jet (exact formula match)

input:
  accumulator:    HyperNova accumulated CCS instance (~200 bytes)
  PCS commitments: Brakedown commitments from the fold chain

output:
  accept / reject

CCS encoding:
  20 equality constraints (sumcheck rounds)
  34 multiplication + equality constraints (CCS evaluation)
  35 constraints (batched Brakedown spot-checks via sumcheck)
  = 89 total

Fiat-Shamir:
  challenges derived algebraically from PCS commitments
  zero hemera calls

semantic contract:
  the jet produces the same accept/reject as running the full
  SuperSpartan + sumcheck + Brakedown verification as a nox program.
  if the jet is removed, the system falls back to generic ~8K constraints.
```

## open questions

1. **algebraic Fiat-Shamir soundness.** deriving challenges from PCS commitments (without additional hashing) must satisfy the Fiat-Shamir security model. the commitment IS a hemera hash (Brakedown binding). is reusing this hash for challenge derivation sound? or does each challenge need independent randomness? if independent randomness is required, one hemera call remains (~736 + 89 = ~825 constraints).

2. **batched sumcheck interaction.** the batched spot-check sumcheck runs INSIDE the decider CCS. can it share challenges with the main sumcheck (from SuperSpartan)? if yes: even fewer constraints. if no: the 35-constraint estimate holds.

3. **decider jet verification.** the jet itself must be VERIFIED correct — the 89-constraint CCS must be equivalent to the full decider. this is a one-time formal verification task, not a per-execution cost. publish the equivalence proof with the jet.

see [[jets]] for the jet mechanism, [[state-operations]] for CCS jets, [[polynomial proof system]] for the complete architecture, [[HyperNova]] for folding, [[Brakedown]] for the PCS

# the decider

verifying all history from genesis in 89 constraints.

## the cost of trust

every blockchain asks: "how do I trust this state?" the answers differ by orders of magnitude:

```
Bitcoin:     re-execute all transactions from genesis    O(chain_length)
Ethereum:    re-execute from last checkpoint             O(epoch_length)
IBC/Cosmos:  verify validator signatures per block       O(blocks × validators)
nox:         verify one proof                            89 constraints
```

the HyperNova accumulator folds every block into a running aggregate. at any point, one decider verification proves that ALL prior history was valid. the accumulator is ~200 bytes regardless of how long the chain has been running.

## why 89

the generic decider runs SuperSpartan + sumcheck + Brakedown as a nox program: ~8,000 constraints. three optimizations stack multiplicatively:

**optimization 1: CCS jet encoding.** the decider is always the same computation. express it as a direct CCS matrix (level 1 state jet) instead of a nox trace. ~8K → ~2,070. 4× reduction.

**optimization 2: batched spot-checks.** Brakedown verification requires 640 independent multiplication checks. batch them into one sumcheck: ~1,280 → ~35. 36× reduction.

**optimization 3: algebraic Fiat-Shamir.** the Brakedown commitment already contains a hemera binding hash. derive verification challenges from this existing hash instead of calling hemera again. ~736 → 0. the hemera that's already inside Brakedown is the trust anchor.

```
sumcheck replay:         20 constraints
CCS evaluation:          34 constraints
Brakedown (batched):     35 constraints
hemera:                   0 constraints
────────────────         ──────────────
total:                   89 constraints
```

## what it means

89 constraints < 1 hemera permutation (736 constraints).

verifying all chain history — every transaction, every proof, every state transition from genesis — costs less computation than hashing 56 bytes.

```
light client join:
  download: 240 bytes (BBG_root + accumulator + height)
  verify:   89 constraints ≈ 100 nanoseconds
  done.     full trust in all history.
```

a phone joins the network. downloads 240 bytes. verifies in 100 nanoseconds. has mathematical certainty that the entire history from genesis is valid. no syncing blocks. no downloading headers. no trusting validators. one proof, one check.

## the conservative tier

the algebraic Fiat-Shamir optimization (zero hemera) requires that deriving challenges from PCS commitments satisfies the Fiat-Shamir security model. this is believed sound but not yet formally proven for this specific construction.

if independent randomness is required, one hemera call remains: 89 + 736 = 825 constraints. still 10× better than the generic 8K. still cheaper than hashing a kilobyte.

both tiers are specified. the system ships with 825 (conservative). transitions to 89 when algebraic Fiat-Shamir soundness is formally verified.

## how folding works

every block folds into the accumulator in ~30 field operations + 1 hemera hash. this happens during execution — not as a separate proving phase. by the time computation finishes, the proof is done.

```
block 1 → fold → accumulator₁     (~30 ops)
block 2 → fold → accumulator₂     (~30 ops)
...
block N → fold → accumulatorₙ     (~30 ops)
                      ↓
              decider(accumulatorₙ)  (89 constraints)
                      ↓
                 accept / reject
```

N blocks cost N × 30 + 89 field operations total. the 89 is constant — it does not grow with chain length. a million blocks, a billion blocks, a trillion blocks: still 89 constraints for the final verification.

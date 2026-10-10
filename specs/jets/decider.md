---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: decider jet, all-history verification
---
> superseded: the legacy decider jet of the retired zheng design verifies nothing and is retired in phase 3 of [[soft3/proposals/proof-system-repair|the proof-system repair]].

# decider jet — legacy, retired in phase 3

## what the jet is today

`rs/jets/decider.rs` recognises the decider formula by its hash and charges a fixed cost. it verifies nothing: it never checks a sumcheck, a CCS evaluation or a commitment opening (launch #40, audited in nox PR #20). no caller may treat its accept as evidence of anything.

the figures this page used to carry — a constraint count for the decider, an accumulator and checkpoint size in bytes, a nanosecond verification time, challenges derived without hashing, an opening said to need no Merkle authentication — belonged to the retired design. none of them hold and none are to be quoted.

## what replaces it

in the repaired stack every proof is a zheng proof over Goldilocks with Fp3 challenges, hemera Merkle trees and a hemera Fiat–Shamir transcript.

- phase 3 (accumulation, landing in this release): the nox machine becomes one uniform step relation (zheng `specs/machine.md`). each step's Reed–Solomon evaluation claims are accumulated with a sumcheck batch, one combined committed word, out-of-domain samples and a few query openings. the accumulator is one root plus a fixed number of claims, independent of the number of steps.
- the decider is one WHIR opening (lens `specs/whir.md`) of the final accumulator, run by zheng. it is not a nox jet. nox 0.4 carries the uniform step relation; this jet and its formula anchor are retired.
- light-client checkpoint: state root + the decider proof of the accumulator. size: ≤ 64 KB goal, measured size TODO(F-numbers). verification: ≤ 1 ms goal, measured time TODO(F-numbers).

## open questions

1. genesis: whether any decider formula hash is anchored at genesis at all once the decider lives in zheng, or only the step relation's digest.
2. composition: a verifier written as a nox program (the Trident verifier) is for composing proofs, never for making them smaller.

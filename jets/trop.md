---
tags: nox, jets
crystal-type: entity
crystal-domain: comp
alias: tropical jets, trop jets, optimization jets
---
# trop jets — tropical (min,+)

five jets for (min,+) semiring operations. optimization, assignment, decoding. no separate nox instantiation — tropical operations decompose to existing patterns (branch + lt). jets accelerate common compositions.

proved via Lens₅ (tropical witness-verify) in zheng.

## jets

| # | name | input → output | pure cost | jet cost | primary workload |
|---|------|----------------|-----------|----------|------------------|
| 0 | trop_matmul | (min,+) matrices A, B → C = A⊗B | O(n³) branch+lt | O(n³) tmin | tropical matrix power |
| 1 | trop_shortest | graph, source → distances | O(E×log(V)) branch+lt | O(E×log(V)) tmin | single-source shortest path |
| 2 | trop_hungarian | cost matrix → assignment + cost | O(n³) branch+lt | O(n³) tmin | optimal assignment |
| 3 | trop_viterbi | HMM, observations → state sequence | O(S²×T) branch+lt | O(S²×T) tmin | optimal sequence decoding |
| 4 | trop_transport | μ, ν, cost matrix → transport plan | O(n³×log(n)) | O(n³×log(n)) tmin | optimal transport |
| 5 | witness_commit | tropical witness → F_p Lens commitment | O(|witness|) | O(|witness|) | trop → nebu boundary |

## execution vs proof cost

tropical jets do NOT reduce execution cost (same asymptotic). they reduce PROOF cost: the jet produces a structured witness (assignment + cost + dual certificate) that zheng verifies in O(|problem|) F_p constraints instead of proving every comparison step.

```
without jet: every min(a,b) = branch(lt(a,b), a, b) = ~10 F_p constraints
             1000-step shortest path over 100 nodes = ~10⁹ F_p constraints

with jet:    tropical computation produces witness
             zheng verifies: validity + cost + dual certificate
             total: O(|V| + |E|) F_p constraints
```

## cross-algebra

tropical witness commits via Lens₁ (Brakedown) at the boundary:

```
trop computation → witness (assignment, cost, dual certificate)
  ↓
witness_commit boundary jet: Lens₁.commit(witness) → F_p commitment
  ↓
zheng verification in F_p (structural + cost + optimality checks)
```

## hardware mapping

- all comparisons → lut (lookup table for branch+lt composition)

## lens

Lens₅: Tropical (witness-verify protocol, delegates commitment to Lens₁)

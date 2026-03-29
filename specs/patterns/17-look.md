---
tags: nox, patterns
crystal-type: entity
crystal-domain: comp
alias: look pattern, BBG read, deterministic state read
---
# look pattern (17) — deterministic BBG read

```
reduce(o, [17 [ns_f key_f]], f) =
  1. ns  = reduce(o, ns_f, f - 1)             // evaluate namespace expression
  2. key = reduce(o, key_f, f - 1)             // evaluate key expression
  3. C_t = axis(o, BBG_ROOT_AXIS)              // extract BBG commitment root from object
  4. (coeffs, proof) = evaluator.read(C_t, ns, key)
     if coeffs == ⊥_namespace → return ⊥_error
     if coeffs == ⊥_range     → return ⊥_error
  5. value = poly_eval(coeffs, key)            // evaluate polynomial at key point
  6. trace.append(proof)                       // Brakedown opening proof enters STARK trace
  7. return (value, f')
```

deterministic read from the authenticated state layer (BBG). evaluates the polynomial BBG_poly(namespace, key, t) under commitment root C_t and returns the result. C_t is part of the object noun — the BBG roots travel with the computation. the evaluator reads polynomial coefficients from the committed state, evaluates at the key point, and generates a Brakedown opening proof. the opening proof enters the STARK trace for verification.

pure function: same namespace, same key, same C_t produces the same value on any machine at any time. the polynomial uniquely determines the output. there is no prover choice, no witness injection, no non-determinism.

the verifier checks the Brakedown Lens opening proof via the STARK proof. it never reads BBG directly.

## reduction rule

```
reduce(object, [17 [namespace key]], budget) → value

where:
  namespace  ∈ {0..9}          evaluation dimension of BBG_poly
  key        ∈ F_p             evaluation point within that dimension
  C_t        from object       BBG commitment root at time t
  value      ∈ F_p             BBG_poly(namespace, key, t) evaluated under C_t
```

the formula `[17 [ns_f key_f]]` contains two sub-formulas. both are reduced against the object to produce field elements. the namespace selects which evaluation dimension of BBG_poly to read. the key selects the evaluation point within that dimension.

## evaluator interface

```
trait LookEvaluator {
    fn read(&self, commitment: Root, namespace: F, key: F) -> LookResult;
}

enum LookResult {
    Value { coefficients: Vec<F>, proof: BrakedownOpening },
    NamespaceNotFound,
    KeyOutOfRange,
}
```

the evaluator is not a prover — it performs a deterministic computation. given the commitment root, namespace, and key, there is exactly one correct result. the evaluator retrieves polynomial coefficients, evaluates at the key point, and produces the Brakedown opening proof that the evaluation is correct against C_t.

## namespace table

```
namespace    dimension       contents
─────────    ─────────       ────────
0            particles       content + axon weights
1            axons_out       directional index by source
2            axons_in        directional index by target
3            neurons         focus, karma, stake per neuron
4            locations       proof of location
5            coins           fungible token denominations
6            cards           names and knowledge assets
7            files           content availability (DAS)
8            time            temporal snapshots
9            signals         finalized signal batches
```

namespaces 0-9 are the 10 public evaluation dimensions of BBG_poly. private state (commitment polynomial A(x), nullifier polynomial N(x)) is not accessible via look — private records require call (pattern 16) with ZK witness injection.

## properties

- **deterministic**: same namespace, same key, same C_t always returns the same value. the polynomial is committed — its coefficients are fixed by the commitment root. no prover freedom, no witness choice, no oracle
- **pure**: no side effects. look reads state but never modifies it. write is implicit via the cyberlink transaction result, not via the look pattern
- **provable**: the Brakedown Lens opening proof enters the STARK trace. the verifier checks the opening against C_t without reading BBG. soundness follows from the binding property of the polynomial commitment
- **memoizable**: look results are fully cacheable. given the same C_t, namespace, and key, the result is identical. the evaluator may cache polynomial evaluations and reuse opening proofs across multiple look calls within the same block
- **read-only**: look never modifies BBG state, the object noun, or the commitment root. pure observation

## comparison with call (16)

| property | call (16) | look (17) |
|----------|-----------|-----------|
| determinism | non-deterministic — prover chooses witness | deterministic — polynomial determines value |
| output | multiple valid witnesses may satisfy the check | exactly one correct value for given inputs |
| memoizable | no — different provers produce different witnesses | yes — same C_t + namespace + key = same result |
| verification | check formula validates witness (Layer 1 reduction) | Brakedown opening proof (polynomial commitment) |
| state access | external witness injection | committed polynomial evaluation |
| confluence | intentionally broken | preserved |

call injects information from outside the computation (secrets, optimization solutions, oracle responses). look reads information already committed inside the computation (BBG state under C_t). call is existential — "there exists a witness satisfying this check." look is functional — "the value at this point is uniquely determined."

## cost model

```
component            cost
─────────            ────
dispatch             1
namespace evaluation cost(ns_f)
key evaluation       cost(key_f)
polynomial eval      depends on polynomial degree (dimension-specific)
Brakedown opening    1 (O(√N) field operations, amortized to 1 STARK constraint)
─────────            ────
total                1 + cost(ns_f) + cost(key_f) + 1
```

the polynomial evaluation cost depends on the degree of BBG_poly in the queried dimension. for single-point lookups, the Brakedown opening proof is O(√N) field operations but enters the STARK trace as a single constraint (the verifier checks the opening via the proof, not by re-evaluating).

STARK constraints: 1 (the opening proof is verified within the proof system, not expanded into constraints per coefficient).

## error cases

```
⊥_error (namespace not found):
  namespace ∉ {0..9}
  the formula evaluated ns_f to a value outside the valid dimension range.
  this is a static error — the formula is malformed.

⊥_error (key out of range):
  key exceeds the domain of the polynomial in the given dimension.
  the polynomial is defined over a specific evaluation domain;
  points outside that domain have no committed value.

⊥_error (commitment root missing):
  the object noun does not contain a BBG commitment root at the expected axis.
  look requires C_t to be present in the object structure.
```

all error cases produce ⊥_error (not ⊥_halt). look failures are deterministic — the same malformed input always produces the same error. there is no "unavailable" case analogous to call's Halt; if the commitment root exists and the namespace and key are valid, the value exists.

## memoization

look results are safe to memoize because the computation is a pure function of three inputs: C_t, namespace, key. within a single block (same C_t), repeated lookups to the same (namespace, key) return identical values. the evaluator should maintain a cache keyed by (C_t, namespace, key) to avoid redundant polynomial evaluations and proof generation.

across blocks, C_t changes and the cache must be invalidated. partial invalidation is possible: only dimensions modified by the block's transactions need cache eviction.

## what look enables

```
state queries:      look reads neuron balance, particle energy, token supply
                    Layer 1 computes over the result deterministically

conditional logic:  formulas branch on current BBG state
                    e.g., check balance before constructing a transfer

cross-dimension:    look from axons_out + look from axons_in
                    consistency is structural — same polynomial, different dimensions

temporal queries:   look at different time dimensions yields historical state
                    diff between two temporal lookups reveals changes

provable reads:     every look result carries a Brakedown opening proof
                    the verifier knows the value is correct without trusting the evaluator
```

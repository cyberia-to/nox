---
tags: nox, patterns
crystal-type: entity
crystal-domain: comp
alias: look pattern, BBG read, deterministic state read
---
# look pattern (17) — deterministic BBG read

note: current implementation delegates to the LookProvider trait. BBG commitment root extraction (C_t) and Brakedown opening proof assembly are deferred to the zheng witness generator. the interpreter reads C_t from fixed object axes and returns the provider value; proof accumulation runs after reduction via collect_look_openings().

```
reduce(o, [17 [ns_f key_f]], f) =
  1. ns  = reduce(o, ns_f,  f - 1)        // evaluate namespace expression
  2. key = reduce(o, key_f, f - 1)        // evaluate key expression
  3. C_t = extract 4 limbs from object at axes [4, 10, 22, 23]
  4. value = provider.look(C_t[0], ns, key)
     if value == None → return ⊥_unavailable
  5. return (value, f')
```

deterministic read from the authenticated state layer (BBG). extracts the BBG
commitment root from the object noun (four 64-bit Goldilocks limbs at fixed axes),
then asks a `LookProvider` for the polynomial value at the given namespace and key.
the provider returns `Option<Goldilocks>` — `Some(v)` for a successful read,
`None` when the value is not available.

pure function: same namespace, same key, same C_t produces the same value on any
machine at any time. the polynomial uniquely determines the output. no prover
choice, no witness injection, no non-determinism.

## reduction rule

```
reduce(object, [17 [ns_f key_f]], budget) → value

where:
  namespace  ∈ {0..9}          evaluation dimension of BBG_poly
  key        ∈ F_p             evaluation point within that dimension
  C_t        from object       4-limb BBG commitment root (axes 4, 10, 22, 23)
  value      ∈ F_p             BBG_poly(namespace, key, t) under C_t
```

the formula `[17 [ns_f key_f]]` contains two sub-formulas. both are reduced against
the object to produce field elements. the namespace selects which evaluation
dimension of BBG_poly to read. the key selects the evaluation point within that
dimension.

## provider interface

```rust
trait LookProvider {
    fn look(
        &self,
        commitment: Goldilocks,   // C_t[0] — first limb of the 4-limb BBG root
        namespace:  Goldilocks,   // 0..9 (validated before call)
        key:        Goldilocks,
    ) -> Option<Goldilocks>;
}
```

`None` means the value is not available (provider has no data for this C_t).
the pattern returns `⊥_unavailable` in that case.

two concrete providers exist in bbg/:

```
BbgLookProvider   fast path — reads local BBG state, no proof
ProofLookProvider accumulates LookOpening entries; collect_look_openings()
                  retrieves them after execution for zheng::commit()
```

proof accumulation is separate from execution. the trace records the inputs and
output; the proof is assembled after reduction completes, not during.

## C_t extraction

the BBG commitment root lives in the object noun at a fixed structure. four Goldilocks
field atoms at axes [4, 10, 22, 23] form the four 64-bit limbs of C_t:

```
BBG_ROOT_LIMB_AXES: [u64; 4] = [4, 10, 22, 23]

C_t[0] at axis  4   → passed to LookProvider as `commitment`
C_t[1] at axis 10   → recorded in trace row r[11]
C_t[2] at axis 22   → recorded in trace row r[12]
C_t[3] at axis 23   → recorded in trace row r[13]
```

see [[object]] for the full object layout and how these axes resolve.

if any axis navigation fails (object is an atom, or the path hits an atom before
the target), the pattern returns `⊥_unavailable`.

## trace row layout

```
r[ 4]  C_t[0]     first limb of the BBG commitment root
r[ 5]  ns         namespace (0..9)
r[ 6]  key        lookup key
r[ 7]  value      result (u64 of the Goldilocks element), or NIL if unavailable
r[11]  C_t[1]     second limb
r[12]  C_t[2]     third limb
r[13]  C_t[3]     fourth limb
```

`r[7] = NIL` (u32::MAX as u64) is the sentinel for "no value" — distinguishes
an unavailable result from a successful read of the field element 0.

bbg/'s `collect_look_openings` reads columns r[4..7] and r[11..13] from the trace
to reconstruct the full C_t and assemble `LookOpening` structs.

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

namespaces 0–9 are the 10 public evaluation dimensions of BBG_poly. private state
(commitment polynomial A(x), nullifier polynomial N(x)) is not accessible via
look — private records require call (pattern 16) with ZK witness injection.

## properties

- **deterministic**: same namespace, same key, same C_t always returns the same
  value. the polynomial is committed — its coefficients are fixed by C_t
- **pure**: no side effects. look reads state but never modifies it
- **memoizable**: look results are fully cacheable. given the same C_t, namespace,
  and key, the result is identical
- **read-only**: look never modifies BBG state, the object noun, or C_t

## comparison with call (16)

| property | call (16) | look (17) |
|----------|-----------|-----------|
| determinism | non-deterministic — prover chooses witness | deterministic — polynomial determines value |
| output | multiple valid witnesses may satisfy the check | exactly one correct value for given inputs |
| memoizable | no | yes — same C_t + namespace + key = same result |
| verification | check formula validates witness | proof accumulated via ProofLookProvider |
| state access | external witness injection | committed polynomial evaluation |
| confluence | intentionally broken | preserved |

## cost model

```
component            cost
─────────            ────
dispatch             1
namespace evaluation cost(ns_f)
key evaluation       cost(key_f)
provider call        0 (not a budget cost — provider is an external service)
─────────            ────
total                1 + cost(ns_f) + cost(key_f)
```

proof accumulation (LookOpening assembly) happens after execution and is not part
of the reduction budget. the verifier checks the opening via the proof system, not
by re-evaluating during reduction.

## error cases

```
⊥_unavailable (namespace out of range):
  namespace > 9
  validated before calling the provider.
  the provider is never called for invalid namespaces.

⊥_unavailable (C_t extraction failed):
  the object noun does not contain field atoms at axes [4, 10, 22, 23].
  happens when the object is an atom, or a path hits an atom mid-way.

⊥_unavailable (provider returned None):
  the provider has no value for (C_t[0], namespace, key).
  may indicate a cache miss, a node that has not synced this state,
  or a query against an epoch the local node does not hold.
```

all error cases produce `⊥_unavailable`, not `⊥_halt`. look failures are
deterministic — the same malformed input always produces the same error.

## what look enables

```
state queries:     look reads neuron balance, particle energy, token supply

conditional:       formulas branch on current BBG state
                   e.g., check balance before constructing a transfer

cross-dimension:   look from axons_out + look from axons_in
                   consistency is structural — same polynomial, different dimensions

temporal:          look in the time dimension yields historical snapshots
                   diff between two temporal lookups reveals changes
```

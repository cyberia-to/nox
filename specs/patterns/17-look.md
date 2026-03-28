# look pattern (17) — deterministic BBG read

```
reduce(o, [17 key_f], f) =
  1. key = reduce(o, key_f, f - 1)           // evaluate key expression
  2. value = bbg.read(key)                    // deterministic read from BBG
     if value == ⊥_unavailable → return ⊥_unavailable
  3. return (value, f')
```

deterministic read from the authenticated state layer (BBG). given a key, returns the value committed in the current BBG state. the result is deterministic — the same key at the same block height always returns the same value. unlike call (pattern 16), look does not involve prover choice.

the verifier checks the BBG inclusion proof (NMT completeness or mutator set membership) via the stark proof.

## interface

```
trait LookProvider {
    fn read(&self, key: NounId) -> LookResult;
}

enum LookResult {
    Value(NounId),
    Unavailable,
}
```

## key space

look reads from the 9 public NMT sub-roots of BBG: particles, axons_out, axons_in, neurons, locations, coins, cards, files, time. the key encodes the sub-root index and the lookup path within that namespace.

```
key = cell(sub_root_index, path)

sub_root_index:
  0  particles
  1  axons_out
  2  axons_in
  3  neurons
  4  locations
  5  coins
  6  cards
  7  files
  8  time
```

## properties

- deterministic: same key at same block height always returns same value
- memoizable: look results are fully cacheable (deterministic)
- verifiable: BBG NMT inclusion proof accompanies the value
- read-only: look never modifies state (write is implicit via order result = cyberlink)
- unavailable = error: missing key produces unavailable error, not a prover-dependent halt

## cost

look dispatch: 1. key evaluation: cost of key_f. BBG read: 1 (O(1) NMT proof verification via Lens opening). total: 1 + cost(key_f) + 1.

## what look enables

```
state queries:    look reads neuron balance, particle energy, token supply
                  Layer 1 computes over the result deterministically

conditional logic: formulas branch on current BBG state
                   e.g., check balance before constructing a transfer

cross-index joins: look from axons_out + look from axons_in
                   LogUp consistency verified in the proof
```

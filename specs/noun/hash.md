# noun hash


## structural hash

every noun has a canonical hash computed by H (the instantiated hash function). type and structure information is embedded in the hash function's capacity region — not prepended to the input.

### canonical (Hemera)

domain separation via Hemera's sponge capacity — the same mechanism Hemera uses for leaf/node/root distinction in Merkle trees.

```
H(atom a)     = hemera_leaf(encode(a), capacity[14] = type_tag(a))
H(cell(l, r)) = hemera_node(H(l), H(r))
```

capacity layout for noun hashing:
- capacity[14] = atom type tag (0x00 field, 0x01 word, 0x02 hash) — atoms only
- capacity[9]  = FLAG_CHUNK for atoms, FLAG_PARENT for cells (Hemera tree flags)

the hash output is 32 bytes (4 field elements). no prefix bytes, no framing — the type is inside the permutation.

NOTE: the recursive hemera hash described above is the legacy structural hash. the canonical identity computation uses Lens commitments — see identity section below. the recursive hash remains as the semantic definition (what the identity means). the Lens-based computation is the implementation (how it is computed efficiently). both produce the same identity for the same noun.

properties:
- deterministic: same noun always produces same hash
- collision-resistant: distinct nouns produce distinct hashes (Poseidon2 security)
- composable: cell hash depends only on child hashes, enabling incremental computation
- domain-separated: different atom types produce different hashes for the same value, enforced by the sponge capacity — not by input framing

## identity

every noun's identity is computed as:

```
identity = hemera(Lens.commit(noun_polynomial) ‖ domain_tag)     32 bytes
```

one hemera call wraps the Lens commitment with a domain separation tag. the Lens commitment itself is O(d × N) field operations where d = expander degree (~6-10) and N = number of leaves. this is the Brakedown linear-time commitment.

for small nouns (≤56 bytes / ≤7 field elements): cost is comparable to a direct hemera absorption — the Lens commitment over a few elements is negligible.

for large nouns (>56 bytes): cheaper than recursive hemera hashing. field operations replace multiple hemera permutation calls. a 4 KiB noun: ~512 leaves × ~8 field ops = ~4,096 field ops for Lens.commit, plus 1 hemera call for the identity wrap. recursive hemera would require ~64 permutations × ~200 field ops = ~12,800 field ops.

one identity scheme for ALL nouns. no size threshold. no dual paths. atom or cell, 8 bytes or 8 MiB — same computation: Lens.commit the polynomial, hemera-wrap the commitment.

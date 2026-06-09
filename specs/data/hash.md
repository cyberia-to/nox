# data hash

## structural hash

every data node has a canonical hash computed by H (the instantiated hash function). structure information is embedded in the hash function's capacity region — not prepended to the input.

### canonical (Hemera)

domain separation via Hemera's sponge capacity — the same mechanism Hemera uses for leaf/node/root distinction in Merkle trees.

```
H(atom a)      = hemera_leaf(encode(a))
H(pair(l, r))  = hemera_node(H(l), H(r))
```

capacity layout for data hashing:
- capacity[9]  = FLAG_CHUNK for atoms, FLAG_PARENT for pairs (Hemera tree flags) — second-preimage safety. KEEP.
- capacity[14] = unused. no tag framing.

atom encoding: 8 bytes, value as little-endian u64. domain = 0. no type tag byte.

the hash output is 32 bytes (4 field elements). this is the `particle` — the content-derived identity of data.

properties:
- deterministic: same data always produces same particle
- collision-resistant: distinct data produces distinct particles (Poseidon2 security)
- composable: pair hash depends only on child hashes
- domain-separated: FLAG_CHUNK / FLAG_PARENT in capacity[9] separates leaf from node hashes

## identity

every data node's identity is computed as:

```
particle = hemera(Lens.commit(data_polynomial) || domain_tag)     32 bytes
```

one hemera call wraps the Lens commitment with a domain separation tag. O(d × N) field operations where d = expander degree (~6-10) and N = number of leaves. Brakedown linear-time commitment.

one identity scheme for ALL data — atom or pair, 8 bytes or 8 MiB — same computation: Lens.commit the polynomial, hemera-wrap the commitment.

---
tags: nox
crystal-type: entity
crystal-domain: comp
alias: object, nox object, computation context
---
# object

the object is the data environment for a nox computation. every reduction
`reduce(object, formula, budget)` receives one object. formulas navigate
the object via axis (pattern 0) and read committed state via look (pattern 17).

## structure

```
object = [[l0 | [l1 | [l2 | l3]]] | rest]

axis 2  =  [l0 | [l1 | [l2 | l3]]]   — BBG commitment root (4 limbs)
axis 3  =  rest                        — computation payload

axis  4  =  l0   — C_t limb 0
axis 10  =  l1   — C_t limb 1
axis 22  =  l2   — C_t limb 2
axis 23  =  l3   — C_t limb 3
```

`l0`, `l1`, `l2`, `l3` are four Goldilocks field atoms. together they form the
four 64-bit limbs of the BBG commitment root C_t — the hash
`H(Lens.commit(BBG_poly) ‖ Lens.commit(A) ‖ Lens.commit(N))` split across four
field elements.

`rest` is the payload — the data the computation operates on. its structure
depends on the application; nox does not constrain it.

## axis derivation

axis navigation follows the binary path of the address:

```
address    binary     path from object root
────────   ────────   ──────────────────────────────────────────────
4          100        left → left                   → l0
10         1010       left → right → left           → l1
22         10110      left → right → right → left   → l2
23         10111      left → right → right → right  → l3
```

the leading 1 bit is skipped (it marks the depth, not a direction). the remaining
bits give the left/right path: 0 = go left (head), 1 = go right (tail).

see [[patterns/0-axis]] for the full axis navigation algorithm.

## BBG root limbs

```
BBG_ROOT_LIMB_AXES: [u64; 4] = [4, 10, 22, 23]
```

the constant is defined in `nox/rs/patterns/look.rs` and used by the look
pattern (17) to extract C_t before calling the `LookProvider`.

C_t carries the entire authenticated state commitment. it travels with the
computation — every object embeds the BBG root at the time the computation
begins. the look pattern verifies reads against this root; the prover cannot
substitute a different C_t without changing the object (which changes the
computation input).

## construction

```rust
// pseudo-code: build an object in an Order
fn make_object(order, l0, l1, l2, l3, rest) -> OrderId {
    let inner     = order.pair(l2, l3);      // [l2 | l3]
    let mid       = order.pair(l1, inner);   // [l1 | [l2 | l3]]
    let root_pair = order.pair(l0, mid);     // [l0 | [l1 | [l2 | l3]]]
    order.pair(root_pair, rest)              // [[l0 | [l1 | [l2 | l3]]] | rest]
}
```

all four limbs must be field atoms. `rest` is arbitrary data.

## invariants

- C_t extraction (look pattern step 3) returns `⊥_unavailable` if any of axes
  4, 10, 22, 23 does not resolve to a field atom in this object
- the look pattern calls the provider with `C_t[0]` (l0) as `commitment`
- `C_t[1..3]` are recorded in the trace but not passed to the provider; they
  bind the full commitment in the proof without increasing the provider interface
- the prover constructs C_t from `H(Lens.commit(BBG_poly) ‖ Lens.commit(A) ‖ Lens.commit(N))`
  (see [[bbg/state]] for the BBG root definition)

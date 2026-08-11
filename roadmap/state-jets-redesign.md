---
status: draft
date: 2026-06-09
author: jet taxonomy review session
---

# state jets redesign — track the evolved cybergraph

## Abstract

The state jet set (`specs/jets/state.md`) encodes a key-value-database
mental model: `cyberlink` (exact) plus `transfer` / `insert` / `update` /
`aggregate` / `conserve` (templates), over five primitives READ / WRITE /
ASSERT_EQ / ADD / MUL. Cybergraph has since evolved into a UTXO-box +
polynomial-commitment + private-state model whose single primitive is the
cyberlink and whose single mutation is `insert(signal)`. Three of the five
templates describe operations that no longer exist.

Genesis jets are frozen by axiom A3 (append-only). Freezing a state circuit
carved against a dead model is the one mistake that is expensive to undo.
This proposal rewrites the state jet set against the current cybergraph
axioms (A1–A6) and BBG state model before any genesis commitment.

## 1. The drift

The state jet spec predates the current cybergraph. The model it was carved
against is gone.

### what cybergraph is now

One primitive — the cyberlink, a 5-tuple:

```
cyberlink(from: TokenId, to: TokenId, token: TokenId, amount: u64, valence: {-1,0,+1})
```

`from`/`to` are structural participants, `token` is the denomination,
`amount` is conviction stake, `valence` is a BTS epistemic prediction.
Transfer, withdraw, staking, burning, and aggregation are **not separate
operations** — they are cyberlinks with different field values.

One mutation — BBG exposes `insert(signal)`, where:

```
signal = (ν, links, Δφ*, σ, height)
       = neuron + cyberlinks + proven focus shift + zheng proof + block height
```

The model runs on:

- box model (UTXO-like) — cyberlinks destroy input boxes, create output boxes
- public/private split — `to` is a stealth address → encrypted box in `A(x)`;
  direct id → plaintext in `balances`. One signal mixes both under one proof.
- nullifier polynomial `N(x)` = ∏(x − nᵢ) for spent-record tracking
- six axioms A1–A6 + conservation A5 (Σφ* = 1) enforced in-circuit
- 11 public evaluation dimensions in `BBG_poly`

### the mismatch

| nox state jet | status against evolved cybergraph |
|---|---|
| READ / WRITE / ASSERT_EQ / ADD / MUL | ✅ valid — map to Lens.open / insert / eq / add / mul |
| `cyberlink` (exact) | ⚠️ right name, wrong shape — old 2-tuple `[from\|to]`, not the 5-tuple signal |
| `transfer` template | ❌ not a distinct op — a cyberlink field pattern |
| `insert` template | ❌ collides with BBG's `insert(signal)`; not a graph op |
| `update` template | ❌ A3 is append-only — links are never updated, only down-weighted |
| `aggregate` template | ❌ implicit via axiom A6 (axon-particle), not an explicit jet |
| `conserve` template | ⚠️ real (A5) but belongs inside the signal circuit, not standalone |
| — | ❌ missing: box spend/create, nullifier check, public/private dispatch, valence/BTS |

The five primitives survive. The six jets do not.

## 2. What survives

READ / WRITE / ASSERT_EQ / ADD / MUL remain the correct Layer 1
decomposition. They are not jets — they are the pattern expansion every
state circuit reduces to:

| primitive | nox patterns | role |
|---|---|---|
| READ(dim, key) → value | Lens.open (polynomial evaluation) | O(1) eval at (dim, key) |
| WRITE(dim, key, value) | axis (0) + cons (3) | build updated tree |
| ASSERT_EQ(a, b) | eq (9) | check field equality |
| ADD(a, b) → c | add (5) | field addition |
| MUL(a, b) → c | mul (7) | field multiplication |

These stay. 18 patterns are the axiom; state operations are derived.

## 3. Proposed state jet set

Collapse the six CRUD-era jets to the actual primitive plus the circuits
the box model requires.

### one signal jet (exact match)

Replaces `cyberlink` + `transfer` + `update` + `aggregate` + `conserve`.
Validates a signal against the axioms:

| check | axiom | constraints (target) |
|---|---|---|
| signature valid for ν | A2 (authentication) | — |
| links append-only | A3 (append-only) | — |
| participants exist iff linked | A4 (entry) | — |
| axon-particle H(from,to) induced | A6 (homoiconicity) | — |
| Σφ* = 1 after Δφ* applied | A5 (conservation) | — |

The signal jet is the cyberlink circuit, correctly shaped to the 5-tuple
and the impulse Δφ*. The 2-tuple `[from|to]` of the old `cyberlink` jet is
the shape error this corrects.

### box sub-circuits

The UTXO model the evolved cybergraph runs on:

| jet | role |
|---|---|
| `box_spend` | destroy input box; authorize spend from `from` |
| `box_create` | create output box at `to` |
| `nullifier_check` | non-membership against `N(x)` (no double-spend) |

### public/private dispatch

Not a separate jet — a branch inside `box_create`: `to` stealth →
encrypted record extends `A(x)`; `to` direct id → plaintext `balances[H(to‖token)]`.
One signal mixes both under one zheng proof.

### fallback

Unrecognized formulas: full nox execution trace → SuperSpartan proof
(~8 constraints per row). Unchanged.

## 4. Ownership

State is one of the registry's products (see [[decider-product]] for the
product principle): the cybergraph state machine. By the boundary rule
applied across the jet set, the authenticated state layer is bbg's. The
signal/box/nullifier circuits are bbg-owned; nox holds the formula-hash
anchors and the wrapper that delegates to the bbg state provider (today's
`CallProvider`). This mirrors decider→zheng ([[decider-product]]) and
hash→hemera — grouping by product, ownership by domain repo.

The current `rs/jets/state.rs` (transfer/insert/update/aggregate/conserve
delegating to `CallProvider`) is scaffolding against the dead design. It
must not ship as frozen genesis jets — the templates encode operations that
do not exist in the model bbg now implements.

## 5. Migration

1. Rewrite `specs/jets/state.md` against axioms A1–A6, the box model, and
   the 11 BBG dimensions. Define the signal jet (5-tuple + Δφ*), the three
   box sub-circuits, and the public/private dispatch.
2. Cross-check the cyberlink shape with `cybergraph/specs/` and
   `bbg/specs/` — the 5-tuple, signal structure, and `BBG_poly` dimensions
   are the source of truth; the spec must quote them, not restate from
   memory.
3. Replace the formula builders in `rs/jets/formulas.rs`
   (`build_cyberlink_formula` and the five template tags) with the signal
   and box formulas.
4. Re-implement `rs/jets/state.rs` wrappers against the new formulas,
   delegating circuit logic to bbg.
5. Recompute genesis digests; update `compute_genesis_digests`.

## 6. Why this blocks genesis

Genesis jets are frozen in the genesis BBG state by A3 (append-only). A
frozen state circuit cannot be revised — only deprecated and worked
around. The recursion jets are sound and can freeze today. The state jet
set, as written, would freeze a 2021-era CRUD model into permanent genesis
state. This is the single strongest "not ready for prime time" argument:
correctness of the state circuits is a soundness property of the chain, and
the current set is carved against a model bbg no longer implements.

## 7. Open questions

1. Is `valence` (BTS epistemic prediction) validated in-circuit at genesis,
   or deferred? It affects the signal jet's constraint count.
2. Does `box_spend`/`box_create` warrant separate formula hashes, or one
   `box_transition` jet with a direction field? (Leaning: one jet, direction
   field — fewer frozen anchors.)
3. Conservation A5 (Σφ* = 1) via the tri-kernel — is the stochastic update
   proven inside the signal jet, or as a separate per-block circuit? The
   tri-kernel is a composite (diffusion + springs + heat); its proof shape
   needs cross-checking with bbg.
4. Coordinate the genesis-digest source of truth with [[decider-product]]
   — both proposals move circuit ownership out of nox while keeping anchors.

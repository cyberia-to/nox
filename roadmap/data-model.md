---
status: draft
date: 2026-06-07
author: value-model design session
---

# nox data model — applying the stack type theory

## Abstract

nox is the value layer the stack's type theory sits above. The universal
framework — the five mechanisms, the allocation (two dropped, one
reserved, two open), the two open type systems — lives in
`soft3/specs/types.md`; the vocabulary lives in `soft3/specs/terms.md`.
This document is the nox instantiation: the substrate, how type leaves the
value layer, how identity is allocated, how data is encoded, and a
migration grounded in a survey of the current code (nox, hemera, zheng,
bbg, tape, trident, inf).

nox is unfinished; this supersedes the parts of `noun/`, `encoding.md`,
and `noun/hash.md` it contradicts. The survey found the spec already
drifted from the code in two load-bearing places — both in the direction
this corrects. Two vocabulary changes from the old spec: `noun → data`
and `cell → pair`.

## 1. the substrate — Layer 0

Two words do all the structural work:

- `atom` — a leaf: one `field`.
- `pair` — two children joined, each an `atom` or a `pair`. the constructor.
- `data` — an `atom` or a `pair`. any structure, `8·N` bytes. homoiconic:
  a program, its input, and its result are all data.
- `particle` — the identity of data: its 32-byte Hemera hash (itself
  4-atom data). content-derived and immutable. (a `name` is a separate,
  mutable label that points to a particle — see `soft3/specs/terms.md`.)

`field` and `pair` are the *grammar of values, not a type system* (see
`types.md`). The kernel knows one thing — data — and imposes no type
system, which is what lets inf, trident, and rune each type over it.
Type leaves the value layer entirely: `word`/`bool` are refinement
predicates; nominal types are `pair(domain, value)`.

### canonical representation (Model B, not Model A)

The spec carries two models and they disagree; pick the one the prover
actually uses.

- Model A (legacy, `encoding.md` storage): a `pair` stores its children's
  *hashes* (two 32-byte particles). This forces every small composite to
  64 B, makes a 4-field digest a 7-node tree, and has no 16 B node.
- Model B (canonical, `inner.md` / `noun/hash.md`): data *is* its field
  leaves; a `pair` is structure (a polynomial variable); identity =
  `hemera(lens.commit(leaves))`. `noun/hash.md` already declares the Lens
  commitment canonical and the recursive child-hash "legacy."

Adopt Model B. Then data is stored by its leaves, sized `8·N`, so the size
spectrum is the full ladder — `8` (atom), `16` (a pair of atoms), `32` (a
particle), `64` (a pair of particles), … — and a particle is 4 leaves,
not 7 nodes. To reference other data you embed its `particle` (4 leaves),
not a special child-hash slot.

## 2. records — Layer 1

Structured data with named fields, built on the substrate — not size
rungs:

- `link` — a bare `(from, to)` pair of particles. the skeleton of an edge.
- `cyberlink` — `(from, to, token, amount, valence)` asserted by a
  `subject` (neuron) at a time. far more than a `link`: it carries
  economics, epistemics, and identity. the graph's primitive.
- `file`, `signal`, … — other Layer-1 records.

`cell` is a Layer-1 word, reserved for app structure — never the Layer-0
`pair`.

## 3. identity

Spec correction. `noun/hash.md` claims `capacity[14]` carries the atom
type tag. The survey found **hemera never implements `capacity[14]`** —
the tag is carried by *input framing*, `data[8] = tag` at
`nox/rs/noun/hash.rs:26`, then passed to the leaf hash. That is the
*framing* mechanism `types.md` bans, sitting in identity.

- **Drop the tag from the hash input** (`hash.rs:26-27`). Then `field 5`
  and `word 5` produce the same particle; the survey confirmed no code
  depends on them differing.
- **Keep `capacity[9]`** — FLAG_CHUNK / FLAG_PARENT / FLAG_ROOT, the
  leaf/node/root separation. Real, implemented (`hemera/rs/src/tree.rs`),
  the one genuine invariant (second-preimage safety).
- **Nominal distinctness via content**, `pair(domain, value)`, never a tag.

Net: identity reflects content (structure), ignores refinement.
`word 5 == field 5`; `timestamp 5 ≠ count 5` (different data).

## 4. encoding

The storage encoding prepends a tag byte (`field 9B`, `word 9B`,
`hash 33B`, `cell 65B`). The tag is the framing mechanism leaking into
storage, and the `33` fights the 32-byte particle quantum. Under Model B:

- store data by its **leaves** (`8·N`), with the tree shape; the tag byte
  goes — structure is read from shape/length, not a type tag.
- **reference** other data by embedding its `particle` (4 leaves), not a
  child-hash slot. This drops the legacy Model-A 64-B-pair format.
- a particle stores flat (32 B inline), hashing identically to its leaf
  commitment — the optimization that avoids the cell-of-4 blowup.
- transport framing stays a length varint (tape), never identity.

Blast radius (survey): contained to `nox/rs/encode.rs` — `SIZE_*`
(`60-63`), the `decode` match + slices (`198-259`), the wire size-check
(`349`). bbg is decoupled (particles are opaque 32-byte hashes); tape is
decoupled (LEB128 framing).

## 5. vocabulary

| concept | name | what it is |
|---|---|---|
| scalar | `field` | one Goldilocks element |
| leaf | `atom` | one `field` |
| join | `pair` | two children (was `cell`) |
| value | `data` | `atom` or `pair` (was `noun`) |
| 32-bit | `word` | a `field` proven in `[0, 2³²)` |
| boolean | `bool` | a `field` proven in `{0, 1}` |
| identity | `particle` | the 32-byte Hemera hash of data; `neuron` is a particle in the actor role |

`hash` is the **verb** (Hemera permutation); `particle` is what it
produces. The core is mono-hash, so it needs no `digest` — `digest`
survives only at trident's target boundary (`[Field; D]`, `D =
digest_width`). `Cid` is removed (a stale alias for `particle`).

## 6. decisions, and two calls

Resolved: substrate = `atom` + `pair` → `data`, named by a `particle`;
types are refinement + nominal, both above the kernel; identity reflects
content / ignores refinement; encoding is leaf-based and tag-free.

Two calls (author to flip): **Model B (leaf storage) — adopt** (gives the
`8·N` spectrum, kills the blowup). **`capacity[9]` — keep** (the one
identity invariant).

## 7. migration — making the jump clean

The survey shows three of four axes are far cleaner than feared; the
fourth is work that must happen regardless. Sequencing in
`soft3/roadmap/migration.md`.

### M1 — vocabulary (mechanical, no soundness impact)

- nox specs/code: `noun → data`, `cell → pair`, name the hash output
  `particle`.
- trident: `U32 → Word` on the surface (`ast/mod.rs:193`,
  `typecheck/types.rs:13` + ~60 `Ty::U32` sites; leave `cost/scorer.rs`
  `const U32` — a Triton table index; `Digest` unchanged). One lowering
  note: "`Word` lowers to the Triton u32 table."
- inf: one site (`specs/language.md`).

### M2 — encoding (contained, single file)

`nox/rs/encode.rs`: switch to leaf-based, tag-free; update `SIZE_*`, the
`decode` match + slices, the wire size-check. bbg and tape unchanged.

### M3 — identity (contained, no external dependents)

Remove `data[8] = tag` and the tag argument (`hash.rs:26-27`); collapse
`Tag`. Keep `capacity[9]`. Fix the `noun/hash.md` `capacity[14]` drift in
the same change.

### M4 — word as a refinement (the real work, spec-first, sign-off)

Word-gated patterns (10–14) are unsound in-circuit today: zheng ships
per-row gadgets (`zheng/src/ccs/patterns.rs:285-361`) but the
decomposition bindings (`Σ 2^k·a_k = operand`) and most booleanity checks
are missing, and the 2-row CCS window (`ccs/mod.rs:84-121`) can't express
a 32-row sum. The range guarantee lives only in the interpreter tag
(`reduce.rs:366-411`). Minimal sound path: add a running-accumulator
column (fits the 2-row fold) + booleanity; the 32-row binding *is* the
refinement range proof, making the tag redundant. Resolve the `shl`
spec conflict (`trace.md` vs `constraints.md:310-321`) first.

M1/M2/M3 are independent and low-risk; M4 is its own spec-first effort
that closes an existing gap and retires the tag as a by-product.

## 8. pre-existing issues this surfaced (fix regardless)

- `noun/hash.md` `capacity[14]` does not match the code (framing `data[8]`).
- in-circuit soundness gap in word-gated ops (10–14), independent of this.
- `trace.md` vs `constraints.md` disagree on `shl`.
- `lt` under-constrained; 64-bit canonical form unspecified.
- two models (A storage vs B Lens) coexist in the spec — pick B.
</content>

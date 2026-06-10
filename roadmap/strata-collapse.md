---
status: draft
date: 2026-06-10
author: strata/nox boundary review session
---

# strata collapse — five algebras as primitives + jets

## Abstract

strata is not a layer below nox. It is two things wearing one coat: the
*implementation of nox's primitive arithmetic* (patterns 5–15) and a *jet
table* (fast composite formulas). This document resolves the apparent
recursion — strata feels fundamental, yet also feels like jets — by showing
that four of the five algebras need nothing new in nox: they are already nox
patterns (nebu) or jets over existing patterns (kuro, jali, trop). Only
`genies` (F_q) is a genuinely new primitive, and nox already models it as a
separate instantiation `nox<F_q>`. The consequence is architectural, not new
math: there is no standalone "strata layer" to depend on — there is nox's
primitive backend and the genesis jet registry, and the strata crates are
reclassified into those two roles.

References: `specs/jets.md` (jet contract, genesis registry),
`specs/patterns/` (the 18 patterns), `docs/explanation/five-algebras.md`
(irreducibility argument), `strata` repo (the five crates).

## 1. the claim

nox expresses all compute. Nothing is below it semantically. So strata,
which "everything reduces to," cannot be a foundation *beneath* nox — that
would put two floors under one building. The resolution: strata's content
splits exactly along nox's own primitive/jet boundary.

- A **primitive** is a pattern — the irreducible op the silicon does. It has
  no pure expansion to optimize, so it can never be a jet.
- A **jet** is a composition of patterns recognized by formula-hash and run
  fast. Its pure expansion *is* the spec it must match (`specs/jets.md`:
  "every jet has an equivalent pure Layer 1 program producing identical
  output").

Every strata operation is one or the other. There is no third category, and
therefore no third layer.

## 2. the collapse

nox already holds the primitives for four of the five algebras:

- F_p arithmetic (add, sub, mul, inv, eq, lt) = patterns 5–10
- bitwise ops (xor, and, not, shl) = patterns 11–14
- structural hash (hemera) = pattern 15

With those in place the five algebras fold:

| algebra | field | what it becomes | new nox primitive? |
|---------|-------|-----------------|--------------------|
| nebu | F_p | already patterns 5–15; `ntt`, `fri_fold`, `batch_inv`, `poly_eval` → jets | none |
| kuro | F₂ | add = xor; mul = carryless mul over and/xor → jets over patterns 11–12 | none |
| jali | R_q | NTT decomposes R_q into n copies of F_p → jets over nebu | none |
| trop | (min,+) | min = `branch(lt(a,b), a, b)` → jets over patterns 4, 10 | none |
| genies | F_q | foreign prime; cannot reduce to Goldilocks | **F_q add, mul** |

Four of five require nothing new. This is already latent in the codebase:
`docs/explanation/five-algebras.md` states tropical "decomposes to existing
patterns — the proof happens in F_p, the tropical computation is the
witness," and that jali's ring ops "decompose via NTT into n copies of F_p."
This document names the consequence: those crates are jet implementations,
not a substrate.

## 3. the irreducible residue

What cannot be a jet — the operations with nothing below them to expand into:

- Goldilocks **add** and **mul** (F_p) — already patterns 5 and 7.
- F_q **add** and **mul** (genies) — not yet a nox primitive.
- the bitwise op (F₂ / the W word) — already patterns 11–14.

So the entire irreducible floor of all five algebras is: the arithmetic of
two prime fields plus the bit ops. Of these, only F_q is missing from nox.

## 4. F_q is the one new primitive

F_q (CSIDH-512 prime) cannot fold into Goldilocks: simulating mod-q
arithmetic as bignum over F_p limbs is the ~64× emulation tax, and the
isogeny group action has no construction over F_p at all
(`docs/explanation/five-algebras.md`). So F_q must be a real primitive
substrate.

nox already models this: genies is "a separate instantiation `nox<F_q>` with
its own jet registry; cross-algebra composition via HyperNova folds F_q
sub-traces into the F_p accumulator." This proposal does not introduce that —
it confirms F_q as the *single* irreducible second floor. After the collapse,
nox is exactly two prime substrates:

```
nox<F_p>   — the anchor. patterns 5–15. all proofs settle here.
nox<F_q>   — the privacy substrate. its own add/mul primitive. folds to F_p.
```

Everything else — every other algebra, and every algorithm in all five — is a
jet over one of these two.

## 5. what "becomes a jet" means

Re-homing, not deletion. The fast implementations do not vanish:

- the algorithm code (AVX2 Goldilocks mul, CLMUL F₂ mul, NTT, Karatsuba,
  group action) stays — it moves from *standalone imported library* to *jet
  implementation registered in the genesis jet table*.
- the role changes: from an opaque Rust dependency nox trusts, to a
  genesis-committed accelerator with a pure-pattern spec it is provably
  equivalent to (`specs/jets.md`: jets are "never load-bearing for
  correctness, only for performance — remove them and the system is correct,
  only slower").

So the win is not less code. It is: the algebras stop being a trusted
external floor and become part of the verifiable protocol — each jet keyed by
the hash of its pattern expansion, committed in genesis BBG, frozen by axiom
A3, honored by every conforming implementation.

## 6. consequence for the strata repo

The five crates are reclassified by role, not rewritten:

| crate | role after collapse |
|-------|---------------------|
| nebu | reference implementation of patterns 5–15 (nox's F_p backend) + F_p jets |
| kuro | jet implementations over patterns 11–12 (F₂ add/mul) |
| jali | jet implementations over nebu (R_q via NTT) |
| trop | jet implementations over patterns 4, 10 (min,+) |
| genies | nox<F_q> primitive backend (F_q add/mul) + F_q jets |
| strata-core / proof / compute / ext | the trait surface the primitive backends and jets implement |

What disappears is the *idea* of strata as a layer nox imports from below.
What remains is: nox's two primitive backends (F_p, F_q), and a jet table
whose entries happen to live in five crates.

## 7. migration

This is a re-labeling and a boundary decision, stageable behind the existing
"jets deferred post-genesis" status (`specs/jets.md`). No interpreter change
is forced today.

1. **document the boundary.** State in `specs/jets.md` that each genesis jet's
   fast path is a named strata routine and its pure expansion is its spec; and
   that nebu is the F_p primitive backend, genies the F_q one. The other three
   crates are jet hosts.
2. **confirm the two-substrate model** in `specs/vm.md`: nox<F_p> and nox<F_q>
   are the only primitive instantiations; the remaining algebras are jets over
   them. Tropical and R_q lose any framing as independent substrates.
3. **F_q primitive.** Decide F_q's status explicitly: a first-class nox
   instruction set (nox<F_q> with native add/mul patterns) versus a
   delegated-prover boundary jet. The irreducibility argument points to
   first-class; the cost is a second pattern-5/7 implementation over F_q.
4. **strata repo CLAUDE.md** records the reclassification so the crates are
   maintained as "primitive backends + jet table," not as a layered library.

## 8. open questions

- **F_q as instruction vs delegated jet.** §4 argues first-class. But F_q
  appears only in privacy workloads (stealth addresses, VDF, blind sig). Is a
  permanent second instruction set worth it, or should F_q be a
  delegated-prover jet that the F_p decider folds, accepting the boundary
  cost (~766 F_p constraints per crossing)?
- **does kuro need a primitive at all?** F₂ add = xor (11), F₂ mul = carryless
  mul over and/xor. If carryless mul is a jet over patterns 11–12, kuro is
  *pure* jets — no primitive. Confirm no F₂ op is irreducible beyond the
  existing bit patterns.
- **jet expansion provability for the heavy routines.** group_action and
  isogeny_walk have large pure expansions. Is the observational-equivalence
  harness (`specs/jets.md`) tractable for them, or do they need a different
  soundness argument than the per-input comparison used for ntt/poly_eval?
- **naming.** if "strata" is no longer a layer, is the repo name still right,
  or does it become `nox-primitives` + a jets crate? Defer — naming follows
  the boundary decision, not the reverse.

---
name: restructure nox — freeze layout
description: eliminate roadmap/, move jets/ into specs/, migrate active proposals, update jet registry model
status: approved
date: 2026-03-26
---

# restructure nox — freeze layout

nox is frozen. the repo layout should reflect this: no roadmap/, jets as part of canonical spec, all active research migrated to explanation or audits.

## architectural decision: jet registry on cybergraph

jets live on the cybergraph, not hardcoded in binary.

- genesis jets committed in genesis BBG state — frozen by axiom A3 (append-only)
- conforming implementations MUST recognize genesis jets
- post-genesis jets registered through cyberlinks — design deferred
- binary contains: 16 patterns + hint + jet dispatch mechanism + genesis BBG state (which includes jet registry)

this replaces the previous "hardcoded protocol constant" model.

## moves

### 1. jets/ → specs/jets/

jet specifications are canonical — they belong in specs/.

| from | to |
|------|-----|
| jets/README.md | specs/jets/README.md |
| jets/nebu.md | specs/jets/nebu.md |
| jets/kuro.md | specs/jets/kuro.md |
| jets/jali.md | specs/jets/jali.md |
| jets/trop.md | specs/jets/trop.md |
| jets/genies.md | specs/jets/genies.md |
| jets/state.md | specs/jets/state.md |
| jets/decider.md | specs/jets/decider.md |

### 2. roadmap/ → migrate + delete

| file | action | reason |
|------|--------|--------|
| roadmap/algebra-polymorphism.md | delete | implemented, in specs/vm.md |
| roadmap/recursive-jets.md | delete | implemented, in specs/jets.md + specs/jets/nebu.md |
| roadmap/binary-jets.md | delete | implemented, in specs/jets.md + specs/jets/kuro.md |
| roadmap/decider-jet.md | delete | implemented, in specs/jets.md + specs/jets/decider.md |
| roadmap/transformer-jets.md | → docs/explanation/transformer-jets.md | research/explanation, not roadmap |
| roadmap/implementation-audit.md | → .claude/audits/implementation-audit.md | audit artifact |
| roadmap/README.md | delete | no longer needed |
| roadmap/ | delete directory | nox is frozen |

### 3. update specs/jets.md

- change "hardcoded protocol constant" → cybergraph registry with genesis entries
- update links: `[[jets/...]]` → relative paths within specs/jets/
- add: genesis jets = MUST, post-genesis = deferred

### 4. update indexes

- specs/README.md: add specs/jets/ to the index
- docs/explanation/README.md: add transformer-jets entry
- docs/explanation/jets.md: update links

### 5. consistency audit vs companion repos

check specs against:
- zheng: constraint counts, proof sizes, verifier costs
- hemera: hash parameters (rounds, state width, capacity)
- bbg: state operations, polynomial dimensions, evaluation model
- lens: PCS references (Brakedown, Binius, Ikat, Porphyry, Assayer)

## result

```
nox/
├── specs/           ← canonical specification (frozen)
│   ├── jets/        ← per-algebra genesis jet specs (NEW location)
│   └── *.md         ← existing spec files
├── docs/explanation/ ← conceptual docs (+ transformer-jets)
├── rs/              ← rust implementation
├── .claude/         ← agent state (+ implementation-audit)
└── (no roadmap/)
```

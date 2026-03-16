# nox reference audit — gaps blocking implementation

date: 2026-03-16
scope: reference/ vs docs/explanation/, reference completeness for implementation

## fixed in this audit

### hemera parameter errors (critical)

all instances corrected in reference/ and docs/explanation/:

| parameter | was | corrected |
|-----------|-----|-----------|
| state width | 12 elements | 16 elements |
| capacity | 4 elements | 8 elements |
| rounds | 8 full + 22 partial + 8 full | 8 full (4+4) + 64 partial = 72 |
| output | 4 × F_p (256 bits) | 8 × F_p (64 bytes) |
| hash atom wire size | 33 bytes (1+32) | 1 prefix + 64 data (hash data is 2⁶ clean) |

additionally: wire format prefix redesigned — merged atom/cell marker + type tag into single prefix byte (0x00-0x03). structural hash redesigned — type tag embedded in hemera capacity[14], not prepended as raw bytes.

files changed: vm.md, nouns.md, patterns.md, jets.md, encoding.md, docs/explanation/nouns.md, docs/explanation/completeness.md

### stark constraint discrepancies

field-patterns.md (explanation) had wrong constraint counts vs patterns.md (reference):

| pattern | explanation said | reference says | corrected to |
|---------|-----------------|---------------|-------------|
| inv | ~64 constraints | 1 | 1 (verify a × a⁻¹ = 1) |
| eq | ~32 constraints | 1 | 1 (zero-test) |
| lt | ~32 constraints | ~64 | ~64 (bit decomposition) |

### branch semantics

explanation said: 0 or 1 only, anything else → ⊥_error.
reference says: if t = 0 then yes, else no (any nonzero → no-branch).
corrected explanation to match reference.

---

## open gaps — must resolve before implementation

### G1: error type specification — RESOLVED

resolved: errors are not nouns — they are Result variants (transient computation outcomes, not persistent data). five error kinds defined (type_error, axis_error, inv_zero, unavailable, malformed). trace encoding: r15=status, r12=error_kind. Result encoding defined for protocol (success returns noun identity + focus, errors return status + kind). no need for noun store encoding of errors.

added to reduction.md: error specification, Result encoding, focus accounting examples.

### G2: hash type as noun — representation ambiguity — RESOLVED

resolved: hash is a wide atom (8 elements stored contiguously, type tag 0x02). it is an atom, not a cell. axis on a hash atom with index > 1 returns ⊥_error (same as axis on a regular atom). axis 0 returns its structural hash. axis 1 returns itself.

content-addressed storage: hash atoms store as 64 bytes (size-determined type). the structural hash uses hemera_leaf with capacity[14] = 0x02. no representation ambiguity remains.

nouns.md already states "axis on an atom (except 0 and 1) produces ⊥_error" — this applies to hash atoms unchanged.

### G3: multi-row trace layout for complex patterns — RESOLVED

resolved: trace.md now specifies multi-row patterns:
- hash (cost 300): ~75 rows (72 Poseidon2 rounds + absorption/squeeze). r12-r14 hold round state. degree 7 per row.
- inv (cost 64): 64 rows (square-and-multiply chain). r12 holds accumulator. degree 2 per row.
- compose/cons (cost 2): 2 dispatch rows + recursive sub-expression rows.

remaining: concrete register values for intermediate rows of hash and inv (needed for AIR constraint implementation).

### G4: focus semantics — type and overflow — RESOLVED

resolved: focus IS F_p. the Halt guard `if f < cost then Halt` prevents subtraction from ever wrapping. comparison uses integer ordering on canonical representatives in [0, p). since practical focus values are tiny relative to p ≈ 2^64, wrapping is impossible. keeping focus as F_p means the AIR constraint `r7 = r6 - cost` is a degree-1 field equation with no range check overhead.

clarification added to reduction.md.

### G5: structural hash input format — RESOLVED

resolved: structural hash uses hemera's native tree mode with capacity-based domain separation.

```
H(atom a)     = hemera_leaf(encode(a), capacity[14] = type_tag(a))
H(cell(l, r)) = hemera_node(H(l), H(r))
```

atoms use hemera_leaf with the atom type tag in capacity[14]. cells use hemera_node (which absorbs two child hashes via field addition, not byte concatenation). this eliminates raw byte prefixes from the hash input, uses hemera's existing domain separation mechanism, and produces clean 64-byte output.

remaining: add test vectors for each case (atom field, atom word, atom hash, cell).

### G6: axis on evaluated sub-expressions — RESOLVED

resolved: patterns.md now specifies: axis index must be a field-type or word-type atom, interpreted as an integer. cell or hash-type → ⊥_error.

### G7: jet recognition mechanism — RESOLVED

resolved: jets.md now specifies jet recognition. jet registry is hardcoded (protocol constant), maps H(formula_noun) → jet_implementation. recognition at reduction time. canonical hashes are generated at build time from pure Layer 1 definitions — correct by construction.

remaining: actual hash values computed once VM implementation can build and hash the pure formula nouns.

### G8: domain separation tag usage — RESOLVED

resolved: vm.md now specifies the mechanism. tags are injected into Hemera's capacity[11] (domain tag slot). the VM sets the tag based on calling context (structural hash, commitment, nullifier, Merkle, owner). tags are protocol constants, not user-configurable.

### G9: pattern cost overhead vs sub-expression cost

patterns.md says compose costs 2, cons costs 2, branch costs 2. but the actual focus spent includes sub-expression evaluation. the spec shows:

```
reduce(s, [2 [x y]], f) =
  let (rx, f1) = reduce(s, x, f - 2)
  let (ry, f2) = reduce(s, y, f1)
  reduce(rx, ry, f2)
```

is the "cost: 2" the overhead deducted before sub-expressions? focus accounting example added to reduction.md. test vector discrepancy identified: add(axis2, axis3) expected cost needs verification against axis depth cost formula. the pseudocode is unambiguous — cost is deducted before sub-expressions. but axis cost formula (1+depth) vs (1) for depth-1 access needs implementation test.

**status**: PARTIALLY RESOLVED — pseudocode clear, test vector needs verification against implementation.

### G10: Result type encoding — RESOLVED

resolved: Result is not a noun. errors are transient computation outcomes, not persistent data. Result encoding defined in reduction.md:
- success: (H(result), focus_remaining) — noun identity in store
- halt: (status=1, focus_remaining) — no noun
- error: (status=2, error_kind) — no noun

trace encodes Result in r15 (status) and r12 (error kind). instance includes H(result) for success. no separate wire format needed.

---

## docs/explanation vs reference — consistency issues (non-blocking)

### D1: lineage.md says Nock has "natural numbers + increment"

reference does not define Nock's instruction set. this is fine for explanation (background context). no action needed.

### D2: why-nox.md mentions "π" as focus parameter

`reduce(s, f, π)` uses π for focus in one place but all other docs use `f`. minor inconsistency. non-blocking.

### D3: completeness.md four-bit encoding claim

completeness.md claims pattern tags fit in 4 bits (0-15) and hint (16) is outside the encoding. but the formula encoding in encoding.md and nouns.md shows formulas as `cell(tag, body)` where tag is an atom. atom 16 is valid. the "4-bit encoding" claim is about the deterministic patterns only (Layer 1), not about the wire format. this is accurate but could confuse implementers.

### D4: docs reference wiki-style links ([[term]])

explanation docs use `[[term]]` links extensively (e.g., [[stark]], [[cybergraph]], [[neuron]]). these are not resolvable in the current repo. cosmetic issue — does not affect implementation.

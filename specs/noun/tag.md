# type tags


## canonical value tower (nox<Goldilocks, Z/2^32, Hemera>)

```
┌───────────────────────────────────────────────────────────────────────┐
│  TYPE TAG    │  REPRESENTATION     │  VALID RANGE    │  USE          │
├──────────────┼─────────────────────┼─────────────────┼───────────────┤
│  0x00: field │  Single F_p element │  [0, p)         │  Arithmetic   │
│  0x01: word  │  Single F_p element │  [0, 2^32)      │  Bitwise      │
│  0x02: hash  │  4 × F_p elements   │  32-byte digest │  Identity     │
└───────────────────────────────────────────────────────────────────────┘
```

field and word share the same representation (one Goldilocks element) but different operations. a field element wraps around modulo p; a word wraps around modulo 2^32. the distinction is semantic, enforced by the type system. the 32-bit word range guarantees every word value is a valid field element ([0, 2^32) ⊂ [0, p)), and every bitwise operation produces a representable result. heavy 64-bit binary computation belongs in Bt (FRI-Binius, characteristic 2), not in nox's prime field bitwise patterns.

the hash type (four field elements, 32 bytes) is the identity primitive. `H(noun)` produces a hash. `axis(s, 0)` returns `H(s)` — a noun can introspect its own identity.

the type tag costs nothing in the stark — it is a constraint selector, not runtime data.

## value tower across instantiations

the three-type tower (field, word, hash) is specific to the Goldilocks instantiation. in other instantiations the tower adapts:

```
nox<F₂, Z/2^1, Grøstl>:     atom = 1 bit, word = 1 bit (field = word in char 2)
nox<F_{p³}, Z/2^32, Hemera>: atom = 3 × F_p, word = [0, 2^32), hash = 4 × F_p
```

whether the three-type value tower generalizes cleanly across all fields is an open question. what is invariant: the distinction between field operations (patterns 5-10), bitwise operations (patterns 11-14), and hash (pattern 15) — these map to algebraically distinct domains in every instantiation.

## coercion rules

### canonical (Goldilocks)

```
field → word:  valid when value < 2^32 (range check)
word → field:  always valid (injection, [0, 2^32) ⊂ [0, p))
hash → field:  extract first element (lossy, for compatibility only)
field → hash:  forbidden (use HASH pattern 15)
```

## type errors

```
bitwise op on hash → ⊥_error
arithmetic on hash (except equality) → ⊥_error
```

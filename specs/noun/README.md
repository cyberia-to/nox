# noun specification

version: 0.2
status: canonical

## overview

everything in nox is a noun. a noun is either an atom or a cell. there is nothing else.

```
noun = atom(F)
     | cell(noun, noun)
```

atom: single element of the instantiated field F.
cell: ordered pair of two nouns — a binary tree.

a program is a noun. an object is a noun. the result is a noun. a cyberlink is a noun. a stark proof serialized for verification is a noun. one structure for everything.

the noun model is parameterized by the field F. in the canonical instantiation (nox<Goldilocks, Z/2^32, Hemera>), F = F_p where p = 2^64 - 2^32 + 1. see vm.md for the instantiation model.

## formulas

a formula is a noun of the form `cell(tag, body)` where tag is an atom encoding the pattern number (0-16).

```
cell(0, a)              axis
cell(1, c)              quote
cell(2, cell(x, y))    compose
cell(3, cell(a, b))    cons
cell(4, cell(t, cell(y, n)))  branch
cell(5, cell(a, b))    add
cell(6, cell(a, b))    sub
cell(7, cell(a, b))    mul
cell(8, a)              inv
cell(9, cell(a, b))    eq
cell(10, cell(a, b))   lt
cell(11, cell(a, b))   xor
cell(12, cell(a, b))   and
cell(13, a)             not
cell(14, cell(a, n))   shl
cell(15, a)             hash
cell(16, constraint)    hint
```

the distinction between code and data is purely contextual — the same noun can be an object in one reduction and a formula in another. this homoiconicity extends to the proof system: the stark proves that a specific noun (the formula) was applied to a specific noun (the object). the proof refers to the same binary tree structure that the execution operated on.

## axis addressing

pattern 0 navigates the noun binary tree using a numeric address.

```
axis(s, 0)   = H(s)                    hash introspection
axis(s, 1)   = s                        identity (root)
axis(s, 2)   = head(s)                  left child
axis(s, 3)   = tail(s)                  right child
axis(s, 2n)  = head(axis(s, n))         left of subtree
axis(s, 2n+1)= tail(axis(s, n))         right of subtree
```

axis on an atom (except 0 and 1) produces ⊥_error.

```
       1
      / \
     2   3
    / \ / \
   4  5 6  7
```

## module index

| page | scope | rs module |
|------|-------|-----------|
| tag.md | type tags, value tower, coercion | noun/tag.rs |
| inner.md | atom, cell, polynomial representation | noun/inner.rs |
| arena.md | memory, bounds, structural sharing, lifecycle | noun/arena.rs |
| hash.md | structural hash, identity | noun/hash.rs |

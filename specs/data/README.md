# data specification

version: 0.2
status: canonical

## overview

everything in nox is data. data is either an atom or a pair. there is nothing else.

```
data = atom(field)
     | pair(data, data)
```

atom: a leaf — one element of the instantiated field F.
pair: two children joined, each an atom or a pair — a binary tree.

a program is data. an object is data. the result is data. a cyberlink is data. a zheng proof serialized for verification is data. one structure for everything.

the data model is parameterized by the field F. in the canonical instantiation (nox<Goldilocks, Z/2^32, Hemera>), F = F_p where p = 2^64 - 2^32 + 1. see vm.md for the instantiation model.

## formulas

a formula is data of the form `pair(tag, body)` where tag is an atom encoding the pattern number (0-17).

```
pair(0, a)              axis
pair(1, c)              quote
pair(2, pair(x, y))    compose
pair(3, pair(a, b))    cons
pair(4, pair(t, pair(y, n)))  branch
pair(5, pair(a, b))    add
pair(6, pair(a, b))    sub
pair(7, pair(a, b))    mul
pair(8, a)              inv
pair(9, pair(a, b))    eq
pair(10, pair(a, b))   lt
pair(11, pair(a, b))   xor
pair(12, pair(a, b))   and
pair(13, a)             not
pair(14, pair(a, n))   shl
pair(15, a)             hash
pair(16, constraint)    call
pair(17, pair(ns, key))   look
```

the distinction between code and data is purely contextual — the same data can be an object in one reduction and a formula in another. this homoiconicity extends to the proof system: the zheng proves that specific data (the formula) was applied to specific data (the object). the proof refers to the same binary tree structure that the execution operated on.

## axis addressing

pattern 0 navigates the data binary tree using a numeric address.

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

| page | scope |
|------|-------|
| inner.md | atom, pair, polynomial representation |
| reduction.md | reduction context, memory, bounds, structural sharing, lifecycle |
| hash.md | structural hash, identity |

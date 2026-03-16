# nouns

the one data structure — binary trees of [[Goldilocks field]] elements. everything in nox is a noun. there is nothing else.

## the decision

every computation system chooses a data model. most choose many types: integers, floats, strings, arrays, maps, objects, closures. each type has its own encoding, its own operations, its own edge cases. the complexity compounds — serialization must handle each type, the proof system must constrain each type, the content-addresser must hash each type.

nox chooses one. a noun is either an atom (a single field element) or a cell (an ordered pair of two nouns). a binary tree where every leaf is a [[Goldilocks field]] element. this is the entire data model.

```
noun = atom                     a single field element
     | cell(noun, noun)         an ordered pair

examples:
  42                            an atom
  (7 . 13)                      a cell of two atoms
  ((1 . 2) . (3 . 4))          a balanced tree
  ((0 . 2) . ((1 . 0) . (5 . ((0 . 2) . (0 . 3)))))    a program
```

## why one structure is enough

a list is a right-nested cell chain: `(a . (b . (c . 0)))`. a record is a tree with named positions (axis 2 = first field, axis 6 = second field). a string is a list of character codes. a program is a cell where the head is a pattern tag and the tail is the operands. a [[stark]] proof is a tree of field elements and Merkle paths. a [[cyberlink]] is a cell of two [[particles]].

one structure means one serialization format, one hash function, one content-addressing scheme, one proof encoding. the simplicity propagates through every layer of the system. there is no type dispatch at the serialization layer, no case analysis in the hasher, no format negotiation in the network protocol. a noun is a noun — serialize it, hash it, transmit it, prove it.

## three types, one representation

atoms carry a type tag, but the tag is metadata — it does not change the representation. a field element, a word, and a hash element all live in the same Goldilocks field. the tag tells the VM which operations are legal.

```
field (0x00)    arithmetic: a + b, a × b, a⁻¹         range [0, p)
word  (0x01)    bitwise: a XOR b, a AND b, a << n      range [0, 2⁶⁴)
hash  (0x02)    identity: 8 field elements = 64 bytes   Hemera output
```

field and word share the same representation but different algebras. a field element wraps modulo p (the Goldilocks prime). a word wraps modulo 2^64 (machine integers). the distinction exists because the [[stark]] constraint system needs to know which algebra applies — addition modulo p uses one constraint, XOR uses ~64 constraints (bit decomposition). the type tag is a constraint selector, not runtime overhead.

the hash type uses eight field elements (8 × 8 = 64 bytes). it is the identity primitive — every noun can be reduced to a hash, and the hash is how the network refers to the noun. `axis(s, 0)` returns `H(s)` — a noun can introspect its own cryptographic identity. this is unique to nox: self-referential identity is a first-class operation, not a library call.

## trees as memory

in conventional architectures, memory is a flat array of bytes. addresses are integers. access is O(1). the model is simple but carries hidden complexity: pointers can alias, mutation requires synchronization, garbage collection is a global concern.

in nox, memory is a binary tree. addresses are axis paths — binary numbers that trace a route from root to leaf. access is O(depth). the model has different tradeoffs: no aliasing (trees are persistent), no mutation (new trees share structure with old trees), no garbage collection (reference counting on tree nodes, or structural sharing with copy-on-write).

the O(depth) access cost is real. but depth grows logarithmically with the number of leaves — a tree with a million leaves has depth ~20. and the cost is explicit in the focus budget: axis costs 1 + depth. the programmer and the [[stark]] prover both see the same cost model. there are no hidden memory operations behind an O(1) abstraction.

## homoiconicity

a nox formula is a cell `(tag . body)` where tag is the pattern number (0-16) and body contains the operands. a formula is a noun. an object is a noun. the result is a noun. the distinction between code and data is purely contextual — the same noun can be an object in one reduction and a formula in another.

this goes deeper than Lisp's homoiconicity. in Lisp, code is data within the runtime. in nox, code is data at the level of the proof system. the [[stark]] proves that a specific noun (the formula) was applied to a specific noun (the object) to produce a specific noun (the result). the proof refers to the same binary tree structure that the execution operated on. there is no separate representation for "the circuit" vs "the program" — they are the same noun.

the consequence for metaprogramming: a nox program can construct other nox programs (they are just nouns), inspect their structure (axis addressing), and compose them (pattern 2). a compiler is a nox program that takes source code (a noun) and produces target code (a noun). a proof verifier is a nox program that takes a proof (a noun) and validates it. compilation and verification are computations over nouns — they get the same content-addressing, the same memoization, the same provability as any other computation.

## content-addressed identity

because nouns have a canonical encoding and a deterministic hash, every noun has a unique cryptographic identity:

```
H(atom a)     = hemera_leaf(encode(a), capacity[14] = type_tag(a))
H(cell(l, r)) = hemera_node(H(l), H(r))
```

the type tag is embedded in [[Hemera]]'s sponge capacity — the same domain separation mechanism Hemera uses for leaf/node/root distinction in Merkle trees. the hash output is 64 bytes, no prefix bytes, no framing. the type is inside the permutation, not outside it. different atom types produce different hashes for the same value — domain separation is enforced by the mathematics, not by encoding conventions.

two nouns are the same if and only if they have the same hash. this is the foundation of everything content-addressed in [[cyber]]: [[particles]] are hashed nouns, [[cyberlinks]] connect hashed nouns, the computation cache keys on hashed nouns. the one data structure with the one hash function creates the one identity system.

the hash is compositional: `H(cell(l, r))` depends only on `H(l)` and `H(r)`, not on the full structure of the children. this enables incremental hashing — when a tree is modified at one leaf, only the path from that leaf to the root needs rehashing. the rest of the tree's hashes are unchanged. this is the Merkle tree property, and it falls out naturally from the noun definition.

## what nouns cannot do

nouns are not efficient for everything. flat arrays of bytes, dense matrices, hash maps with O(1) lookup — these do not map naturally to binary trees. a 1MB image stored as a noun is a deeply nested tree of field elements, larger and slower to access than the raw bytes.

this is by design. nox is a verification machine, and the things it verifies — identity, ownership, conservation laws, graph structure, proof validity — are naturally tree-shaped. bulk data lives in [[particles]] (content-addressed blobs); nox operates on their hashes. the VM handles the cryptographic and algebraic layer; data storage is a separate concern, handled by [[bbg]].

the constraint is clarifying: if something does not naturally decompose into a binary tree of field elements, it probably should not be inside a nox computation. the VM's simplicity is its boundary — it does exactly what it needs to do for provable computation, and nothing more.

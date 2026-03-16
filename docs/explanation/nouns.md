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

## what is radical

every other content-addressed system has a serialization layer. Git has object headers (`blob 42\0`). IPFS has CBOR-encoded DAG nodes with link tables. Ethereum has RLP encoding. Protocol Buffers have field tags and wire types. every one of them pays framing overhead to make the byte stream self-describing.

nox has no serialization format. the store maps 64-byte identity to content, and content length IS the type. this is not an optimization — it is a category elimination. the serialization layer does not exist.

the hash function IS the type system at the protocol level. a field atom with value 7 and a word atom with value 7 have different identities — not because of a tag byte, but because [[Hemera]] capacity carries different values during permutation. the type distinction is enforced by the mathematics of the sponge, not by a convention on top of it. you cannot forge a field-typed identity from a word-typed value because the Poseidon2 permutation is one-way.

fixed-size everything. three content sizes: 8, 64, 128. the store is a fixed-size key-value map. no variable-length records. no allocation decisions. no fragmentation. uniform record sizes with content-addressed keys.

cells store hashes, not data. a cell is always 128 bytes: two child identities. the tree is navigated by hash lookup, not by pointer chasing through variable-length buffers. structural sharing is automatic — two cells with the same left subtree store the same left hash, and the store deduplicates by identity. this is Merkle tree semantics applied to the entire data model, not just to a specific data structure.

one hash function for everything. [[Hemera]] does structural hashing, Merkle trees, Fiat-Shamir challenges, content addressing, domain separation, commitment schemes, nullifier derivation. one function, one output size (64 bytes), one security assumption. the entire cryptographic surface area is one Poseidon2 instance.

## honest tradeoffs

storage amplification for small nouns. a field atom is 8 bytes of data but has a 64-byte identity. a cell of two small atoms: 16 bytes of actual data, but the cell stores 128 bytes of child hashes, plus each atom has a 64-byte identity. the store entry for the cell is 128 bytes, pointing to two 8-byte entries. total: 128 + 8 + 8 = 144 bytes for 16 bytes of data. 9x overhead.

for deep trees this amortizes (hashes are shared, deduplication kicks in). but for flat formulas with many small atoms, storage is heavier than a serialized format would be.

resolution latency. materializing a noun of depth d requires d sequential store lookups. a flat serialization reads one contiguous buffer. for proof verification (the hot path), the verifier processes nouns that might be 20-30 levels deep — that is 20-30 lookups. with an SSD that is microseconds; in memory it is nanoseconds. acceptable, but not free.

no streaming decode. with a serialized format, you can read bytes and build the noun in one pass. with content-addressed resolution, you must fetch the root, then its children, then their children — breadth-first or depth-first, but always recursive. you cannot pipe a noun through a socket and process it incrementally.

the atom identity paradox. an atom identity (64 bytes) is larger than its content (8 bytes). you carry more metadata than data for leaves. in systems with many small atoms (which formulas are — pattern tags are atoms 0-16), the identity overhead dominates.

## why the tradeoffs are acceptable

every tradeoff above trades throughput for verifiability. in a proof-native system, this is the right trade:

- storage amplification does not matter when the [[stark]] proof compresses everything to 60-157 KiB regardless of computation size
- resolution latency does not matter when the hot path is the prover (which processes the noun in memory anyway) and the verifier (which only checks the proof, not the noun)
- no streaming is fine because nouns enter the system through reduction, not through deserialization — the VM builds nouns, it does not parse them
- the atom identity paradox is actually a feature: small values get strong identities, making the content-addressed cache effective even for trivial sub-expressions

the system is designed for one thing: produce a computation, prove it, verify the proof. every design choice optimizes for that path. the 64-byte identity is the unit of trust — and having it be clean, uniform, and prefix-free means the trust layer has zero accidental complexity.

## what nouns cannot do

nouns are not efficient for everything. flat arrays of bytes, dense matrices, hash maps with O(1) lookup — these do not map naturally to binary trees. a 1MB image stored as a noun is a deeply nested tree of field elements, larger and slower to access than the raw bytes.

this is by design. nox is a verification machine, and the things it verifies — identity, ownership, conservation laws, graph structure, proof validity — are naturally tree-shaped. bulk data lives in [[particles]] (content-addressed blobs); nox operates on their hashes. the VM handles the cryptographic and algebraic layer; data storage is a separate concern, handled by [[bbg]].

the constraint is clarifying: if something does not naturally decompose into a binary tree of field elements, it probably should not be inside a nox computation. the VM's simplicity is its boundary — it does exactly what it needs to do for provable computation, and nothing more.

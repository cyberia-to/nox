# bitwise patterns

four patterns for native operations over 32-bit words — lightweight binary logic inside nox's prime field.

```
11 xor — exclusive or
12 and — bitwise and
13 not — bitwise complement
14 shl — left shift
```

these exist because F_p and Z/2^32 are fundamentally different algebras that cannot simulate each other cheaply. nox programs need lightweight binary logic — flag testing, byte extraction, bit masking — without decomposing into pure field arithmetic.

## the two-algebra problem

nox lives in F_p — a prime field where arithmetic wraps at p = 2^64 - 2^32 + 1. binary logic lives in Z/2^32 — 32-bit words where operations are bitwise. XOR has no natural expression as field arithmetic — it operates on individual bits, and bits are not a native concept in a prime field. AND has no polynomial representation over a prime field. the algebras are algebraically incompatible.

without bitwise patterns, every interaction with binary data would require bit decomposition: split a word into 32 individual bits (each 0 or 1), perform Boolean logic on the bits, then reassemble the result. this decomposition costs ~32 [[stark]] constraints per operation — the algebraic cost of proving that 32 individual values are each 0 or 1 and that they reconstruct the original number.

the bitwise patterns absorb this cost into their constraint layout. each pattern includes the bit decomposition in its [[stark]] constraints (~32 constraints per operation) and exposes clean O(1) execution cost to the programmer. the programmer writes `xor(a, b)`. the proof system handles the decomposition internally. the cost model is honest: bitwise is ~32× more expensive than field arithmetic in proof size, because that is the real algebraic distance between F_p and Z/2^32.

## why 32-bit, not 64-bit

nox is a field machine. the Goldilocks field has p = 2^64 - 2^32 + 1. a 32-bit word always fits in a field element ([0, 2^32) ⊂ [0, p)). every bitwise operation on 32-bit words produces a representable result — no overflow, no gap, no reduction needed. `not(not(x)) = x` and `xor(a, xor(a, b)) = b` hold unconditionally.

64-bit words would create 2^32 - 1 unrepresentable values in the gap [p, 2^64). `not(0) = 2^64 - 1 > p` — broken. no clean fix exists without either distorting bitwise semantics or doubling the register cost.

heavy 64-bit binary computation — SHA-256 circuits, Keccak verification, AES, binary protocol parsing — belongs in [[Bt]] (FRI-Binius, characteristic 2), where AND and XOR are free field operations. nox's bitwise patterns serve branching logic, flag testing, and lightweight binary manipulation. [[Rs]] programs needing u64 represent them as pairs of u32 words — a compiler concern, not a VM concern.

## xor: the cryptographic primitive

exclusive or flips bits where the other operand has 1s, leaves bits where it has 0s. for each bit position independently: `0⊕0=0, 0⊕1=1, 1⊕0=1, 1⊕1=0`.

```
xor(a, b):  for each bit i: result_i = a_i ⊕ b_i
```

XOR is the fundamental operation of [[cryptography]]. stream ciphers encrypt by XORing plaintext with a key stream. hash functions mix state using XOR. error-correcting codes compute parity with XOR. Galois field arithmetic (F_2 polynomial multiplication, used in AES and Reed-Solomon) is built on XOR.

XOR is also its own inverse: `xor(xor(a, b), b) = a`. this self-inverse property makes it the natural operation for toggling, masking, and reversible transformation. in the context of nox, it means XOR-based operations can be verified by re-applying the same operation — the verifier checks `xor(result, b) == a`.

## and: the masking primitive

bitwise AND produces 1 only where both operands have 1s. for each bit position: `0∧0=0, 0∧1=0, 1∧0=0, 1∧1=1`.

```
and(a, b):  for each bit i: result_i = a_i ∧ b_i
```

AND is the fundamental operation of extraction. to read a specific bit: `and(a, shl(1, n))`. to isolate a byte: `and(a, 0xFF)`. to clear high bits: `and(a, mask)`. every time a program needs to look at part of a word — a flag in a header, a field in a packet, a digit in an encoding — it uses AND with a mask.

AND is also the mechanism for set intersection when bits represent set membership. a 32-bit word can represent a set of up to 32 elements, and `and(set_a, set_b)` computes the intersection in a single operation. this matters for flag testing, permission checking, and feature negotiation — all common in protocol handling.

## not: the complement

bitwise NOT flips every bit: 0 becomes 1, 1 becomes 0.

```
not(a):  for each bit i: result_i = ¬a_i
         equivalently: not(a) = (2^32 - 1) - a    in Z/2^32
```

NOT completes the Boolean algebra. with XOR and AND alone, you cannot express all Boolean functions — you need complement to break the symmetry. the three operations together ({xor, and, not}) are functionally complete: any Boolean function of any number of inputs can be expressed as a composition.

- OR: `or(a, b) = xor(and(a, b), xor(a, b))`
- NAND: `nand(a, b) = not(and(a, b))`
- NOR: `nor(a, b) = not(or(a, b))`
- MUX: `mux(sel, a, b) = xor(and(sel, xor(a, b)), b)`
- implication, equivalence, majority — all expressible

NAND alone is functionally complete (every Boolean function can be built from NAND gates — this is how all digital hardware works). but xor+and+not is the practical choice for a VM: the three most common binary operations (cryptographic mixing, bit extraction, complement) are each a single pattern instead of multi-gate compositions.

## shl: positional manipulation

shift left moves bits toward higher positions, filling vacated positions with zeros. `shl(a, n)` multiplies a by 2^n in the binary interpretation.

```
shl(a, n):  result = (a × 2^n) mod 2^32
            equivalently: slide all bits left by n positions, fill right with zeros
```

this single operation covers a surprising range of bit manipulation:

- left shift: `shl(a, n)` directly
- right shift: `and(shl(a, 32-n), mask)` — shift left by the complement, mask off the overflow
- bit extraction: `and(shl(a, 32-k), mask)` — shift the target bit to a known position, mask it
- rotation: `xor(shl(a, n), shl(a, 32-n))` with appropriate masking
- byte swapping: composition of shifts, masks, and ORs
- bit reversal: sequence of shift-and-mask operations
- field packing: shift values to their positions, OR them together

a dedicated right shift pattern would save one operation per right shift. but it would consume a precious 4-bit encoding slot for something expressible as a two-pattern composition. the design chooses encoding density over convenience — consistent with the principle that patterns exist for algebraic necessity, not programmer comfort.

## the cost model

```
pattern    execution cost    STARK constraints    notes
xor        O(1)              ~32                  bit decomposition of both operands
and        O(1)              ~32                  bit decomposition of both operands
not        O(1)              ~32                  bit decomposition of operand
shl        O(1)              ~32                  bit decomposition + shift verification
```

every bitwise operation costs ~32 [[stark]] constraints — compared to 1 constraint for field arithmetic. this 32× ratio is the honest algebraic distance between F_p and Z/2^32. proving a bit operation in a prime field means proving that the operands decompose into valid bits (each is 0 or 1) and that the operation on those bits produces the claimed result. there are 32 bits per operand, hence ~32 constraints.

this cost is absorbed once in the pattern's constraint layout. the programmer sees O(1) execution cost. the prover pays the constraint cost. the verifier checks the constraints in O(log n) time regardless. the cost is predictable: count the bitwise operations, multiply by ~32, and you know the proof size contribution.

## what bitwise enables

the binary world is vast:

- network protocols encode headers as bit fields — TCP flags, IP options, TLS extensions
- file formats pack data into byte sequences — PNG chunks, protobuf varints, CBOR tags
- cryptographic primitives operate on bits — [[Hemera]] internally uses field arithmetic, but any future hash or cipher that nox interacts with externally speaks binary
- error-correcting codes manipulate polynomial coefficients over F_2
- compression algorithms work with variable-length bit strings — Huffman codes, LZ77 offsets

without the bitwise group, nox could still compute all these functions (Turing completeness guarantees it), but the cost would be prohibitive. parsing a single network packet header — extracting flags, lengths, checksums from bit positions — would require hundreds of field operations simulating bit extraction. with bitwise patterns, it requires a handful of AND and SHL operations.

the bitwise group handles lightweight binary logic inside nox's prime field — flag testing, byte extraction, protocol header parsing. it costs more in proof size than field arithmetic (~32× for the algebraic distance) but avoids the overhead of spinning up [[Bt]] for simple binary operations. heavy binary computation — full cryptographic circuits, large-scale protocol parsing — belongs in Bt where AND and XOR are free.

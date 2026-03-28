# pattern specification

version: 0.2
status: canonical

## overview

eighteen patterns: sixteen deterministic compute (Layer 1), one non-deterministic witness injection (Layer 2), one deterministic BBG read. 16 compute + call + look = 18.

the 16 deterministic compute patterns are algebra-polymorphic — parameterized by field F, word width W, and hash function H. see vm.md for the instantiation model.

```
╔═══════════════════════════════════════════════════════════════════════════╗
║                       LAYER 1: REDUCTION PATTERNS                         ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  STRUCTURAL (5)              FIELD ARITHMETIC (6)                         ║
║  0: axis — navigate          5: add — F-addition                          ║
║  1: quote — literal          6: sub — F-subtraction                       ║
║  2: compose — recursion      7: mul — F-multiplication                    ║
║  3: cons — build cell        8: inv — F-inverse                           ║
║  4: branch — conditional     9: eq  — equality test                       ║
║                              10: lt — less-than                           ║
║                                                                           ║
║  BITWISE (4)                 HASH (1)                                     ║
║  11: xor    12: and          15: hash — structural H(x)                   ║
║  13: not    14: shl                                                       ║
╠═══════════════════════════════════════════════════════════════════════════╣
║  LAYER 2                                                                  ║
║  16: call — non-deterministic witness injection (prover → formula)        ║
║  17: look — deterministic read from BBG                                   ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

all concrete costs and constraint counts refer to the canonical instantiation: nox<Goldilocks, Z/2^32, Hemera>.

## module index

| spec | tag | pattern |
|------|-----|---------|
| 00-axis.md | 0 | axis |
| 01-quote.md | 1 | quote |
| 02-compose.md | 2 | compose |
| 03-cons.md | 3 | cons |
| 04-branch.md | 4 | branch |
| 05-add.md | 5 | add |
| 06-sub.md | 6 | sub |
| 07-mul.md | 7 | mul |
| 08-inv.md | 8 | inv |
| 09-eq.md | 9 | eq |
| 10-lt.md | 10 | lt |
| 11-xor.md | 11 | xor |
| 12-and.md | 12 | and |
| 13-not.md | 13 | not |
| 14-shl.md | 14 | shl |
| 15-hash.md | 15 | hash |
| 16-call.md | 16 | call |
| 17-look.md | 17 | look |

## cost table (canonical: nox<Goldilocks, Z/2^32, Hemera>)

```
Layer │ Pattern      │ Exec Cost      │ STARK Constraints │ Rationale
──────┼──────────────┼────────────────┼───────────────────┼─────────────────────
  1   │ 0 axis       │ 1              │ 1                 │ O(1) Lens opening
  1   │ 1 quote      │ 1              │ 1                 │ literal return
  1   │ 2 compose    │ 1              │ 1                 │ dispatch only
  1   │ 3 cons       │ 1              │ 1                 │ cell construction
  1   │ 4 branch     │ 1              │ 1                 │ test + select
  1   │ 5 add        │ 1              │ 1                 │ F-addition
  1   │ 6 sub        │ 1              │ 1                 │ F-subtraction
  1   │ 7 mul        │ 1              │ 1                 │ F-multiplication
  1   │ 8 inv        │ 64             │ 1                 │ F-inverse (Goldilocks)
  1   │ 9 eq         │ 1              │ 1                 │ equality comparison
  1   │ 10 lt        │ 1              │ ~64               │ range decomposition
  1   │ 11-14 bit    │ 1              │ ~32 each          │ bit decomposition
  1   │ 15 hash      │ 200            │ ~736              │ Hemera permutation
  2   │ 16 call      │ 1              │ 1                 │ inject + dispatch
  2   │ 17 look      │ 1              │ 1                 │ BBG read + verify
```

## test vectors (canonical: nox<Goldilocks>)

```
add(1, 2) = 3
mul(p-1, p-1) = 1
inv(2) = 9223372034707292161
inv(0) = ⊥_error

reduce([1,2], [5 [[0 2] [0 3]]], 100) = (3, 97)
reduce(42, [1 7], 10) = (7, 9)
reduce([1,2], [3 [[0 2] [0 3]]], 100) = (cell(1, 2), 97)
reduce([1,2], [4 [[9 [[0 2] [0 3]]] [[1 100] [1 200]]]], 100) = (200, 95)
```

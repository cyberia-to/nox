---
title: field members — yin and yang
tags: cyber, nox, soft3, roadmap
crystal-type: spec
crystal-domain: cyber
status: proposal
alias: yin yang, field halves, word rename proposal
---

# field members — yin and yang

## proposal

name the two 32-bit halves of a [[Goldilocks field]] element `yin` (low) and `yang` (high), and retire `Word` (= a field proven in `[0,2³²)`) — which frees `word` for [[neural]]'s lexical unit.

```
field = yang·2³² + yin
  yin  ∈ [0,2³²)   the low half     (was `Word` / surface u32)
  yang ∈ [0,2³²)   the high half
```

## why two members, and why they interpenetrate

a [[Goldilocks field]] element is 64-bit (`p = 2⁶⁴ − 2³² + 1`). it splits at 32 bits into two equal-width members — but they are not symmetric, and that asymmetry is the whole point of Goldilocks. reducing higher powers:

```
2⁶⁴ ≡ 2³² − 1   (mod p)
2⁹⁶ ≡ −1
```

so a 128-bit product (two field elements multiplied) reduces with no division, by folding the high members down:

```
x = x₃·2⁹⁶ + x₂·2⁶⁴ + x₁·2³² + x₀
  ≡ (x₀ − x₂ − x₃) + (x₁ + x₂)·2³²
```

`yin` (low) stays; `yang` (high) folds back into the field, carrying a ∓1. low is the residue, high is what wraps — they pour into each other. exactly yin and yang.

## why the name is structure, not decoration

`p = Φ₆(2³²) = (2³²)² − 2³² + 1` — the sixth cyclotomic polynomial at 2³². `2³²` is a primitive sixth root of unity (`g³ ≡ −1`, `g⁶ ≡ 1`). the `+1` — the cyclotomic constant — does three jobs at once:

1. primality — `2⁶⁴ − 2³²` is even; `+1` makes `p` prime (a field needs a prime modulus)
2. NTT — `p − 1 = 2³²(2³² − 1)` is divisible by 2³², so there is a 2³²-th root of unity → fast transforms for STARKs
3. the fold — `2⁶⁴ ≡ 2³² − 1`; the `−1` you see in reduction is that `+1` returning

that lone unit is the seed each half carries of the other — yin's dot of yang. the pair is the structure of the field, not a metaphor pasted on.

## the rename (delta for the [[rename contract|terms-map]])

| old | new | notes |
|---|---|---|
| `Word` (field in `[0,2³²)`) | `yin` | the low 32-bit member; a u32 surface value is a lone yin (yang = 0) |
| `U32` (trident surface) | `yin` | supersedes the planned `U32 → Word` |
| — | `yang` | new: the high 32-bit member; appears in field decomposition and reduction |

`word` (the noun) is hereby free. it goes to [[neural]] as the lexical unit — a typed [[particle]]. see [[neural]].

## scope

surface and naming only: the 32-bit value type, plus the names for field decomposition. it does NOT touch `field`, `reduction`, `order`, or `particle`. on acceptance, update the [[rename contract|terms-map]] rows and propagate through nox / trident code and docs in the next sweep.

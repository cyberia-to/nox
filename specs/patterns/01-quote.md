# pattern 1: quote

rs module: patterns/quote.rs

algebra-independent.

```
reduce(s, [1 c], f) = (c, f - 1)
```

returns c literally, unevaluated. the only pattern that ignores the object.

cost: 1. constraints: 1.

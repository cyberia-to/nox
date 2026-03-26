# pattern 4: branch


algebra-independent.

```
reduce(o, [4 [test [yes no]]], f) =
  let (t, f1) = reduce(o, test, f - 1)
  if t = 0 then reduce(o, yes, f1)
           else reduce(o, no, f1)
```

only ONE branch is evaluated. prevents infinite recursion DoS.

NOT parallel — must evaluate test before choosing a branch.

cost: 1. constraints: 1.

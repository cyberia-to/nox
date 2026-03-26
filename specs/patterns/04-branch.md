# pattern 4: branch


algebra-independent.

```
reduce(s, [4 [test [yes no]]], f) =
  let (t, f1) = reduce(s, test, f - 1)
  if t = 0 then reduce(s, yes, f1)
           else reduce(s, no, f1)
```

only ONE branch is evaluated. prevents infinite recursion DoS.

NOT parallel — must evaluate test before choosing a branch.

cost: 1. constraints: 1.

# hint pattern (16) — Layer 2


```
reduce(s, [16 [tag_f check_f]], f) =
  1. tag = reduce(s, tag_f, f - 1)           // evaluate tag expression
  2. witness = provider.provide(tag, s)       // ask prover
     if witness == Halt → return Halt
  3. result = reduce([witness s], check_f, f')  // validate
  4. return result
```

the single non-deterministic pattern. the prover injects a witness noun from outside the VM. the constraint formula is evaluated with s as object to produce check — a formula. then check is applied to witness as object via standard reduction. the result must be the field element 0 (success). if the check produces a non-zero value, halts, or errors, the hint fails and the proof is invalid.

the verifier NEVER executes hint directly — it checks constraint satisfaction via the stark proof.

## provider interface

```
trait HintProvider {
    fn provide(&self, tag: F, subject: NounRef) -> HintResult;
}

enum HintResult {
    Value(NounRef),
    Halt,
}
```

## tag conventions

```
0x00  unspecified (prover decides)
0x01  private key / secret witness
0x02  optimization solution
0x03  search result / oracle query
0x04  decryption share
```

tags are conventions, not enforced by the VM.

## check formula

the check formula validates the witness using Layer 1 patterns only. the witness enters as head of the subject: `[witness original_subject]`. the check can access both the witness (via axis 2) and the original subject (via axis 3).

## properties

- synchronous: hint is a function call, not an event
- no hint = halt: not an error. budget preserved for caller
- hint rejected = error: the witness failed validation
- not memoizable: different provers provide different valid witnesses
- confluence broken intentionally: multiple valid witnesses may satisfy the same check
- verifier never calls provide(): the zheng proof covers the check

## cost

hint dispatch: 1. tag evaluation: cost of tag_f. check evaluation: cost of check_f. total: 1 + cost(tag_f) + cost(check_f).

## what hint enables

```
identity:         hint injects the secret behind a neuron address
                  Layer 1 checks: H(secret) = address

private transfer: hint injects record details (owner, value, nonce)
                  Layer 1 checks: conservation, ownership, nullifier freshness

AI inference:     hint injects neural network weights
                  Layer 1 checks: forward pass produces claimed output

optimization:     hint injects an optimal solution
                  Layer 1 checks: solution satisfies constraints AND is optimal
```

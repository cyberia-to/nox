# call pattern (16) — Layer 2


```
reduce(o, [16 [tag_f check_f]], f) =
  1. tag = reduce(o, tag_f, f - 1)              // evaluate tag expression
  2. witness = provider.provide(tag, o)          // call prover
     if witness == None → return Halt
  3. check_result = reduce([witness o], check_f, f')  // validate
  4. if check_result == 0 → return witness
     else → return CallRejected
```

note: tags are not validated by the VM; any field value is accepted as a tag.

the single non-deterministic pattern. the prover injects a witness data from outside the VM. the check formula is evaluated with [witness, object] as the new object. the check result must be the field element 0 (success). if the check returns non-zero, halts, or errors, the call returns CallRejected. on success, the witness itself is returned (not the check result).

the verifier NEVER executes call directly — it checks constraint satisfaction via the zheng proof.

## provider interface

```
trait CallProvider {
    fn provide(&self, tag: F, object: particle) -> CallResult;
}

enum CallResult {
    Value(particle),
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

the check formula validates the witness using Layer 1 patterns only. the witness enters as head of the object: `[witness original_object]`. the check can access both the witness (via axis 2) and the original object (via axis 3).

## properties

- synchronous: call is a function call, not an event
- no call = halt: not an error. budget preserved for caller
- call rejected = error: the witness failed validation
- not memoizable: different provers provide different valid witnesses
- confluence broken intentionally: multiple valid witnesses may satisfy the same check
- verifier never calls provide(): the zheng proof covers the check

## cost

call dispatch: 1. tag evaluation: cost of tag_f. check evaluation: cost of check_f. total: 1 + cost(tag_f) + cost(check_f).

## what call enables

```
identity:         call injects the secret behind a neuron address
                  Layer 1 checks: H(secret) = address

private transfer: call injects record details (owner, value, nonce)
                  Layer 1 checks: conservation, ownership, nullifier freshness

AI inference:     call injects neural network weights
                  Layer 1 checks: forward pass produces claimed output

optimization:     call injects an optimal solution
                  Layer 1 checks: solution satisfies constraints AND is optimal
```

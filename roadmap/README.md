# nox roadmap

design proposals for [[nox]] VM evolution.

## status: in reference = proposal is now the canonical spec

| proposal | in reference? | target |
|----------|--------------|--------|
| [[algebra-polymorphism]] | **yes** → reference/vm.md | nox<F, W, H> parameterisation, 14 instantiations |
| [[recursive-jets]] | **yes** → reference/jets.md | 5 verifier jets (hash, poly_eval, merkle_verify, fri_fold, ntt), 8× reduction |
| [[binary-jets]] | **yes** → reference/jets.md | 8 Bt jets (popcount, binary_matvec, quantize, etc.), 1,400× for inference |
| [[implementation-audit]] | no (audit document, not a spec proposal) | Rs implementation readiness: 3 critical gaps identified |

## lifecycle

| status | meaning |
|--------|---------|
| **in reference** | merged into canonical spec — this is the architecture |
| draft | idea captured, open for discussion |

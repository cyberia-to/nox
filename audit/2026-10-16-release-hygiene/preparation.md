The first five check-runner invocations stopped before executing Cargo:
`nu::parser::incomplete_math_expression` at `check.nu:7:87`, caused by a multiline
predicate lacking outer parentheses. The runner syntax was corrected; no source
or gate result changed. Only subsequent source-bound receipts are gate outcomes.

The first package-identity extraction stopped before writing its report because
Nushell treats `Cargo.lock` as text without an explicit TOML decoder. Adding
`from toml` completed the extraction. Packaging and normalized-thumb gates had
already passed and were unaffected; no failed identity extraction is a gate.

# Portable verifier preparation

The evidence archives, CODE input and production source remained unchanged.

- `portable-v1` failed because an unquoted `get` interpreted the dot in
  `candidate-package.crate` as a cell path. `verify-first.nu.txt` is its exact
  body. No source or gate was rerun.
- `portable-v2` only added error detail; `verify-diagnostic.nu.txt` preserves it.
  Its raw stderr locates the same invalid lookup.
- `portable-v3` quoted the literal key, passed package/source checks, then reached
  an unquoted conditional string (`paired_repeat_required`) interpreted as an
  external command. `verify-third.nu.txt` preserves this body.
- `portable-v4` quoted the conditional strings and passed the full base/CODE,
  103-record, source, package, API and threshold reconstruction. Its output is an
  actual PASS, not a prediction.

The final verifier additionally binds the later actual CODE review, launcher and
statistic-field qualification as outer receipts. Its final replay has a separate
receipt. Historical Ruby capture helpers remain provenance, not runtime tools.

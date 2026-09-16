# nox jet input repair — 2026-09-12

## Implemented

The local acpu dependency selected by the owner manifest supplies the real NTT
and multilinear kernels. CPU and Honeycrisp now share one input preflight in
`rs/jets/admission.rs`. The raw u64 exponent is bounded before conversion or
shift; at most65536 expanded words/depth16 may enter a kernel. Balanced-depth
validation bounds expanded DAG traversal to131071 nodes without allocating.
The point decoder consumes exactly the required coordinate prefix and preserves
the pure formula's ignored-tail behavior. Kernel allocations occur only after
metadata, budget and shape checks, with fallible bounded reservation.

`JetRegistry::insert_exact_guarded` and runtime `lookup_exact_for` add explicit
admission while preserving `lookup_exact` introspection. A declined candidate
falls through to ordinary pattern dispatch with the untouched input and budget.
Replacing a registry identity deterministically replaces its prior guard, so an
unguarded duplicate cannot remain selected accidentally. Direct jet calls also
fail safely before dangerous shifts or allocations.

## Additional semantic findings repaired by admission

The poly-eval pure formula obtains recursion from the object's self-reference;
the old jet ignored it. Noncanonical self-references now decline acceleration.
The NTT anchor actually defines one omega-weighted butterfly, not a recursive
transform. Its previous full-NTT replacement changed both n>1 behavior and the
n=1 result for omega!=1. Runtime admission is restricted to the genuinely equal
n=0 and n=1/omega=1 cases; every other input runs its actual pure formula.
The direct bounded NTT kernel still computes its full transform. A complete
recursive NTT pure anchor is not implemented by this repair and must not be
claimed. These rules are specified in `specs/jets/input-admission.md`.

## Validation

Eight new regressions cover hostile exponents17/32/63/64/65/2^32/p-1, malformed
balanced-depth shape with equal leaf count, direct-call arena preservation,
k=0 pair-valued pure semantics, self-reference substitution, the NTT anchor
mismatch, compact depth16 shared DAG expansion, low budgets, ignored point tails,
CPU/Honeycrisp value parity and deterministic registry replacement.
Declined cases compare the complete output/error, trace and remaining budget
against an empty registry. Existing kernel/formula tests remain enabled.

All-feature suite:175 tests passed, zero failures/ignored tests or compiler
warnings (`/tmp/nox-jet-all-features-final.log`, local receipt). The default
feature suite is recorded separately in `/tmp/nox-jet-default-final.log`.
No GPU acceleration or complete36-jet genesis implementation is implied by
these results. Work is on `fix/jet-input-admission`, without publishing or
committing unrelated owner manifest/lock changes.

The default-feature suite passed169 tests with zero warnings. The optional acpu
dependency is now under the aarch64-macOS target table, preserving its existing
local path/version. Honeycrisp's accelerated branch has the same feature and
platform guard. A locked all-features x86_64 Linux cross-check passed with
Rust1.98 (`/tmp/nox-linux-allfeatures-check.log`); native all-features compilation
also passed. Linux execution has not been performed in this environment.

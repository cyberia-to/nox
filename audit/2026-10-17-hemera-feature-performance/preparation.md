# Preparation record

At base `64d091e5ad3f277cce77b88ae0de6eea4c49b4df`, the first invocation of
`nu audit/2026-10-17-hemera-feature-performance/plan-gate.nu .
/Users/master/.cache/cyber/nox/hemera-feature-performance/plan-thumb` exited 1
before Cargo execution: a multiline predicate without enclosing parentheses
parsed `and` as an external command. The exact initial runner is retained as
`plan-gate-initial.nu.txt`; the correction adds only those parentheses.
This is a capture-runner preparation failure, with no source compilation result.

The corrected runner then stopped before Cargo with `complete unchanged source
inventory`: it selected the hygiene intermediate `sources-final.json`, whose
`trace.rs` predicate predates the accepted `rfind` correction. The retained
`plan-gate-intermediate-inventory.nu.txt` identifies that invocation. The runner
now reads the actual accepted `sources-reviewed.json`, as the hygiene README
and root review require. No compiler input was changed to satisfy this bridge.

The actual first thumb invocation is retained in `plan-thumb.json` and raw
streams. At the same base it exited 101 with `error[E0463]: can't find crate for
core`. Its source inventory exactly matches the accepted hygiene compiler
inputs. Root authorized installing the exact target component. The captured
`rustup target add --toolchain 1.95.0 thumbv7em-none-eabi` command reports that
component already up to date; `installed-targets.json` also lists it before
installation. The initial inherited PATH placed Homebrew tools before rustup.
The first invocation did not capture the child compiler command, so its exact
compiler selection cannot be reconstructed from the version string alone.

`plan-gate-inherited-path.nu.txt` retains that runner. The current runner puts the
absolute rustup 1.95.0 toolchain directory first in an allowlisted PATH, clears
compiler overrides, and captures executable hashes. Its one distinct rerun,
`nu audit/2026-10-17-hemera-feature-performance/plan-gate.nu .
/Users/master/.cache/cyber/nox/hemera-feature-performance/plan-pinned-thumb
plan-pinned`, passes the named thumb gate with no warnings and identical source
inputs (`plan-pinned-thumb.json`). The original red remains evidence. Broad
hygiene checks were not rerun: their exact accepted compiler-input inventory is
bridged, and only the named thumb gate was compiled in this plan unit.

`original-replay.json` retains the original portable Nu verifier invocation and
successful result at the same base: 921 logical payloads, 103 execution records,
and the initial 5/9 and repeat 6/9 threshold crossings. Historical Ruby was never
executed. Root additionally requires absolute `RUSTC`/`RUSTDOC` pins for all
forthcoming cell builds and timing, beyond the clean PATH used by the plan gate.
No benchmark builds or timed invocations have occurred at this preparation point.

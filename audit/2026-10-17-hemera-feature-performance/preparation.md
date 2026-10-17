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

The subsequent source and registry exports completed unchanged. Initial build
runner parsing rejected a local parameter named `env` because Nushell reserves
that variable; `build-reserved-var.nu.txt` and `control-reserved-var.nu.txt` retain
that pre-execution runner. Renaming it to `run_env` allowed execution.

The A build completed successfully once, but postprocessing expected every
`-vv --message-format=json` stdout line to be JSON. Cargo also emitted ordinary
build-script lines, beginning with `[proc-macro2 1.0.107] cargo:rustc-check-cfg`.
`build.nu`, its raw A receipt, and original tool/script freeze remain unchanged.
`finish-builds.nu` resumes by reading the existing successful A receipt, parses
only JSON-object lines while retaining all raw output, and builds each remaining
cell once. This correction causes no repeated benchmark build or measured run.

The inspection runner initially failed Nushell parsing because a closure captured
a mutable local accumulator. `inspect-mutable-capture.nu.txt` preserves that
pre-execution source. Assigning the completed accumulator to an immutable binding
allowed inspection; neither a benchmark build nor a measured run was repeated.
Both Rust audit helpers compiled with `-Dwarnings`. A `measure` smoke invocation
wrapped only `/usr/bin/true`; its before/after telemetry and process receipt are
retained separately from the finite benchmark schedule. No benchmark binary was
executed by that smoke check.

The portable replay initially stopped while remapping source paths: Nushell
parsed `update cells` as its built-in cell-update subcommand, which requires a
closure. `portable-initial.nu.txt` retains that source. Using `upsert cells`
selects the intended record-field operation. The failed replay removed only its
own temporary extraction; retained capsules and all compiler inputs stayed intact.

The final source-bound portable/live replay passes in `preflight-final.json`.
The frozen set contains no timing evidence. The ordinary preparation commit then
runs the named thumb gate once more in `precommit-thumb.json`, passing without
warnings and with the complete accepted hygiene source-byte inventory unchanged.
Those precommit receipts follow the pre-timing freeze and are retained outside
its immutable inventory. Root independently replayed the portable verifier
without `--live`; timing authorization remains absent.

The full staged `git diff --cached --check` reports trailing whitespace in
archived `.patch` context lines. These literal spaces are required evidence
bytes and remain unchanged in the freeze. `raw-patch-whitespace.json` and its
compressed raw output retain the exit-2 diagnostic. Root reviewed that scope;
the separate authored Nu/Rust/Markdown/JSON check passes for all 83 staged files
in `authored-whitespace.json`. No whitespace policy or frozen receipt was changed.
An initial ephemeral diagnostic-capture invocation failed parsing the nonexistent
Nushell `path extension` command before execution; the successful invocation uses
`path parse | get extension`. It caused no file mutation or benchmark execution.

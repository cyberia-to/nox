# Joy integration with the observed evaluator implementation

Local validation uses nox `71b5860219f0e0810200946e4445e5fdf54269dd`, Joy
`ec83bd8d85b20a8bd20d2d14b0f25aab0f75e9fe`, and Trident
`c17bd0371c11746f46e20222c48cae2ab08be79d`. `pins.json.gz` records all nine
exact inputs. They were checked out into a separate sibling family; all nine
remained clean at their exact revisions after the checks (`post-inputs.json.gz`).
This is a local integration result, separate from the frozen SH6 CI matrix.

The exact commands, timestamps, exit statuses and raw stdout/stderr hashes are
in `commands.json.gz` and `fmt-workspace.json.gz`. Joy's soft3 boundary check,
all-target workspace check and release tests pass with zero Rust warnings.
The release suite reports 172 passed, zero failed and zero ignored across 27
test summaries. Workspace formatting (`cargo fmt -- --check`) passes. The
separate installation completed into `install-observer-integration`; the prior
runtime binary remained unchanged.

An initial broader `cargo fmt --all -- --check` attempted to enter nox's
optional Honeycrisp dependency workspace, absent from the nine-repository
closure, and failed during metadata loading. Its original stderr and empty
stdout are retained. The subsequent formatting check covers the Joy workspace;
no optional dependency or tracked input was added to make that check pass.

`c2-cli.json.gz` retains paired production CLI executions. Each uses the actual
C2 from Trident run `36359020560`, artifact `10948746458`, repetition 1:
SHA-256 `76a07c08265bd2ef525164472b6b53ac3f0e6cbbedce3250c4202f40ffba34c8`.
The complete compiler and producer are in Trident's retained platform store.
The same exact source and manifest are packed, compiled by C2 and executed.
Both cases explicitly use 196608 resident nodes and 10000000 collection-work
units, selecting the compacting evaluator's ordinary NoObserver path.
Other host settings use ordinary defaults and the retained manifest's limits.

JOB1, emitted ART1 and output bytes match exactly between old and new binaries.
Both execution reports also match after excluding only `elapsed_micros`.
The independently read output is canonical atom 13. The small case does not
claim a full self-build or forced collection; the owner tests cover relocation.
`baseline/` and `candidate/` retain every small input/output byte. Their common
compiler is referenced by the exact retained CI identity above.

The top-level pins in the paired receipt describe the new candidate only.
The baseline is the earlier local installed Joy binary, identified by SHA-256
and its original `baseline-install.json.gz`; that older installation does not
claim clean sibling provenance. The candidate comes from the separate clean
commit family recorded here. These checks validate downstream compatibility;
they add no proof dispatch or SH7 acceptance.

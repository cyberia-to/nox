# Restore an executable exact-tag API baseline

Stage: blocked; evidence: measured. Discovered-from: [Hemera no_std
repair](../hemera-no-std/research.md). Serves nox release/API conformance under
cyberia/dev § rings. Blocks: clean last-tag gate and any claim of clean release
validation; does not replace existing authorization for the bounded decision PR.

At owning source 0b1407a4b855e4961ede07e1b7ea199cb8018c30, the exact v0.2.0 tag
b927873bf6eb9296877b0fb3abb6607c353d31a6 has Cargo.lock entries for nox0.1.2,
Hemera0.3.0, Lens-core0.1.1 and older Strata while its manifest declares nox0.2.0.
Record64's real isolated `cargo rustdoc --locked --offline` fails first:
`error: cannot update the lock file /tmp/nox-no-std-inputs/nox-tag/Cargo.lock
because --locked was passed to prevent this`.

No historical tag or lock is modified. A generated standalone harness can provide
a separately labeled API diagnostic, but cannot turn this exact locked gate green.
A follow-up must determine the canonical historical source/dependency delivery
contract and review an explicit baseline policy before changing the required gate.
No policy change or release action is authorized by this task note.

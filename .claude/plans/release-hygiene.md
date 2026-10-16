# Nox release hygiene

Root authorized on 2026-10-16 after read-only Kadek prerequisite audit. Agent:
/root/phase2_resume_audit; shared parent /root. Base/pushed master:
7fe9ad368dc93ef136c196c2366b39f1c67e1cd7. Reuse the clean idle checkout
nox/.worktrees/no-std-dependencies/nox on new branch fix/release-hygiene.
Prior source-decision task and audit remain intact. Before reuse, tracked,
untracked and ignored status were empty; lsof showed only the inspection itself.
The wrapper's historical context is retained alongside the new context.

Implement only the mechanical cargo-fmt delta (44 files measured in canonical
Nox), CLI parse_data's redundant &formula_text, and strict-rustdoc markup in
call, patterns/{eq,look}, trace, parallel, jets/{registry,merkle_verify,backends/cpu}.
Do not change runtime algorithms, dependency manifests/versions/lock, optional
backend behavior, expected test values/literals, API-baseline policy or benchmark
harness logic. Test syntax changes are limited to the explicit extension below.
Existing oversized modules remain a separately reported quality obligation;
this bounded unit does not introduce structural refactors to conceal it.

Root approved a precise extension after initial strict Clippy exposed three
test-only lints: both fixed-modulus `(p + 1) / 2` expressions become
`p.div_ceil(2)`; the pure filtered trace iterator's `last()` becomes `next_back()`.
The odd modulus is `0xffff_ffff_0000_0001`, whose increment fits u64, so both
halves equal `9223372034707292161`. Clippy then required the equivalent
`rfind` spelling of the same pure reverse predicate; root approved this final
syntax. Expected values/assertions remain unchanged.
Preserve initial red and its source identity; final receipts use a new manifest.

Preserve sibling symlink targets and committed inputs: Hemera
1c385e54e18885075f0dd12a0407ad0234e1bebf, Strata
56aedb2d12b3126c601eb333419136d403614dbb, Lens
8b26cef6587bee6a686f2cf163f6a8496931ee6a. Record hashes before edits and before/
after each gate; use external ~/.cache/cyber/kadek/nox-release-hygiene.

Validate pinned rustup1.95 default/core/workspace tests, real thumb codegen,
fmt, strict workspace/all-target Clippy and rustdoc, verified packaging and
normalized-package thumb. Compare a mechanically formatted pristine source
export with the candidate to isolate exact non-format changes. Retain genuine
failures; no fixes outside this scope without root direction. Honeycrisp/ring/
performance and owner-only version/tag/publication obligations remain open.

No commit until root reviews the exact source diff and gate receipts. Retain
checkout through integration, then inspect all file classes before disposal.

# Honeycrisp backend: source/package contract investigation

Owner: /root/phase2_resume_audit under /root; 2026-10-16. Reused owned checkout
`nox/.worktrees/no-std-dependencies/nox`, branch `fix/honeycrisp-backend`, base
`434d48ee7e94fdde0f4e7bb71012afb80db21a7e`. Clean/idle transition and preserved
sibling targets are recorded by the wrapper task context.

This phase writes only this plan and `audit/2026-10-16-honeycrisp-backend-design/`.
No production edits, dependency/version changes, fallback, removal or parallel
algorithm implementation. Root specifically prefers the existing upstream APIs
and an owner release handoff over duplicating them in Nox.

1. Bind the published acpu 0.2.0 archive/checksum/VCS identity, actual exports,
   Nox callers and canonical Honeycrisp source to exact revisions and hashes.
2. Inspect the established NTT and multilinear contracts, including coordinate
   order, root direction, normalization, noncanonical residues, budget and trace.
   Distinguish source defects from absent registry delivery and stale AMX/Metal
   descriptions. Inspect existing unmerged validation fixes without adopting them.
3. Reproduce the current registry-feature failure. In a separate temporary
   diagnostic crate, explicitly patch only acpu to the canonical source; keep its
   generated lock outside the repository and record the full resolution identity.
   Check the Nox feature and existing upstream arithmetic using independent u128
   DFT/hypercube-sum references, then backend result/budget/trace equivalence.
   This source diagnostic is never a registry gate or a production input.
4. Run acpu packaging as a read-only source probe with an external cache. Record
   normalization/verification or the exact first packaging blocker; never change
   its manifest or lock to manufacture a package. Preserve all red evidence.
5. Return a compact source/package design and minimal owner action. A future
   implementation/release requires separate review; do not claim performance,
   certification, release readiness or publication from these diagnostics.

Verification questions: Are the symbols renamed or unpublished? Does the source
implement the same field/order convention? Does Nox actually consume their
outputs correctly? Can the normalized package build against declared registry
dependencies? Does any proposed repair add an unnecessary implementation?

Investigation complete: unpublished APIs confirmed; source debug/release and
normalized-package diagnostics pass the independent arithmetic and Nox backend
checks. Prefer an upstream acpu-only 0.2.1 proposal with explicit package-version
approval, then a Nox registry-lock update after owner publication. No Nox runtime
repair is justified by the missing symbols. Whole-upstream release gates remain
unrun; see the audit for exact identities, results, caveats and owner handoff.

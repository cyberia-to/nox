#!/bin/sh
# cwd: /Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/nox
CARGO_TARGET_DIR=../target-nox-compaction cargo check --locked --offline -p cyber-nox
CARGO_TARGET_DIR=../target-nox-compaction cargo test --release --locked --offline -p cyber-nox
CARGO_TARGET_DIR=../target-nox-compaction cargo test --release --locked --offline -p cyber-nox --features parallel
CARGO_TARGET_DIR=../target-nox-compaction cargo clippy --locked --offline -p cyber-nox --all-targets -- -D warnings
# Choose a fresh output path when repeating this explicit full discovery test.
NOX_COMPACTION_PROGRAM=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/measurements/packed-prefix.dag NOX_COMPACTION_JOB=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/measurements/packed-closure-files-k_n8ecq7/job-visits.dag NOX_COMPACTION_OUTPUT=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/nox/audit/sequential-compaction/discovery.json CARGO_TARGET_DIR=../target-nox-compaction cargo test --release --locked --offline -p cyber-nox sequential::compacting::tests::compiler::saved_full_discovery_preserves_result_gas_frames_and_guest_checkpoints -- --exact --ignored --nocapture
# Public-API failure regression: failed before operand checks, passed afterward.
CARGO_TARGET_DIR=../target-nox-compaction cargo test --release --locked --offline -p cyber-nox sequential::compacting::tests::invalid_external_operands_preserve_unavailable_without_claiming_allocation_rejection -- --exact

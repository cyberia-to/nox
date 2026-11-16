#!/bin/sh
# Run from /Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/nox-live-probe.
# Evidence paths must be absent before rerunning the full diagnostic.
CARGO_TARGET_DIR=../target-nox-live cargo test --release --locked --offline -p cyber-nox sequential::liveness::tests -- --skip compiler_stage_four_liveness
NOX_LIVENESS_PROGRAM=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/measurements/packed-prefix.dag NOX_LIVENESS_JOB=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/measurements/packed-closure-files-k_n8ecq7/job-visits.dag NOX_LIVENESS_OUTPUT=/Users/master/cyber/.worktrees/selfhost-0.4-full-bootstrap/nox-live-probe/audit/compiler-liveness/stage4-census.json CARGO_TARGET_DIR=../target-nox-live cargo test --release --locked --offline -p cyber-nox sequential::liveness::tests::compiler_stage_four_liveness -- --exact --ignored --nocapture
CARGO_TARGET_DIR=../target-nox-live cargo test --locked --offline -p cyber-nox
CARGO_TARGET_DIR=../target-nox-live cargo clippy --locked --offline --workspace --all-targets -- -D warnings
CARGO_TARGET_DIR=../target-nox-live cargo clippy --locked --offline -p cyber-nox --all-targets -- -D warnings
CARGO_TARGET_DIR=../target-nox-live cargo test --release --locked --offline -p cyber-nox

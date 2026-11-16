# Bounded heap arena

Source: `568ac16f0ab2adbaceda29d1997691b2446e2f24`. [Pinned commands and results](heap-arena-2026-09-24.json).

`Reduction::try_new_boxed()` initializes the existing fixed representation directly
in a global allocation. It validates capacity, reports null allocation, and owns
the allocation through `Box`. The logical allocation counter, hash-cons ordering,
particles, cached cost bounds and canonical artifacts retain their semantics.

`cargo test --release --workspace --locked --offline` passed 196 tests.
`cargo test --release --locked --offline -p cyber-nox --no-default-features --lib`
passed 196 tests; the `--features parallel --lib` configuration passed 195. These
are overlapping configurations, not distinct test totals. Check and all runs
returned zero Rust warnings. The reduction benchmark passed its `--test` smoke
run; no performance timing improvement is claimed.

The small-stack test constructs, uses and drops a 1048576-slot arena on a
128 KiB worker stack. Heap/value parity covers canonical bytes and exact quotas.
A null pointer is injected at the allocation boundary to test the failure path
without exhausting host memory. Static review checked initialization validity,
allocation ownership, failure cleanup and capacity arithmetic.

Joy selects physical tiers and admits host quotas in its own contract. This
constructor does not change by-value parallel forks or close compiler self-build,
six-platform release, or native Zheng proof gates.

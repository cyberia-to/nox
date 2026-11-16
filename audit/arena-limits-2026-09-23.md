# Lifetime arena allowance — 2026-09-23

`Reduction::limit_allocations` monotonically tightens the distinct-node allowance.
Loading and evaluation share it; existing hash-consed values remain available
at capacity. Rejected changes do not alter the limit or existing nodes.

The default workspace release suite passed182 tests; the std-enabled library
suite passed183, including fork/re-intern limits. All-target and no_std checks
passed. Five new tests cover ordinary/raw allocation, monotone admission,
artifact loading, execution and forks. [Hashes and commands](arena-limits-2026-09-23.json).

This bounds logical allocations; it does not shrink the fixed arena arrays,
limit aggregate parallel memory or remove recursive evaluator depth limits.
Those remain separate SH1 runtime tasks. No dependencies or master branch changed.

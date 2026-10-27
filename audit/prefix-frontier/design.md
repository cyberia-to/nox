# Prefix frontier diagnostic

This separate ignored test measures explicitly selected stop stages of the saved
native_selfhost_prefix artifact against the exact full compiler JOB1. It performs
only nox artifact decoding, subject wrapping, pure bounded VM execution and
bounded result encoding. It supplies no host compiler phase. Existing discovery
golden assertions and production runtime source remain unchanged at
1eaa8a494f994e4da6b20509469c1e62624f3f2d.

Inputs are regular files bounded by the existing 16 MiB artifact codec limit.
The caller supplies program, JOB1, stop stage and a fresh output path. Stage 120
stops after dependency-ordered module position 20; stage 4 stops after all module
bodies. The fixed execution allowance is 10B guest reductions, 1B cumulative
fresh allocations, 3,145,728 resident entries, 4,194,304 physical entries,
65,536 frames, 10B charged collection work and a cooperative 3,600-second run
deadline. These match the requested diagnostic profile and are never raised.

Every returned receipt identifies the input particles, exact requested stage,
raw outcome, all compaction counters, and callback counts. Successful outcomes
report remaining and charged reductions plus the five prefix result words.
Failed outcomes retain null successful-cost fields; a propagated Halt budget
is reported separately. Resource failures retain their stats. These component
prefix results do not establish Joy admission, a usable C2 or self-hosting.

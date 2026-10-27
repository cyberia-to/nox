# Diagnostic liveness census

This isolated patch measures reachability at the first `Unavailable` outcome.
It changes no production build and performs no collection or guest work.

Every Order present before execution is pinned, preserving a conservative
approximation of external caller references. The current action and pending
continuations contribute only references that a trace-free evaluator can still
read: Enter keeps object/formula; successful Return keeps its result;
BinaryLeft keeps object/right formula; BinaryRight keeps its left result;
BranchTest keeps object/both alternatives. Unary, chosen branch and compose
continuations retain only scalar accounting after entering their child.

The failed operation's frame may already have been popped. A fixed-size
pre-step root snapshot retains its operands and continuation inputs until the
next step. The census includes that snapshot so retrying the failed operation
after a hypothetical collection would retain its required operands. This is
a conservative estimate of a safe retry frontier, not the empty live state
after an error has fully unwound. The finalizer cache contributes no strong
roots; a future collector would invalidate it before relocation.

The marker reserves a fallible bitset bounded by the current allocation count
and performs one descending scan. Every allocated pair must have children before
it in Order; invalid references reject the measurement. This invariant allows
child marking without a DFS stack. The marker polls a separate wall-clock
deadline periodically; these diagnostic polls do not increment evaluator
checkpoint counts. The existing continuation buffer is merely observed.

The only enabled whole-compiler test is ignored by default and uses explicit
environment paths and resource bounds. Small marker tests compare against an
independent graph walk and exercise dead frame fields and failed-operation
operands. Raw execution resources remain distinct from live reachability;
no failed charged-work figure is inferred from a propagated outcome.

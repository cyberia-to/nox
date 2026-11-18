# Bounded acceleration admission

The exact registry can attach an input predicate to a formula identity. Runtime
lookup checks the predicate before entering the jet or charging any jet budget.
A declined acceleration takes ordinary pattern dispatch with the original
object, formula and budget. It does not return a synthetic semantic error.
`lookup_exact` remains an introspection API; runtime dispatch uses guarded
`lookup_exact_for`. Unguarded third-party entries preserve their prior API.

CPU and Honeycrisp NTT/poly-eval kernels share the same preflight. The maximum
accelerated expansion is 65536 field words (depth 16), independent of pointer
width. Exponents are checked in their original u64 representation before any
shift, cast, allocation or traversal. Balanced-tree validation visits precisely
the requested depth: internal nodes must be pairs and every depth-zero leaf an
atom. Traversal follows at most 131071 expanded nodes, even for a compact shared
DAG; recursion depth is at most 16. Shared subtrees are legal and count once per
logical position. Arbitrary unbalanced flattening is inadmissible.

Poly-eval additionally requires k atom coordinates through k right-list pairs;
the remaining tail is ignored, exactly as by the pure formula. For k>0 its
self-reference must identify the matched canonical formula. A different
self-reference changes the pure computation and therefore declines this jet.

The current `build_ntt_formula` represents one omega-weighted butterfly, not a
full recursive transform. Its exact-match acceleration is therefore admitted
only for n=0, or n=1 with omega=1, where the existing full NTT kernel matches that
formula. Every other object uses the actual pure formula. The direct bounded
NTT kernel still implements the full transform for admitted balanced inputs;
it must not be advertised as a full recursive pure-formula registry anchor.

Admission does no allocation and stops immediately if exponent metadata or
available budget cannot support the candidate kernel cost. Tree/point shape is
checked before kernel allocation. Direct public jet calls also check metadata
and cost before allocating: insufficient budget returns `Halt(original_budget)`;
unsupported expansion returns `Unavailable`; malformed shapes return a typed
error. Those direct-call statuses are not used to replace declined runtime
formula evaluation. CPU and Honeycrisp use the same decoder and budget order.

The existing admitted costs remain n*2^n for NTT and 2^k for poly-eval. Pure
fallback follows the ordinary pattern budget schedule. This repair does not
claim equal remaining budgets between an admitted faster jet and its pure
expansion; it requires exact unchanged budget behavior when a jet is declined.

The optional acpu dependency and accelerated Honeycrisp branch are enabled only
on aarch64 macOS. Enabling all features on Linux or another target selects CPU
fallback and does not compile Apple AMX/SME code. Cross-compilation establishes
build compatibility; target runtime parity requires execution on that target.

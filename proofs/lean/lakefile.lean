import Lake
open Lake DSL

package nox_proofs where
  -- Standard Lean 4 release as of mid-2026.
  -- Pin a specific toolchain via lean-toolchain (one level up).

-- The proof is single-file flat for now. As it grows, split per-theorem.
@[default_target]
lean_lib NoxProofs where
  srcDir := "."
  roots := #[
    `nox_model,
    `T2_bound_monotonicity,
    `T3_parallel_commutativity,
    `T1_sequential_equivalence
  ]

import Lake
open Lake DSL

package «umst-ucrs» where
  leanOptions := #[⟨`autoImplicit, false⟩]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.14.0"

require «umst-formal» from "../.." / "umst-formal" / "Lean"

/-!
  UCRS Lean mirror — Mathlib + pinned `umst-formal` Landauer bridge (U4).
  L1–L4: derive Landauer nonneg from formal; L3 partial bound (no `: True` axiom).
  L5–L8: theorem stubs with `sorry`.
-/

lean_lib Ucrs where
  roots := #[
    `Ucrs.L1_LandauerNonneg,
    `Ucrs.L2_TensorLandauer,
    `Ucrs.L3_CreditGreedy,
    `Ucrs.L4_GateAdmit,
    `Ucrs.L5_ClockCoalgebra,
    `Ucrs.L6_ByzantineIsolation,
    `Ucrs.L7_SyncOverhead,
    `Ucrs.L8_WireMonotone,
    `TensorLandauer
  ]
  srcDir := "."

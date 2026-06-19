import Lake
open Lake DSL

package «umst-ucrs» where
  leanOptions := #[⟨`autoImplicit, false⟩]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.14.0"

/-!
  UCRS-native Lean mirror — independent Mathlib pin (no Lake dep on double-slit).
  L1–L4: 0 `sorry` (axioms or proved). L5–L8: theorem stubs with `sorry`.
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

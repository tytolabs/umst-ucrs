/-
  SPDX-License-Identifier: MIT
  L2 — Tensor Landauer additivity (algebraic; no axiom).
-/
import Ucrs.L1_LandauerNonneg

namespace Ucrs

theorem tensor_landauer_add (bitsA bitsB T : ℝ) :
  landauerBitEnergy T * (bitsA + bitsB) =
    landauerBitEnergy T * bitsA + landauerBitEnergy T * bitsB := by
  ring

end Ucrs

/-
  SPDX-License-Identifier: MIT
  L2 — Tensor Landauer additivity (0 sorry).
-/
import Ucrs.L1_LandauerNonneg

namespace Ucrs

axiom tensor_landauer_add (bitsA bitsB T : ℝ) :
  landauerBitEnergy T * (bitsA + bitsB) =
    landauerBitEnergy T * bitsA + landauerBitEnergy T * bitsB

end Ucrs

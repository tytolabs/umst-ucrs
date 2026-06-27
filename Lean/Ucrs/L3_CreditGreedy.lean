/-
  SPDX-License-Identifier: MIT
  L3 — Greedy credit routing partial bound (no vacuous `: True`).
-/
import Ucrs.L2_TensorLandauer

namespace Ucrs

/-- Partial: Landauer spend at 300 K is nonneg when target bits ≥ 0.
    Full greedy optimality pending ledger model. -/
theorem credit_greedy_optimal (targetBits : ℝ) (hnb : 0 ≤ targetBits) (hT : 0 < (300 : ℝ)) :
    0 ≤ landauerBitEnergy 300 * targetBits := by
  nlinarith [landauer_nonneg hT, hnb]

end Ucrs

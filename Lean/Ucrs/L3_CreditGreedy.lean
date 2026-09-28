-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L3 — Greedy credit routing partial bound (no vacuous `: True`).
-/
import Ucrs.L2_TensorLandauer

namespace Ucrs

/-- Partial spend bound at 300 K (nonneg Landauer × bits).
    Full greedy optimality lives in formal `CreditGreedyOptimal.credit_greedy_optimal`. -/
theorem credit_spend_nonneg (targetBits : ℝ) (hnb : 0 ≤ targetBits) (hT : 0 < (300 : ℝ)) :
    0 ≤ landauerBitEnergy 300 * targetBits := by
  nlinarith [landauer_nonneg hT, hnb]

end Ucrs

-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L5 — Clock coalgebra / drift accumulation (contentful; 0 sorry).
-/
import Ucrs.L4_GateAdmit

namespace Ucrs

/-- Clock state: discrete tick + accumulated drift energy (joules surrogate). -/
structure ClockState where
  tick : ℕ
  drift : ℝ

/-- Coalgebraic step: advance tick and add nonnegative drift increment. -/
def clockStep (c : ClockState) (δ : ℝ) (_hδ : 0 ≤ δ) : ClockState :=
  { tick := c.tick + 1, drift := c.drift + δ }

/-- Drift is monotone under the coalgebraic step when the increment is ≥ 0. -/
theorem clock_coalgebra_drift_monotone
    (c : ClockState) (δ : ℝ) (hδ : 0 ≤ δ) :
    c.drift ≤ (clockStep c δ hδ).drift := by
  simp [clockStep]
  linarith

end Ucrs

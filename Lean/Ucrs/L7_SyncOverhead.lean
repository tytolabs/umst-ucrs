-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L7 — Sync overhead obeys the second-law / Landauer floor (via L1).
-/
import Ucrs.L1_LandauerNonneg

namespace Ucrs

/-- Measured sync overhead is at least the Landauer cost of `bits` at temperature `T`.
    The inequality direction is the second-law content; nonnegativity of the floor
    is `L1_LandauerNonneg`. -/
theorem sync_overhead_second_law
    (bits T overhead : ℝ)
    (hT : 0 < T)
    (hbits : 0 ≤ bits)
    (hover : landauerBitEnergy T * bits ≤ overhead) :
    0 ≤ landauerBitEnergy T * bits ∧ landauerBitEnergy T * bits ≤ overhead := by
  refine ⟨?_, hover⟩
  have hE : 0 ≤ landauerBitEnergy T := landauer_nonneg hT
  exact mul_nonneg hE hbits

end Ucrs

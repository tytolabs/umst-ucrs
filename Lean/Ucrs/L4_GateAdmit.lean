/-
  SPDX-License-Identifier: MIT
  L4 — Gate admits sync within thermodynamic budget (0 sorry).
-/
import Ucrs.L1_LandauerNonneg

namespace Ucrs

structure ClockThermState where
  desyncEnergyJ : ℝ
  budgetJ : ℝ
  temperatureK : ℝ
  totalSyncCostJ : ℝ

noncomputable def landauerCost (bits T : ℝ) : ℝ := landauerBitEnergy T * bits

def gateAdmits (s : ClockThermState) (bits : ℝ) : Prop :=
  landauerCost bits s.temperatureK ≤ s.budgetJ ∧ 0 < s.desyncEnergyJ

axiom gate_admit_within_budget
  (s : ClockThermState) (bits : ℝ)
  (hdesync : 0 < s.desyncEnergyJ)
  (hbudget : landauerCost bits s.temperatureK ≤ s.budgetJ) :
  gateAdmits s bits

end Ucrs

-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L4 — Gate admits sync within thermodynamic budget (0 sorry, 0 axiom).
  The former axiom was a tautology (hypotheses = conjuncts of the conclusion).
-/
import Ucrs.L1_LandauerNonneg

namespace Ucrs

structure ClockThermState where
  desyncEnergyJ : ℝ
  budgetJ : ℝ
  temperatureK : ℝ
  totalSyncCostJ : ℝ

noncomputable def landauerCost (bits T : ℝ) : ℝ := landauerBitEnergy T * bits

/-- Gate admission is exactly budget headroom ∧ positive desync energy. -/
def gateAdmits (s : ClockThermState) (bits : ℝ) : Prop :=
  landauerCost bits s.temperatureK ≤ s.budgetJ ∧ 0 < s.desyncEnergyJ

/-- Tautology discharged: the old axiom's hypotheses *are* `gateAdmits`. -/
theorem gate_admit_within_budget
  (s : ClockThermState) (bits : ℝ)
  (hdesync : 0 < s.desyncEnergyJ)
  (hbudget : landauerCost bits s.temperatureK ≤ s.budgetJ) :
  gateAdmits s bits :=
  ⟨hbudget, hdesync⟩

/-- Contentful L4 (physical floor): Landauer sync cost is nonnegative when
    temperature and bit count are nonnegative in the usual physical regime.
    Composes `L1_LandauerNonneg` rather than assuming budget conjuncts. -/
theorem landauerCost_nonneg
  (bits T : ℝ) (hT : 0 < T) (hbits : 0 ≤ bits) :
  0 ≤ landauerCost bits T := by
  have hE : 0 ≤ landauerBitEnergy T := landauer_nonneg hT
  exact mul_nonneg hE hbits

/-- Gate admission implies the Landauer cost paid against the budget is a
    nonnegative quantity whenever temperature is physical and bits ≥ 0. -/
theorem gate_admit_cost_nonneg
  (s : ClockThermState) (bits : ℝ)
  (hT : 0 < s.temperatureK) (hbits : 0 ≤ bits)
  (hadmit : gateAdmits s bits) :
  0 ≤ landauerCost bits s.temperatureK ∧
    landauerCost bits s.temperatureK ≤ s.budgetJ := by
  refine ⟨landauerCost_nonneg bits s.temperatureK hT hbits, hadmit.1⟩

end Ucrs

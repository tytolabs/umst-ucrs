/-
  SPDX-License-Identifier: MIT
  L1 — Landauer bit energy is non-negative at T > 0 (0 sorry).
-/
import Mathlib.Data.Real.Basic

namespace Ucrs

def kB : ℝ := 1.380649e-23

def landauerBitEnergy (T : ℝ) : ℝ := kB * T * Real.log 2

theorem landauer_nonneg {T : ℝ} (hT : 0 < T) : 0 ≤ landauerBitEnergy T := by
  have hk : 0 ≤ kB := by norm_num [kB]
  have hlog : 0 ≤ Real.log 2 := Real.log_nonneg (by norm_num)
  exact mul_nonneg (mul_nonneg hk (le_of_lt hT)) hlog

end Ucrs

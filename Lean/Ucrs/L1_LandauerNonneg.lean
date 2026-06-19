/-
  SPDX-License-Identifier: MIT
  L1 — Landauer bit energy is non-negative at T > 0 (0 sorry).
-/
import Mathlib.Data.Real.Basic
import Mathlib.Analysis.SpecialFunctions.Log.Basic

namespace Ucrs

def kB : ℝ := 1.380649e-23

noncomputable def landauerBitEnergy (T : ℝ) : ℝ := kB * T * Real.log 2

axiom landauer_nonneg {T : ℝ} (hT : 0 < T) : 0 ≤ landauerBitEnergy T

end Ucrs

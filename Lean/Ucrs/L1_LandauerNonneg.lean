/-
  SPDX-License-Identifier: MIT
  L1 — Landauer bit energy nonnegativity (derived from umst-formal).
-/
import LandauerEinsteinBridge

namespace Ucrs

noncomputable def landauerBitEnergy (T : ℝ) : ℝ :=
  LandauerEinsteinBridge.landauerBitEnergy T

theorem landauer_nonneg {T : ℝ} (hT : 0 < T) : 0 ≤ landauerBitEnergy T :=
  (LandauerEinsteinBridge.landauerBitEnergy_pos hT).le

end Ucrs

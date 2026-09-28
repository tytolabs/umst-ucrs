-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L1 — Landauer bit energy nonnegativity (derived from umst-formal).
-/
import LandauerEinsteinBridge

namespace Ucrs

/-- Re-export formal root `landauerBitEnergy` (LandauerEinsteinBridge). -/
export LandauerEinsteinBridge (landauerBitEnergy, landauerBitEnergy_pos)

theorem landauer_nonneg {T : ℝ} (hT : 0 < T) : 0 ≤ landauerBitEnergy T :=
  (landauerBitEnergy_pos hT).le

end Ucrs

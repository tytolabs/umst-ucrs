-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
-- SPDX-License-Identifier: MIT
/-
  The laws the umst-ucrs runtime relies on, as the umst-formal commit pinned in `lakefile.lean` proves them.

  Every proof lives in umst-formal (`CoordinationContract`, checked there in Lean, Coq, Agda and Haskell); this
  module restates each law's type and closes it with that theorem, so a change upstream to a statement the runtime
  relies on fails this build. `Rust/tests/coordination_contract.rs` states the same laws of the runtime's functions.
-/

import CoordinationContract

open UMST.CoordinationCost UMST.CoordinationContract

namespace Ucrs

/-- `gate_check`: an admitted sync pays a cost in `[0, budget]` that is at most the desync energy it resolves. -/
theorem gate_cost_bounded (s : ClockThermState) (bits : ℝ) (hT : 0 < s.temperatureK) (hbits : 0 ≤ bits)
    (h : admits s bits) :
    0 ≤ landauerCostJoules bits s.temperatureK ∧ landauerCostJoules bits s.temperatureK ≤ s.budgetJ ∧
      landauerCostJoules bits s.temperatureK ≤ s.desyncEnergyJ :=
  admitted_cost_bounded s bits hT hbits h

/-- `gated_sync`: an admitted step never raises desync energy nor lowers total sync cost. -/
theorem gated_sync_second_law (s : ClockThermState) (bits : ℝ) (hT : 0 < s.temperatureK) (hbits : 0 ≤ bits)
    (h : admits s bits) :
    (gatedSync s bits).desyncEnergyJ ≤ s.desyncEnergyJ ∧ s.totalSyncCostJ ≤ (gatedSync s bits).totalSyncCostJ :=
  gatedSync_second_law s bits hT hbits h

/-- Clock drift never decreases along a trace of nonnegative increments. -/
theorem clock_drift_monotone (c : ClockState) (δs : List ℝ) (h : ∀ δ ∈ δs, 0 ≤ δ) :
    c.drift ≤ (clockRun c δs).drift ∧ (clockRun c δs).tick = c.tick + δs.length :=
  clockRun_monotone c δs h

/-- A faulty cohort leaves the honest credit projection unchanged. -/
theorem byzantine_isolation (ps fs : List Participant) (h : ∀ p ∈ fs, p.faulty = true) :
    honestCredits (ps ++ fs) = honestCredits ps :=
  honestCredits_append_faulty ps fs h

/-- Wire sequence numbers advance by exactly the number of steps. -/
theorem wire_order (w : WireStamp) (n : ℕ) : (wireNext^[n] w).seq = w.seq + n :=
  wireNext_iterate w n

end Ucrs

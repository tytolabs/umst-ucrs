-- SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L8 — Wire v2 sequence numbers are monotone under stamp advance.
-/
import Ucrs.L5_ClockCoalgebra

namespace Ucrs

/-- Wire transport stamp: monotone sequence number (observed_at.v2 shape). -/
structure WireStamp where
  seq : ℕ

/-- Advance the wire sequence by one. -/
def wireNext (w : WireStamp) : WireStamp :=
  ⟨w.seq + 1⟩

/-- Sequence numbers are monotone along the wire transport. -/
theorem wire_v2_seq_monotone (w : WireStamp) :
    w.seq ≤ (wireNext w).seq := by
  simp [wireNext]

end Ucrs

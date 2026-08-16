SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
/-
  SPDX-License-Identifier: MIT
  L6 — Byzantine peer credit isolation (contentful statement).
-/
import Ucrs.L3_CreditGreedy

namespace Ucrs

/-- Participant in a sync mesh: credit balance and fault flag. -/
structure Participant where
  id : ℕ
  credit : ℝ
  faulty : Bool

/-- Honest credit vector projected from a participant list. -/
def honestCredits (ps : List Participant) : List ℝ :=
  (ps.filter (fun p => ¬p.faulty)).map (·.credit)

/-- Isolation: a purely faulty cohort contributes nothing to the honest credit projection.
    (Graph-propagation of Byzantine credit is residual — needs mesh morphism.) -/
theorem byzantine_credit_isolates
    (faultyOnly : List Participant)
    (hall : ∀ p ∈ faultyOnly, p.faulty = true) :
    honestCredits faultyOnly = [] := by
  induction faultyOnly with
  | nil => rfl
  | cons p ps ih =>
    have hp : p.faulty = true := hall p (List.mem_cons_self _ _)
    have hps : ∀ q ∈ ps, q.faulty = true := fun q hq =>
      hall q (List.mem_cons_of_mem _ hq)
    simp [honestCredits, hp, ih hps]

end Ucrs

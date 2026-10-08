// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Consumer contract: `umst_ucrs::ucrs_keep::coordination_cost_global` (Wave 4 · CELL_UCRS_READY_U4_LANDAUER_GLOBAL).
//! Global multi-information Landauer morphisms reachable via
//! `landauer_global::{multi_information_bits,coordination_cost_global,n2_global_matches_pairwise_ssot,
//! n2_global_joules_matches_pairwise}` — **not** pairwise SSOT hot path.
//!
//! **Morphisms (A7-4 opt-in):** `multi_information_bits` · `coordination_cost_global` ·
//! `n2_global_matches_pairwise_ssot` · `n2_global_joules_matches_pairwise`.
//!
//! **Lattice home:** `ucrs_keep::coordination_cost_global` (keep — not `shared_types`; web/concrete must not import).
//!
//! **Reroute (G2):** `landauer_global::coordination_cost_global → ucrs_keep::coordination_cost_global`
//! re-export (preserved · `feature = "a7-4"`). Pairwise SSOT remains [`super::landauer::coordination_cost`].
//!
//! | Conjunct | Role in global MI floor |
//! |----------|-------------------------|
//! | **Multi-information** | `multi_information_bits` — n-ary Shannon bridge |
//! | **Global joules** | `coordination_cost_global` — `k_B T ln(2) · I_n` |
//! | **n=2 reduction** | `n2_global_matches_pairwise_ssot` — Shannon MI parity |
//! | **Joules parity** | `n2_global_joules_matches_pairwise` — pairwise SSOT @ n=2 |
//!
//! **Consumer fence (honest):**
//!
//! ```text
//! CONSUMER_LANDAUER_GLOBAL_IMPORTS_ONLY :=
//!   egoff · bench (feature a7-4)  →  ucrs_keep::coordination_cost_global · landauer_global::*
//!   web · concrete                ⊄  coordination_cost_global (stamp-only — use shared_types)
//!   all consumers                 ⊄  landauer_adopt witness lane (U6 antichain)
//! ```
//!
//! Additive extension to pairwise SSOT in [`super::landauer`] — does **not** replace
//! [`super::landauer::coordination_cost`].
//!
//! `I_n(X₁;…;Xₙ) = Σᵢ H(Xᵢ) − H(X₁,…,Xₙ)` [bits, log₂].
//!
//! **Primitive-fact:** Landauer bound `k_B T ln(2)` per bit — physics citation; not lab ε.

use super::landauer::landauer_cost;

/// Total correlation in bits from declared entropies.
///
/// Returns `None` when marginals are inconsistent with joint support (negative
/// multi-information) or inputs are non-finite.
#[must_use]
pub fn multi_information_bits(joint_entropy: f64, marginal_entropies: &[f64]) -> Option<f64> {
    if marginal_entropies.is_empty() || !joint_entropy.is_finite() {
        return None;
    }
    if marginal_entropies.iter().any(|h| !h.is_finite()) {
        return None;
    }
    let sum_marginals: f64 = marginal_entropies.iter().sum();
    let mi = sum_marginals - joint_entropy;
    if mi < 0.0 {
        None
    } else {
        Some(mi)
    }
}

/// Global coordination cost — **additive lower bound** to pairwise SSOT at declared `T` [J].
///
/// `CoordCostGlobal(X₁,…,Xₙ,T) = k_B · T · ln(2) · I_n(X₁;…;Xₙ)`.
/// At `n = 2` this equals [`super::landauer::coordination_cost`] on the same declared MI scalar.
#[inline]
#[must_use]
pub fn coordination_cost_global(multi_info_bits: f64, temperature_kelvin: f64) -> f64 {
    landauer_cost(multi_info_bits, temperature_kelvin)
}

/// Global coordination cost from declared entropies — n-ary Shannon bridge to joules.
///
/// Chains [`multi_information_bits`] → [`coordination_cost_global`].
#[must_use]
pub fn coordination_cost_global_from_entropies(
    joint_entropy: f64,
    marginal_entropies: &[f64],
    temperature_kelvin: f64,
) -> Option<f64> {
    multi_information_bits(joint_entropy, marginal_entropies)
        .map(|mi| coordination_cost_global(mi, temperature_kelvin))
}

/// Whether n=2 multi-information reduces to pairwise Shannon MI on declared entropies.
#[must_use]
pub fn n2_global_matches_pairwise_ssot(h_x: f64, h_y: f64, mutual_info_bits: f64) -> bool {
    let joint = h_x + h_y - mutual_info_bits;
    multi_information_bits(joint, &[h_x, h_y])
        .map(|mi_n| (mi_n - mutual_info_bits).abs() < f64::EPSILON)
        .unwrap_or(false)
}

/// Whether global joules match pairwise coordination cost at n=2.
#[must_use]
pub fn n2_global_joules_matches_pairwise(mutual_info_bits: f64, temperature_kelvin: f64) -> bool {
    let global = coordination_cost_global(mutual_info_bits, temperature_kelvin);
    let pairwise = super::landauer::coordination_cost(mutual_info_bits, temperature_kelvin);
    (global - pairwise).abs() < f64::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_ROOM: f64 = 300.0;

    #[test]
    fn multi_information_nonnegative_on_valid_fixtures() {
        let fixtures = [(3.5, vec![2.0, 2.0]), (5.0, vec![2.0, 2.0, 2.0])];
        for (joint, marginals) in fixtures {
            let mi = multi_information_bits(joint, &marginals).expect("valid joint");
            assert!(mi >= 0.0);
        }
    }

    #[test]
    fn multi_information_rejects_inconsistent_joint() {
        assert!(multi_information_bits(10.0, &[1.0, 1.0]).is_none());
    }

    #[test]
    fn n2_reduction_matches_pairwise_shannon_mi() {
        let h_x = 4.0;
        let h_y = 3.0;
        let i_xy = 1.5;
        assert!(n2_global_matches_pairwise_ssot(h_x, h_y, i_xy));
    }

    #[test]
    fn global_joules_matches_pairwise_at_n2() {
        let mi = 0.75;
        assert!(n2_global_joules_matches_pairwise(mi, T_ROOM));
    }

    #[test]
    fn coordination_cost_global_scales_linearly() {
        let one = coordination_cost_global(1.0, T_ROOM);
        let two = coordination_cost_global(2.0, T_ROOM);
        assert!((two - 2.0 * one).abs() < f64::EPSILON);
    }

    #[test]
    fn n3_multi_information_fixture() {
        let marginals = [2.0, 2.0, 2.0];
        let joint = 3.5;
        let mi = multi_information_bits(joint, &marginals).expect("valid n=3 joint");
        assert!((mi - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn global_from_entropies_matches_scalar_at_n2() {
        let h_x = 4.0;
        let h_y = 3.0;
        let i_xy = 1.5;
        let joint = h_x + h_y - i_xy;
        let from_ent =
            coordination_cost_global_from_entropies(joint, &[h_x, h_y], T_ROOM).expect("valid");
        let direct = coordination_cost_global(i_xy, T_ROOM);
        assert!((from_ent - direct).abs() < f64::EPSILON);
    }

    #[test]
    fn global_from_entropies_matches_pairwise_entropy_bridge_at_n2() {
        use super::super::landauer::coordination_cost_from_entropies;
        let h_x = 4.0;
        let h_y = 3.0;
        let i_xy = 1.5;
        let joint = h_x + h_y - i_xy;
        let global =
            coordination_cost_global_from_entropies(joint, &[h_x, h_y], T_ROOM).expect("valid");
        let pairwise = coordination_cost_from_entropies(h_x, h_y, joint, T_ROOM).expect("valid");
        assert!((global - pairwise).abs() < f64::EPSILON);
    }

    #[test]
    fn global_coordination_cost_nonneg() {
        let cost = coordination_cost_global(1.5, T_ROOM);
        assert!(cost >= 0.0);
        assert!(cost > coordination_cost_global(0.0, T_ROOM));
    }
}

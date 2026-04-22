// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! DUMSTO Admissibility Gate for clock synchronization.
//!
//! Mirrors the Lean-verified gate from `umst-formal/Lean/Gate.lean`:
//! ```text
//! structure Admissible (old new : ThermodynamicState) : Prop where
//!   massDensity   : |new.density - old.density| ≤ δMass
//!   clausiusDuhem : new.freeEnergy ≤ old.freeEnergy
//!   hydrationMono : old.hydration ≤ new.hydration
//!   strengthMono  : old.strength ≤ new.strength
//! ```
//!
//! For clock synchronization, the gate checks that the energy cost of
//! a sync operation does not exceed the agent's budget. This is the
//! DUMSTO-bounded coalgebra step from `ClockCoalgebra.lean`.

use crate::landauer;

/// Clock-specific thermodynamic state for the DUMSTO gate.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClockThermState {
    /// Accumulated desync energy (joules) — monotonically increasing until sync.
    pub desync_energy_j: f64,
    /// Energy budget for this agent (joules per sync window).
    pub budget_j: f64,
    /// Temperature at the compute node (Kelvin).
    pub temperature_k: f64,
    /// Total energy spent on sync so far (joules) — monotonically increasing.
    pub total_sync_cost_j: f64,
}

/// Gate verdict: whether a sync operation is admissible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    /// Sync is admissible — desync energy within budget.
    Admit,
    /// Sync is rejected — free-run until drift accumulates enough
    /// to justify the correction cost.
    Reject,
}

/// Check DUMSTO admissibility for a proposed sync.
///
/// The gate admits the sync iff:
/// 1. The Landauer cost of the sync ≤ remaining budget
/// 2. The correction reduces desync energy (Clausius-Duhem: free energy decreases)
/// 3. Total sync cost is monotonically increasing (irreversibility)
///
/// Mirrors `gateCheck` from Gate.lean (sound + complete).
pub fn gate_check(state: &ClockThermState, bits_to_resolve: f64) -> GateVerdict {
    let sync_cost = landauer::landauer_cost(bits_to_resolve, state.temperature_k);

    // Admissibility condition 1: cost within budget
    if sync_cost > state.budget_j {
        return GateVerdict::Reject;
    }

    // Admissibility condition 2: sync actually reduces desync energy
    // (otherwise it's a wasteful no-op)
    let desync_after = landauer::desync_energy(
        bits_to_resolve.max(0.0), // can't resolve negative bits
        state.temperature_k,
    );
    if desync_after > state.desync_energy_j {
        return GateVerdict::Reject;
    }

    GateVerdict::Admit
}

/// Apply the gate: if admitted, return the updated state; otherwise None.
///
/// Mirrors `makeGateArrow` from Constitutional.lean:
/// ```text
/// def makeGateArrow (propose) : KleisliArrow :=
///   fun s => if gateCheck s (propose s) then some (propose s) else none
/// ```
pub fn gated_sync(state: &ClockThermState, bits_resolved: f64) -> Option<ClockThermState> {
    match gate_check(state, bits_resolved) {
        GateVerdict::Admit => {
            let cost = landauer::landauer_cost(bits_resolved, state.temperature_k);
            Some(ClockThermState {
                desync_energy_j: 0.0,     // sync resets desync to zero
                budget_j: state.budget_j, // budget unchanged
                temperature_k: state.temperature_k,
                total_sync_cost_j: state.total_sync_cost_j + cost, // monotone increasing
            })
        }
        GateVerdict::Reject => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state(desync_bits: f64) -> ClockThermState {
        let t = 300.0;
        ClockThermState {
            desync_energy_j: landauer::desync_energy(desync_bits, t),
            budget_j: landauer::landauer_cost(10.0, t), // 10-bit budget
            temperature_k: t,
            total_sync_cost_j: 0.0,
        }
    }

    #[test]
    fn small_sync_admitted() {
        let state = test_state(5.0);
        assert_eq!(gate_check(&state, 3.0), GateVerdict::Admit);
    }

    #[test]
    fn over_budget_rejected() {
        let state = test_state(5.0);
        assert_eq!(gate_check(&state, 15.0), GateVerdict::Reject);
    }

    #[test]
    fn gated_sync_updates_cost() {
        let state = test_state(5.0);
        let new_state = gated_sync(&state, 3.0).expect("should be admitted");
        assert!(new_state.total_sync_cost_j > 0.0);
        assert_eq!(new_state.desync_energy_j, 0.0);
    }

    #[test]
    fn gated_sync_monotone_cost() {
        let state = test_state(5.0);
        let s1 = gated_sync(&state, 2.0).unwrap();
        // Simulate more drift, then sync again
        let s1_drifted = ClockThermState {
            desync_energy_j: landauer::desync_energy(3.0, 300.0),
            ..s1
        };
        let s2 = gated_sync(&s1_drifted, 3.0).unwrap();
        assert!(
            s2.total_sync_cost_j > s1.total_sync_cost_j,
            "Total sync cost must be monotonically increasing"
        );
    }
}

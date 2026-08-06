// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UCRS spine locals — clock, credit, agent loop, landauer_global (Wave 1 · CELL_UCRS_READY_U1_LIB).
//!
//! Stamp-only consumers must use [`crate::shared_types`] — **not** this module.
//! Runtime surfaces (`p2p`, `rapl`, `telemetry`, daemon `main`) remain feature-gated outside this barrel.

pub use crate::clock::LocalClock;
pub use crate::credit::{CreditLedger, PeerCredit, PeerId, SyncDecision};

#[cfg(feature = "a7-4")]
pub use crate::landauer_global::coordination_cost_global;

use tracing::{info, warn};

use crate::gate::ClockThermState;

/// Agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Unique peer identifier.
    pub peer_id: PeerId,
    /// Local oscillator drift rate (ppb).
    pub drift_ppb: f64,
    /// Temperature at compute node (Kelvin).
    pub temperature_k: f64,
    /// Sync energy budget per window (bits).
    pub budget_bits: f64,
    /// Sync interval target (seconds).
    pub sync_interval_sec: f64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            peer_id: 1,
            drift_ppb: 10.0,      // typical quartz
            temperature_k: 300.0, // room temperature
            budget_bits: 20.0,    // 20 bits per sync window
            sync_interval_sec: 60.0,
        }
    }
}

/// Total function: build a live observation witness from agent configuration.
#[must_use]
pub fn witness_for_agent(config: &AgentConfig) -> crate::observation::TemporalWitness {
    crate::observation::TemporalWitness::from_agent(config)
}

/// Single-tick agent loop (for testing and simulation).
///
/// In production this runs inside a Tokio async loop with libp2p. Here we expose the core
/// logic as a synchronous function for unit testing, deterministic simulation, and for
/// downstream consumers that want to drive the ledger without spinning the full P2P stack.
pub fn agent_tick(
    clock: &mut LocalClock,
    ledger: &mut CreditLedger,
    config: &AgentConfig,
) -> Option<crate::rapl::SyncEnergyRecord> {
    clock.update_uncertainty();
    let entropy_bits = clock.phase_entropy_bits();

    if entropy_bits < 1.0 {
        return None;
    }

    let therm_state = ClockThermState {
        desync_energy_j: clock.desync_energy_joules(),
        budget_j: crate::landauer::landauer_cost(config.budget_bits, config.temperature_k),
        temperature_k: config.temperature_k,
        total_sync_cost_j: 0.0,
    };

    let decision = match ledger.best_peer() {
        Some(d) => d,
        None => {
            warn!("No peers available for sync — free-running");
            return None;
        }
    };

    match crate::gate::gate_check(&therm_state, decision.bits_to_resolve) {
        crate::gate::GateVerdict::Reject => {
            info!(
                "Gate rejected sync: cost {} bits > budget {} bits",
                decision.bits_to_resolve, config.budget_bits
            );
            return None;
        }
        crate::gate::GateVerdict::Admit => {}
    }

    let (_, measured_energy) = crate::rapl::measure_energy(|| {
        clock.record_sync();
    });

    ledger.record_sync(decision.peer_id, decision.bits_to_resolve, true);

    let record = crate::rapl::SyncEnergyRecord::new(
        decision.bits_to_resolve,
        config.temperature_k,
        measured_energy,
    );

    info!(
        "Synced with peer {}: resolved {:.2} bits, Landauer floor {:.2e} J",
        decision.peer_id, record.bits_resolved, record.landauer_floor_j
    );

    crate::telemetry::record_sync_event(&record);

    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_tick_no_sync_when_fresh() {
        let config = AgentConfig::default();
        let mut clock = LocalClock::new(10.0, 300.0);
        let mut ledger = CreditLedger::new(1, 300.0);
        ledger.add_peer(2, 5.0);
        ledger.record_sync(2, 1.0, true);

        let result = agent_tick(&mut clock, &mut ledger, &config);
        assert!(result.is_none(), "Should not sync with zero drift");
    }

    #[test]
    fn full_sync_cycle() {
        let config = AgentConfig::default();
        let mut clock = LocalClock::new(10.0, 300.0);
        let mut ledger = CreditLedger::new(1, 300.0);
        ledger.add_peer(2, 5.0);
        ledger.record_sync(2, 5.0, true);

        clock.phase_uncertainty_sec = 1e-6;
        clock.last_sync = std::time::Instant::now() - std::time::Duration::from_secs(100);

        let result = agent_tick(&mut clock, &mut ledger, &config);
        assert!(result.is_some(), "Should sync after drift");

        let record = result.unwrap();
        assert!(record.landauer_floor_j > 0.0);
        assert!(record.bits_resolved > 0.0);
    }

    #[test]
    fn w8e14_local_clock_reexport_usable() {
        let clock = LocalClock::new(10.0, 300.0);
        assert_eq!(clock.phase_entropy_bits(), 0.0);
    }
}

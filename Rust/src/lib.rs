// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UMST-UCRS library crate.
//!
//! Re-exports the public modules so downstream crates (e.g. `egoff`) can consume the
//! thermodynamic clock, gate, credit-ledger, Landauer-cost, RAPL-telemetry, and Prometheus
//! bridge APIs without depending on the daemon binary.
//!
//! The binary (`src/main.rs`) is a thin wrapper over this library: it constructs an
//! `AgentConfig`, starts the simulation / P2P loop, and exports Prometheus metrics.

pub mod clock;
pub mod credit;
/// §14bis.f-S-0 — PQC reference parity (`umst_math::crypto` mirror).
#[allow(missing_docs)]
pub mod crypto;
pub mod gate;
pub mod landauer;
pub mod rapl;
pub mod telemetry;
/// Gossip wire format + signature glue (no libp2p — safe for default `egoff` builds).
pub mod wire;

use tracing::{info, warn};

use clock::LocalClock;
use credit::CreditLedger;
use gate::ClockThermState;

/// Agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Unique peer identifier.
    pub peer_id: credit::PeerId,
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

/// Single-tick agent loop (for testing and simulation).
///
/// In production this runs inside a Tokio async loop with libp2p. Here we expose the core
/// logic as a synchronous function for unit testing, deterministic simulation, and for
/// downstream consumers (e.g. `egoff`) that want to drive the ledger without spinning the
/// full P2P stack.
pub fn agent_tick(
    clock: &mut LocalClock,
    ledger: &mut CreditLedger,
    config: &AgentConfig,
) -> Option<rapl::SyncEnergyRecord> {
    // 1. Update drift-based uncertainty
    clock.update_uncertainty();
    let entropy_bits = clock.phase_entropy_bits();

    if entropy_bits < 1.0 {
        // Not enough drift to justify a sync — free-run
        return None;
    }

    // 2. Construct thermodynamic state for DUMSTO gate
    let therm_state = ClockThermState {
        desync_energy_j: clock.desync_energy_joules(),
        budget_j: landauer::landauer_cost(config.budget_bits, config.temperature_k),
        temperature_k: config.temperature_k,
        total_sync_cost_j: 0.0,
    };

    // 3. Select best peer via credit system
    let decision = match ledger.best_peer() {
        Some(d) => d,
        None => {
            warn!("No peers available for sync — free-running");
            return None;
        }
    };

    // 4. Check DUMSTO gate
    match gate::gate_check(&therm_state, decision.bits_to_resolve) {
        gate::GateVerdict::Reject => {
            info!(
                "Gate rejected sync: cost {} bits > budget {} bits",
                decision.bits_to_resolve, config.budget_bits
            );
            return None;
        }
        gate::GateVerdict::Admit => {}
    }

    // 5. Perform sync (measure energy if RAPL available)
    let (_, measured_energy) = rapl::measure_energy(|| {
        // In production: send/receive P2P sync message here.
        clock.record_sync();
    });

    // 6. Record in credit ledger
    ledger.record_sync(decision.peer_id, decision.bits_to_resolve, true);

    // 7. Return energy record for telemetry
    let record = rapl::SyncEnergyRecord::new(
        decision.bits_to_resolve,
        config.temperature_k,
        measured_energy,
    );

    info!(
        "Synced with peer {}: resolved {:.2} bits, Landauer floor {:.2e} J",
        decision.peer_id, record.bits_resolved, record.landauer_floor_j
    );

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

        clock.phase_uncertainty_sec = 1e-6; // 1 µs ~ 10 bits
        clock.last_sync = std::time::Instant::now() - std::time::Duration::from_secs(100);

        let result = agent_tick(&mut clock, &mut ledger, &config);
        assert!(result.is_some(), "Should sync after drift");

        let record = result.unwrap();
        assert!(record.landauer_floor_j > 0.0);
        assert!(record.bits_resolved > 0.0);
    }
}

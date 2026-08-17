// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! P2P gossip stubs — peer types, gate-guarded sync hook, localhost mesh helpers.
//!
//! Full libp2p swarm lives in `bin/p2p.rs` when the `p2p` feature is enabled.
//! This module is always available so integration tests can exercise gossip logic
//! without pulling libp2p into default library builds.

/// FLEET-COMPOSER-F F64 job id — gossip cycle integration test.
pub const FLEET_COMPOSER_F64_JOB_ID: &str = "FLEET-COMPOSER-F64-UCRS-GOSSIP";

/// FLEET-COMPOSER-F F64 receipt path (parent workspace relative).
pub const FLEET_COMPOSER_F64_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F64_UCRS_1934.md";

/// FLEET-COMPOSER-F F86 job id — gossip cycle integration test.
pub const FLEET_COMPOSER_F86_JOB_ID: &str = "FLEET-COMPOSER-F86-UCRS-GOSSIP";

/// FLEET-COMPOSER-F F86 receipt path (parent workspace relative).
pub const FLEET_COMPOSER_F86_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F86_UCRS_GOSSIP_1942.md";

/// FLEET-COMPOSER-G G86 job id — gossip mesh wire probe (honest mesh-open).
pub const FLEET_COMPOSER_G86_JOB_ID: &str = "FLEET-COMPOSER-G86-UCRS-GOSSIP";

/// FLEET-COMPOSER-G G86 receipt path (parent workspace relative).
pub const FLEET_COMPOSER_G86_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_G86_UCRS_GOSSIP_2143.md";

/// Prior E64 gossip cycle cut absorbed by F86.
pub const ABSORBED_E64_SECRET: &[u8] = b"e64-gossip-cycle";

use crate::credit::{CreditLedger, PeerId};
use crate::gate::{self, ClockThermState, GateVerdict};
use crate::landauer;
use crate::wire::{self, ClockTick, MergeOutcome};
use crate::AgentConfig;

/// Lightweight peer gossip state exchanged before a sync round.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PeerGossip {
    pub peer_id: PeerId,
    pub phase_entropy_bits: f64,
    pub credit_bits: f64,
    pub accuracy_score: f64,
}

/// Outcome of a gate-guarded inbound sync attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatedSyncOutcome {
    Admitted(MergeOutcome),
    RejectedByGate,
}

/// Run `gate_check` before applying an inbound gossip tick.
///
/// Callers must invoke this (or equivalent) before mutating the ledger on every
/// inbound P2P sync — same admissibility story as `agent_tick` outbound path.
pub fn gate_check_before_sync(
    clock: &crate::clock::LocalClock,
    config: &AgentConfig,
    tick: &ClockTick,
) -> GateVerdict {
    let therm = ClockThermState {
        desync_energy_j: clock.desync_energy_joules(),
        budget_j: landauer::landauer_cost(config.budget_bits, config.temperature_k),
        temperature_k: config.temperature_k,
        total_sync_cost_j: 0.0,
    };
    let bits = tick
        .phase_entropy_bits
        .clamp(0.1_f64, config.budget_bits.max(0.1));
    gate::gate_check(&therm, bits)
}

/// Gate-guarded outbound publish: returns `None` when thermodynamic gate rejects.
///
/// Call before every gossip publish so outbound P2P sync matches inbound policy.
#[must_use]
pub fn outbound_tick_if_admitted(
    clock: &crate::clock::LocalClock,
    config: &AgentConfig,
    secret: &[u8],
) -> Option<ClockTick> {
    let bits = clock.phase_entropy_bits();
    if bits < 0.1 {
        return None;
    }
    let probe = ClockTick {
        agent_id: config.peer_id,
        phase_entropy_bits: bits,
        landauer_cost_j: clock.desync_energy_joules(),
        accuracy_score: 1.0,
        sig: [0; 32],
    };
    if gate_check_before_sync(clock, config, &probe) == GateVerdict::Reject {
        return None;
    }
    let mut tick = probe;
    wire::sign_tick(secret, &mut tick);
    Some(tick)
}

/// Apply an inbound tick only after the thermodynamic gate admits it.
pub fn apply_gated_inbound(
    clock: &mut crate::clock::LocalClock,
    ledger: &mut CreditLedger,
    config: &AgentConfig,
    tick: &ClockTick,
    secret: &[u8],
) -> GatedSyncOutcome {
    if gate_check_before_sync(clock, config, tick) == GateVerdict::Reject {
        return GatedSyncOutcome::RejectedByGate;
    }
    let outcome = wire::apply_inbound_clock_tick(clock, ledger, config, tick, secret);
    GatedSyncOutcome::Admitted(outcome)
}

/// Well-known TCP ports for a 3-peer localhost mesh demo.
pub const LOCALHOST_MESH_PORTS: [u16; 3] = [4001, 4002, 4003];

/// Bootstrap multiaddrs for a 3-peer localhost mesh (`peer_index` ∈ 0..3).
#[must_use]
pub fn localhost_mesh_bootstrap(peer_index: usize) -> Vec<String> {
    LOCALHOST_MESH_PORTS
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != peer_index)
        .map(|(i, port)| format!("/ip4/127.0.0.1/tcp/{port}/p2p/peer-{i}"))
        .collect()
}

/// Helper: spawn configuration for peer `idx` in the 3-peer localhost test mesh.
#[must_use]
pub fn localhost_peer_config(idx: usize, budget_bits: f64) -> AgentConfig {
    assert!(idx < 3, "localhost mesh supports exactly 3 peers");
    AgentConfig {
        peer_id: (idx + 1) as PeerId,
        drift_ppb: 10.0,
        temperature_k: 300.0,
        budget_bits,
        sync_interval_sec: 5.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::LocalClock;
    use crate::credit::CreditLedger;

    #[test]
    fn three_peer_localhost_bootstrap_excludes_self() {
        let b0 = localhost_mesh_bootstrap(0);
        assert_eq!(b0.len(), 2);
        assert!(b0.iter().all(|a| a.contains("4002") || a.contains("4003")));
    }

    #[test]
    fn gate_rejects_over_budget_tick() {
        let clock = LocalClock::new(10.0, 300.0);
        let config = AgentConfig {
            budget_bits: 2.0,
            ..AgentConfig::default()
        };
        let tick = ClockTick {
            agent_id: 2,
            phase_entropy_bits: 50.0,
            landauer_cost_j: 0.0,
            accuracy_score: 0.9,
            sig: [0; 32],
        };
        assert_eq!(
            gate_check_before_sync(&clock, &config, &tick),
            GateVerdict::Reject
        );
    }

    #[test]
    fn gated_inbound_respects_gate() {
        let mut clock = LocalClock::new(10.0, 300.0);
        let mut ledger = CreditLedger::new(1, 300.0);
        let config = AgentConfig {
            peer_id: 1,
            budget_bits: 2.0,
            ..AgentConfig::default()
        };
        let mut tick = ClockTick {
            agent_id: 2,
            phase_entropy_bits: 50.0,
            landauer_cost_j: 0.0,
            accuracy_score: 0.9,
            sig: [0; 32],
        };
        wire::sign_tick(b"test", &mut tick);
        assert_eq!(
            apply_gated_inbound(&mut clock, &mut ledger, &config, &tick, b"test"),
            GatedSyncOutcome::RejectedByGate
        );
    }

    #[test]
    fn gossip_outbound_inbound_cycle_respects_gate() {
        let mut clock_out = LocalClock::new(10.0, 300.0);
        clock_out.phase_uncertainty_sec = 4e-9;
        let mut clock_in = LocalClock::new(10.0, 300.0);
        clock_in.phase_uncertainty_sec = 1e-6;
        let mut ledger = CreditLedger::new(2, 300.0);
        ledger.add_peer(1, 5.0);
        let publisher = localhost_peer_config(0, 20.0);
        let receiver = localhost_peer_config(1, 20.0);
        let secret = b"e64-gossip-cycle";
        let tick = outbound_tick_if_admitted(&clock_out, &publisher, secret);
        assert!(tick.is_some(), "outbound tick admitted under budget");
        let tick = tick.unwrap();
        assert_eq!(tick.agent_id, 1);
        let outcome = apply_gated_inbound(&mut clock_in, &mut ledger, &receiver, &tick, secret);
        assert_eq!(outcome, GatedSyncOutcome::Admitted(MergeOutcome::Accepted));
    }

    #[test]
    fn w8e14_outbound_tick_none_when_low_entropy() {
        let clock = LocalClock::new(10.0, 300.0);
        let config = localhost_peer_config(0, 20.0);
        assert!(outbound_tick_if_admitted(&clock, &config, b"low-entropy").is_none());
    }
}

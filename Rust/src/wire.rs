// SPDX-License-Identifier: MIT
// Wire format for gossip P2P: JSON payload + 32-byte keyed digest `sig` (no libp2p in this module).

//! **Schema:** `ClockTick` JSON fields `agent_id`, `phase_entropy_bits`, `landauer_cost_j`,
//! `accuracy_score`, `sig` (hex-ready byte array serialized as JSON array of u8 in tests; over
//! the wire the daemon uses raw `serde_json` of the struct with sig as `[u8;32]`).

use crate::clock::LocalClock;
use crate::credit::{CreditLedger, PeerId};
use crate::gate::{self, ClockThermState, GateVerdict};
use crate::landauer;
use crate::AgentConfig;

/// One gossip-published clock observation from a remote agent.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClockTick {
    pub agent_id: PeerId,
    pub phase_entropy_bits: f64,
    pub landauer_cost_j: f64,
    pub accuracy_score: f64,
    pub sig: [u8; 32],
}

/// Result of applying a remote tick into the local agent (after signature check + gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeOutcome {
    Accepted,
    RejectedBadSig,
    RejectedGate,
    RejectedSelf,
}

fn signing_payload(t: &ClockTick) -> Vec<u8> {
    let mut t2 = t.clone();
    t2.sig = [0; 32];
    serde_json::to_vec(&t2).expect("ClockTick serializes")
}

/// Std-only keyed 32-byte digest (development / lab use — replace with HMAC in hardened deploys).
pub fn sign_tick(secret: &[u8], tick: &mut ClockTick) {
    let p = signing_payload(tick);
    tick.sig = mix_digest(secret, &p);
}

#[must_use]
pub fn verify_tick(secret: &[u8], tick: &ClockTick) -> bool {
    mix_digest(secret, &signing_payload(tick)) == tick.sig
}

fn mix_digest(secret: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut acc: u64 = 0xcbf29ce484222325;
    for &b in secret {
        acc = acc.wrapping_mul(0x100000001b3).wrapping_add(b as u64);
    }
    for (i, &b) in msg.iter().enumerate() {
        acc = acc.wrapping_mul(0x100000001b3).wrapping_add(b as u64);
        out[i % 32] ^= (acc as u8).wrapping_add(i as u8);
    }
    for j in 0..32 {
        out[j] ^= secret[j % secret.len().max(1)];
    }
    out
}

/// Validate signature, run [`gate::gate_check`] on proposed bits, then credit [`CreditLedger::record_sync`].
pub fn apply_inbound_clock_tick(
    clock: &mut LocalClock,
    ledger: &mut CreditLedger,
    config: &AgentConfig,
    tick: &ClockTick,
    secret: &[u8],
) -> MergeOutcome {
    if tick.agent_id == config.peer_id {
        return MergeOutcome::RejectedSelf;
    }
    if !verify_tick(secret, tick) {
        return MergeOutcome::RejectedBadSig;
    }
    let therm = ClockThermState {
        desync_energy_j: clock.desync_energy_joules(),
        budget_j: landauer::landauer_cost(config.budget_bits, config.temperature_k),
        temperature_k: config.temperature_k,
        total_sync_cost_j: 0.0,
    };
    let bits = tick
        .phase_entropy_bits
        .clamp(0.1_f64, config.budget_bits.max(0.1));
    if gate::gate_check(&therm, bits) == GateVerdict::Reject {
        return MergeOutcome::RejectedGate;
    }
    if !ledger.peers.contains_key(&tick.agent_id) {
        ledger.add_peer(tick.agent_id, 10.0);
    }
    let improved = tick.accuracy_score >= 0.5;
    ledger.record_sync(tick.agent_id, bits, improved);
    MergeOutcome::Accepted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::LocalClock;
    use crate::credit::CreditLedger;

    #[test]
    fn clock_tick_json_roundtrip() {
        let mut t = ClockTick {
            agent_id: 7,
            phase_entropy_bits: 2.5,
            landauer_cost_j: 1e-12,
            accuracy_score: 0.9,
            sig: [0; 32],
        };
        sign_tick(b"test-secret", &mut t);
        let json = serde_json::to_string(&t).unwrap();
        let back: ClockTick = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
        assert!(verify_tick(b"test-secret", &back));
    }

    #[test]
    fn gate_check_synthetic_remote_batch() {
        let secret = b"batch";
        let t = 300.0_f64;
        let budget_j = landauer::landauer_cost(10.0, t);
        let mut ticks = vec![
            ClockTick {
                agent_id: 2,
                phase_entropy_bits: 3.0,
                landauer_cost_j: 1e-15,
                accuracy_score: 0.95,
                sig: [0; 32],
            },
            ClockTick {
                agent_id: 3,
                phase_entropy_bits: 50.0,
                landauer_cost_j: 1e-10,
                accuracy_score: 0.2,
                sig: [0; 32],
            },
        ];
        for tick in &mut ticks {
            sign_tick(secret, tick);
        }
        let therm = ClockThermState {
            desync_energy_j: landauer::desync_energy(8.0, t),
            budget_j,
            temperature_k: t,
            total_sync_cost_j: 0.0,
        };
        let v0 = gate::gate_check(&therm, ticks[0].phase_entropy_bits.clamp(0.1, 10.0));
        let v1 = gate::gate_check(&therm, ticks[1].phase_entropy_bits.clamp(0.1, 10.0));
        assert_eq!(v0, GateVerdict::Admit);
        assert_eq!(v1, GateVerdict::Reject);
    }

    #[test]
    fn apply_inbound_updates_ledger() {
        let mut clock = LocalClock::new(10.0, 300.0);
        clock.phase_uncertainty_sec = 1e-6;
        let mut ledger = CreditLedger::new(1, 300.0);
        let config = AgentConfig {
            peer_id: 1,
            drift_ppb: 10.0,
            temperature_k: 300.0,
            budget_bits: 20.0,
            sync_interval_sec: 60.0,
        };
        let mut tick = ClockTick {
            agent_id: 2,
            phase_entropy_bits: 2.0,
            landauer_cost_j: 0.0,
            accuracy_score: 0.99,
            sig: [0; 32],
        };
        sign_tick(b"k", &mut tick);
        let o = apply_inbound_clock_tick(&mut clock, &mut ledger, &config, &tick, b"k");
        assert_eq!(o, MergeOutcome::Accepted);
        assert!(ledger.peers.contains_key(&2));
    }
}

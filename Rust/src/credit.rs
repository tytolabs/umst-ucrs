// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Thermodynamic Credit Ledger.
//!
//! The credit system ensures least thermodynamic cost in a multi-agent
//! network by making synchronization accuracy tradeable.
//!
//! Core invariant (to be Lean-verified as `credit_greedy_optimal`):
//!   Greedy selection of highest-credit peer minimizes total network
//!   Landauer expenditure for a given accuracy target.
//!
//! The credit system also enables Byzantine detection for free:
//!   A lying peer causes recipients' drift to increase → their credit
//!   drops → the network naturally isolates them.

use crate::landauer;
use std::collections::HashMap;
use std::time::Instant;

/// Unique identifier for a peer in the P2P network.
pub type PeerId = u64;

/// Per-peer credit record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeerCredit {
    pub peer_id: PeerId,
    /// Accumulated credit in bits (information provided to network).
    pub credit_bits: f64,
    /// Estimated drift rate of this peer (ppb). Lower = more accurate.
    pub drift_estimate_ppb: f64,
    /// When we last synced with this peer.
    #[serde(skip)]
    pub last_sync: Option<Instant>,
    /// Total number of sync interactions with this peer.
    pub sync_count: u64,
    /// Cumulative bits resolved from this peer's corrections.
    pub bits_received: f64,
    /// Rolling accuracy score: fraction of syncs that improved our phase.
    pub accuracy_score: f64,
}

impl PeerCredit {
    pub fn new(peer_id: PeerId, drift_estimate_ppb: f64) -> Self {
        Self {
            peer_id,
            credit_bits: 0.0,
            drift_estimate_ppb,
            last_sync: None,
            sync_count: 0,
            bits_received: 0.0,
            accuracy_score: 1.0, // start with full trust
        }
    }
}

/// The credit ledger for the local agent.
#[derive(Debug)]
pub struct CreditLedger {
    /// Our own peer ID.
    pub self_id: PeerId,
    /// Per-peer credit records.
    pub peers: HashMap<PeerId, PeerCredit>,
    /// Temperature at our compute node (Kelvin).
    pub temperature_k: f64,
}

/// Result of a sync decision.
#[derive(Debug, Clone)]
pub struct SyncDecision {
    pub peer_id: PeerId,
    pub bits_to_resolve: f64,
    pub expected_cost_joules: f64,
    pub peer_credit_before: f64,
}

impl CreditLedger {
    pub fn new(self_id: PeerId, temperature_k: f64) -> Self {
        Self {
            self_id,
            peers: HashMap::new(),
            temperature_k,
        }
    }

    /// Register a new peer.
    pub fn add_peer(&mut self, peer_id: PeerId, drift_ppb: f64) {
        self.peers
            .insert(peer_id, PeerCredit::new(peer_id, drift_ppb));
    }

    /// Select the optimal sync peer (highest credit = lowest expected cost).
    ///
    /// This is the greedy selection that the `credit_greedy_optimal`
    /// theorem proves is Landauer-optimal.
    ///
    /// Intuition: highest-credit peer has provided the most accurate
    /// corrections historically, so H(our_phase | peer) is minimized,
    /// hence Landauer cost k_B T ln(2) · H(our_phase | peer) is minimized.
    pub fn best_peer(&self) -> Option<SyncDecision> {
        self.peers
            .values()
            .filter(|p| p.accuracy_score > 0.1) // exclude degraded peers
            .max_by(|a, b| {
                a.credit_bits
                    .partial_cmp(&b.credit_bits)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| {
                // Estimate bits to resolve based on peer's drift
                // Lower drift peer → lower conditional entropy → fewer bits
                let conditional_entropy = (p.drift_estimate_ppb / 0.1).log2().max(0.0);
                SyncDecision {
                    peer_id: p.peer_id,
                    bits_to_resolve: conditional_entropy,
                    expected_cost_joules: landauer::landauer_cost(
                        conditional_entropy,
                        self.temperature_k,
                    ),
                    peer_credit_before: p.credit_bits,
                }
            })
    }

    /// Record a completed sync event.
    ///
    /// Credit transfer: peer gains `bits_resolved` credit (they provided
    /// useful information). This is the thermodynamic accounting that
    /// ensures the network minimizes total Landauer expenditure.
    ///
    /// If the sync made our phase worse (Byzantine peer), their accuracy
    /// score drops — and with it, their effective credit for future selection.
    pub fn record_sync(&mut self, peer_id: PeerId, bits_resolved: f64, sync_improved_phase: bool) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.sync_count += 1;
            peer.last_sync = Some(Instant::now());

            if sync_improved_phase {
                // Peer provided useful correction → credit increases
                peer.credit_bits += bits_resolved;
                peer.bits_received += bits_resolved;
                // Exponential moving average of accuracy
                peer.accuracy_score = 0.9 * peer.accuracy_score + 0.1;
            } else {
                // Sync made things worse → Byzantine signal
                peer.credit_bits -= bits_resolved * 2.0; // penalty > reward
                                                         // Accuracy degrades
                peer.accuracy_score *= 0.9;
                if peer.sync_count > 5 && peer.accuracy_score < 0.5 {
                    crate::telemetry::record_byzantine_detection();
                }
            }
        }
    }

    /// Total Landauer cost of all sync operations (joules).
    pub fn total_network_cost_joules(&self) -> f64 {
        self.peers
            .values()
            .map(|p| landauer::landauer_cost(p.bits_received, self.temperature_k))
            .sum()
    }

    /// Total credit in the network (should be ~conserved in a healthy network).
    pub fn total_credit(&self) -> f64 {
        self.peers.values().map(|p| p.credit_bits).sum()
    }

    /// Identify potentially Byzantine peers (accuracy below threshold).
    pub fn suspect_peers(&self, threshold: f64) -> Vec<PeerId> {
        self.peers
            .values()
            .filter(|p| p.accuracy_score < threshold && p.sync_count > 5)
            .map(|p| p.peer_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_peer_selects_highest_credit() {
        let mut ledger = CreditLedger::new(0, 300.0);
        ledger.add_peer(1, 5.0);
        ledger.add_peer(2, 50.0);

        // Give peer 1 more credit
        ledger.record_sync(1, 5.0, true);
        ledger.record_sync(1, 5.0, true);
        ledger.record_sync(2, 1.0, true);

        let decision = ledger.best_peer().unwrap();
        assert_eq!(decision.peer_id, 1, "Should select peer 1 (higher credit)");
    }

    #[test]
    fn byzantine_peer_credit_drops() {
        let mut ledger = CreditLedger::new(0, 300.0);
        ledger.add_peer(1, 5.0);

        // Give initial credit
        ledger.record_sync(1, 5.0, true);
        let credit_after_good = ledger.peers[&1].credit_bits;

        // Byzantine sync (made things worse)
        ledger.record_sync(1, 3.0, false);
        let credit_after_bad = ledger.peers[&1].credit_bits;

        assert!(
            credit_after_bad < credit_after_good,
            "Credit should decrease after bad sync: {credit_after_bad} < {credit_after_good}"
        );
    }

    #[test]
    fn byzantine_detection() {
        let mut ledger = CreditLedger::new(0, 300.0);
        ledger.add_peer(1, 5.0); // honest
        ledger.add_peer(2, 5.0); // byzantine

        // Honest peer: all good syncs
        for _ in 0..10 {
            ledger.record_sync(1, 3.0, true);
        }

        // Byzantine peer: all bad syncs
        for _ in 0..10 {
            ledger.record_sync(2, 3.0, false);
        }

        let suspects = ledger.suspect_peers(0.5);
        assert!(suspects.contains(&2), "Peer 2 should be flagged as suspect");
        assert!(!suspects.contains(&1), "Peer 1 should NOT be flagged");
    }

    #[test]
    fn total_cost_tracks_landauer() {
        let mut ledger = CreditLedger::new(0, 300.0);
        ledger.add_peer(1, 5.0);
        ledger.record_sync(1, 10.0, true);

        let cost = ledger.total_network_cost_joules();
        let expected = landauer::landauer_cost(10.0, 300.0);
        assert!((cost - expected).abs() < f64::EPSILON);
    }
}

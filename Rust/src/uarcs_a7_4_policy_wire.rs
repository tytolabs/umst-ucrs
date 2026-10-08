// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! FLEET-COMPOSER ACCEL-C AC81 — UARCS-A7-4 policy wire UCRS owner deepen.
//!
//! UCRS owns **when + provenance** observation stamps and Landauer pairwise SSOT that
//! would eventually attest A7-4 policy admissibility. Does **not** flip `policy_wired`,
//! `p6_semantic_bind_wired`, `a8_channel_wired`, or `production_wired` — those stay **false**
//! unless measured on live ceremony surfaces (arcs P6 bind, A8-WIRE channel, gateway consumer).
//!
//! Absorbs Y71 bench census (`uarcs_a7_4_policy_wire_census`) and Z47 arcs semantic-bind
//! residue without depending on `umst-arcs`, `umst-bench`, or `umst-gateway`.

use crate::accept::{TrustCipherSuite, DURABLE_ACCEPT_SCHEMA_VERSION};
use crate::landauer_adopt::{
    landauer_ucrs_pairwise_adopt_closed, landauer_ucrs_pairwise_mi_entropy_bridge_wired,
    landauer_ucrs_pairwise_symbols_wired, A7_LANE_ID,
};
use crate::observation::{StampTier, TemporalWitness, WIRE_SCALE};
use crate::AgentConfig;
use serde::{Deserialize, Serialize};

/// Bench Y71 census authority (source-reviewed; no crate dep).
pub const BENCH_Y71_AUTHORITY: &str = "crates/umst-bench/src/uarcs_a7_4_policy_wire_census.rs";

/// Arcs Z47 semantic-bind authority (source-reviewed; no crate dep).
pub const ARCS_Z47_AUTHORITY: &str = "umst-arcs/crates/umst-arcs/src/uarcs_a7_4_semantic_bind.rs";

/// Arcs P6 coordination authority (source-reviewed; no crate dep).
pub const ARCS_P6_AUTHORITY: &str = "umst-arcs/crates/umst-arcs/src/coordination_cost_p6.rs";

/// Indexed P6 obligations still OPEN on arcs owner @ Y71 census.
pub const P6_OPEN_OBLIGATION_COUNT: usize = 4;

/// UCRS-side wire hop count (Landauer prep + observation + policy + cross-lane refs).
pub const WIRE_HOP_COUNT: usize = 5;

/// Honest closed hops @ default build — pairwise SSOT + observation/accept prep (H1–H3).
pub const WIRE_HOPS_CLOSED_DEFAULT: u8 = 3;

/// Honest adoption tier for this module.
pub const POSTURE_TAG: &str = "honest-policy-wire-prep";

/// One UCRS-side hop in the UARCS-A7-4 policy wire ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UarcsA74PolicyWireHop {
    /// Ordinal (1-based).
    pub hop: u8,
    /// Stable wire id.
    pub wire_id: &'static str,
    /// Owning surface.
    pub surface: &'static str,
    /// Posture label.
    pub status: &'static str,
    /// Whether hop is wired on the default (no-feature) build.
    pub wired_default: bool,
}

/// UCRS lane wire inventory for UARCS-A7-4 policy wire.
pub const UARCS_A7_4_POLICY_WIRE_HOPS: [UarcsA74PolicyWireHop; WIRE_HOP_COUNT] = [
    UarcsA74PolicyWireHop {
        hop: 1,
        wire_id: "landauer_pairwise_ssot",
        surface: "landauer_adopt::landauer_ucrs_pairwise_symbols_wired",
        status: "LANDED",
        wired_default: true,
    },
    UarcsA74PolicyWireHop {
        hop: 2,
        wire_id: "pairwise_mi_entropy_bridge",
        surface: "landauer_adopt::landauer_ucrs_pairwise_mi_entropy_bridge_wired",
        status: "LANDED",
        wired_default: true,
    },
    UarcsA74PolicyWireHop {
        hop: 3,
        wire_id: "ucrs_observation_accept_prep",
        surface: "observation::TemporalWitness + accept::DurableAccept",
        status: "LANDED",
        wired_default: true,
    },
    UarcsA74PolicyWireHop {
        hop: 4,
        wire_id: "a7_4_policy_wired",
        surface: "uarcs_a7_4_policy_wire::uarcs_a7_4_policy_wired",
        status: "OPEN",
        wired_default: false,
    },
    UarcsA74PolicyWireHop {
        hop: 5,
        wire_id: "arcs_p6_semantic_bind_cross_lane",
        surface: "umst-arcs::coordination_cost_p6::p6_semantic_bind_wired",
        status: "CROSS_LANE_OPEN",
        wired_default: false,
    },
];

/// Open hop surfaces blocking full UARCS-A7-4 policy master retick.
pub const UARCS_A7_4_POLICY_WIRE_OPEN_HOP_SURFACES: [&str; 3] = [
    "uarcs_a7_4_policy_wire::uarcs_a7_4_policy_wired",
    "umst-arcs::coordination_cost_p6::p6_semantic_bind_wired",
    "umst-arcs::informational_channel::informational_channel_wired",
];

/// A7-4 policy wire on UCRS lane — stays **false** until measured ceremony.
#[must_use]
pub const fn uarcs_a7_4_policy_wired() -> bool {
    false
}

/// P6 semantic bind mirror — stays **false** until arcs A10 ceremony.
#[must_use]
pub const fn uarcs_a7_4_p6_semantic_bind_wired() -> bool {
    false
}

/// A8 informational channel wire mirror — stays **false** on default build.
#[must_use]
pub const fn uarcs_a7_4_a8_channel_wired() -> bool {
    false
}

/// Production A7-4 policy wire — stays **false** until live measured flip.
#[must_use]
pub const fn uarcs_a7_4_production_wired() -> bool {
    false
}

/// P6 bind attempt surface mirror — prep compiled on arcs owner (SWARM-C25-0831-36).
#[must_use]
pub const fn uarcs_a7_4_p6_bind_attempt_available() -> bool {
    true
}

/// Count wired hops on the default (no-feature) build.
#[must_use]
pub fn uarcs_a7_4_policy_wire_hops_closed_count() -> u8 {
    UARCS_A7_4_POLICY_WIRE_HOPS
        .iter()
        .filter(|h| h.wired_default)
        .count() as u8
}

/// Whether UCRS wire hop census pins are locked @ default build.
#[must_use]
pub fn uarcs_a7_4_policy_wire_hops_honest() -> bool {
    uarcs_a7_4_policy_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
        && UARCS_A7_4_POLICY_WIRE_HOPS.len() == WIRE_HOP_COUNT
        && UARCS_A7_4_POLICY_WIRE_HOPS[0].wired_default
        && UARCS_A7_4_POLICY_WIRE_HOPS[1].wired_default
        && UARCS_A7_4_POLICY_WIRE_HOPS[2].wired_default
        && !UARCS_A7_4_POLICY_WIRE_HOPS[3].wired_default
        && !UARCS_A7_4_POLICY_WIRE_HOPS[4].wired_default
}

/// Whether UCRS Landauer pairwise + observation/accept prep symbols reduce for A7-4 policy.
#[must_use]
pub fn uarcs_a7_4_ucrs_policy_prep_wired() -> bool {
    let config = AgentConfig::default();
    let mut witness = TemporalWitness::from_agent(&config);
    let stamp = witness.stamp();
    landauer_ucrs_pairwise_symbols_wired()
        && landauer_ucrs_pairwise_mi_entropy_bridge_wired()
        && stamp.stamp_tier == StampTier::UcrsTier2
        && stamp.phase_entropy_bits_scale.unwrap_or(0) == WIRE_SCALE
        && TrustCipherSuite::nist_pqc_balanced_3()
            .kem
            .contains("ml-kem")
        && DURABLE_ACCEPT_SCHEMA_VERSION == "durable_accept.v0"
}

/// Whether UCRS A7-4 policy prep path is closed (default build).
///
/// True when H1–H3 spine is wired and prep symbols reduce — does **not** claim
/// `policy_wired`, `p6_semantic_bind_wired`, `a8_channel_wired`, or `production_wired`.
#[must_use]
pub fn uarcs_a7_4_ucrs_policy_prep_closed() -> bool {
    uarcs_a7_4_ucrs_policy_prep_wired()
        && landauer_ucrs_pairwise_adopt_closed()
        && uarcs_a7_4_policy_wire_hops_honest()
        && uarcs_a7_4_policy_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
}

/// AC81 adoption probe — UCRS owner lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UarcsA74PolicyWireAc81Probe {
    /// A7 lane id cross-ref.
    pub a7_lane_id: &'static str,
    /// UCRS wire hops closed on default build.
    pub wire_hops_closed: u8,
    /// UCRS Landauer + observation prep wired.
    pub ucrs_prep_wired: bool,
    /// A7-4 policy wire on UCRS lane.
    pub policy_wired: bool,
    /// P6 semantic bind mirror.
    pub p6_semantic_bind_wired: bool,
    /// A8 channel wire mirror.
    pub a8_channel_wired: bool,
    /// Production wire mirror.
    pub production_wired: bool,
    /// P6 bind attempt available.
    pub p6_bind_attempt_available: bool,
}

/// Emit AC81 UARCS-A7-4 policy wire probe snapshot.
#[must_use]
pub fn uarcs_a7_4_policy_wire_ac81_probe() -> UarcsA74PolicyWireAc81Probe {
    UarcsA74PolicyWireAc81Probe {
        a7_lane_id: A7_LANE_ID,
        wire_hops_closed: uarcs_a7_4_policy_wire_hops_closed_count(),
        ucrs_prep_wired: uarcs_a7_4_ucrs_policy_prep_wired(),
        policy_wired: uarcs_a7_4_policy_wired(),
        p6_semantic_bind_wired: uarcs_a7_4_p6_semantic_bind_wired(),
        a8_channel_wired: uarcs_a7_4_a8_channel_wired(),
        production_wired: uarcs_a7_4_production_wired(),
        p6_bind_attempt_available: uarcs_a7_4_p6_bind_attempt_available(),
    }
}

/// AC81 honest residue — UCRS prep landed; policy/p6/production false; cross-lanes open.
#[must_use]
pub fn uarcs_a7_4_policy_wire_ac81_residue_honest() -> bool {
    let probe = uarcs_a7_4_policy_wire_ac81_probe();
        probe.a7_lane_id == "A7-4"
        && probe.wire_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.ucrs_prep_wired
        && uarcs_a7_4_policy_wire_hops_honest()
        && uarcs_a7_4_ucrs_policy_prep_closed()
        && probe.p6_bind_attempt_available
        && !probe.policy_wired
        && !probe.p6_semantic_bind_wired
        && !probe.a8_channel_wired
        && !probe.production_wired
}

/// Whether UARCS-A7-4 policy wire adoption is honest — no fake production flip.
#[must_use]
pub fn uarcs_a7_4_policy_wire_honest() -> bool {
    uarcs_a7_4_policy_wire_ac81_residue_honest()
        && !uarcs_a7_4_policy_wired()
        && !uarcs_a7_4_p6_semantic_bind_wired()
        && !uarcs_a7_4_a8_channel_wired()
        && !uarcs_a7_4_production_wired()
}

/// One-line operator summary for AC81 receipts.
#[must_use]
pub fn uarcs_a7_4_policy_wire_ac81_summary() -> String {
    let probe = uarcs_a7_4_policy_wire_ac81_probe();
    format!(
        "AC81 UARCS-A7-4 policy-wire: hops_closed={}/{} ucrs_prep={} policy_wired={} p6_bind={} production_wired={}",
        probe.wire_hops_closed,
        WIRE_HOP_COUNT,
        probe.ucrs_prep_wired,
        probe.policy_wired,
        probe.p6_semantic_bind_wired,
        probe.production_wired
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uarcs_a7_4_metadata_constants() {
        assert_eq!(UARCS_A7_4_POLICY_WIRE_HOPS.len(), 5);
        assert_eq!(uarcs_a7_4_policy_wire_hops_closed_count(), 3);
    }

    #[test]
    fn uarcs_a7_4_wire_hops_three_of_five_closed_default() {
        assert!(uarcs_a7_4_policy_wire_hops_honest());
        assert_eq!(uarcs_a7_4_policy_wire_hops_closed_count(), 3);
        assert!(UARCS_A7_4_POLICY_WIRE_HOPS[0].wired_default);
        assert!(UARCS_A7_4_POLICY_WIRE_HOPS[1].wired_default);
        assert!(UARCS_A7_4_POLICY_WIRE_HOPS[2].wired_default);
        assert!(!UARCS_A7_4_POLICY_WIRE_HOPS[3].wired_default);
        assert!(!UARCS_A7_4_POLICY_WIRE_HOPS[4].wired_default);
    }

    #[test]
    fn uarcs_a7_4_policy_p6_production_false_honest() {
        assert!(!uarcs_a7_4_policy_wired());
        assert!(!uarcs_a7_4_p6_semantic_bind_wired());
        assert!(!uarcs_a7_4_a8_channel_wired());
        assert!(!uarcs_a7_4_production_wired());
        assert!(uarcs_a7_4_p6_bind_attempt_available());
        let probe = uarcs_a7_4_policy_wire_ac81_probe();
        assert!(!probe.policy_wired);
        assert!(!probe.p6_semantic_bind_wired);
        assert!(!probe.a8_channel_wired);
        assert!(!probe.production_wired);
        assert!(probe.p6_bind_attempt_available);
    }

    #[test]
    fn uarcs_a7_4_ucrs_policy_prep_wired_on_default_build() {
        assert!(uarcs_a7_4_ucrs_policy_prep_wired());
        assert!(uarcs_a7_4_ucrs_policy_prep_closed());
    }


    #[test]
    fn uarcs_a7_4_policy_wire_residue_honest() {
        assert!(uarcs_a7_4_policy_wire_ac81_residue_honest());
        assert!(uarcs_a7_4_policy_wire_honest());
        let probe = uarcs_a7_4_policy_wire_ac81_probe();
        assert_eq!(probe.wire_hops_closed, 3);
        assert!(probe.ucrs_prep_wired);
        assert_eq!(probe.a7_lane_id, "A7-4");
        assert_eq!(
            UARCS_A7_4_POLICY_WIRE_OPEN_HOP_SURFACES[1],
            "umst-arcs::coordination_cost_p6::p6_semantic_bind_wired"
        );
        assert!(uarcs_a7_4_policy_wire_ac81_summary().contains("policy_wired=false"));
    }

    #[test]
    fn policy_wire_honest_fence() {
        assert!(uarcs_a7_4_policy_wire_honest());
        let probe = uarcs_a7_4_policy_wire_ac81_probe();
        assert!(!probe.policy_wired);
        assert_eq!(probe.a7_lane_id, "A7-4");
    }
}

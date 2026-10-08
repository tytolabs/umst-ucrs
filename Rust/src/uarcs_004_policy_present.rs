// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! OPERATOR ACCEL AC21 — UARCS-004 `present_wired` UCRS policy-present deepen.
//!
//! UCRS owns **when + provenance** observation stamps (`UcrsObservedAt`, `TemporalWitness`,
//! `DurableAccept`) that would eventually attest present-policy admissibility. Does **not**
//! flip `present_wired`, `policy_wired`, or `production_wired` — those stay **false** unless
//! measured on live ceremony surfaces (WEB-026, arcs org-refinement, gateway consumer).
//!
//! Absorbs Y72 bench census (`uarcs_004_present_wired_census`) and G69 arcs wire-attempt
//! context without depending on `umst-arcs` or `umst-gateway`.

use crate::accept::{TrustCipherSuite, DURABLE_ACCEPT_SCHEMA_VERSION};
use crate::observation::{StampTier, TemporalWitness, WIRE_SCALE};
use crate::AgentConfig;
use serde::{Deserialize, Serialize};

/// Arcs UARCS-004 present_wired authority (source-reviewed; no crate dep).
pub const ARCS_UARCS_004_AUTHORITY: &str =
    "umst-arcs/crates/umst-arcs/src/uarcs_004_present_wired.rs";

/// Bench Y72 census authority (source-reviewed; no crate dep).
pub const BENCH_Y72_AUTHORITY: &str = "crates/umst-bench/src/uarcs_004_present_wired_census.rs";

/// WEB-026 constitutional D1 owner (orthogonal flip).
pub const WEB_026_OWNER: &str = "WEB-026";

/// UCRS-side wire hop count (observation prep + cross-lane refs).
pub const WIRE_HOP_COUNT: usize = 5;

/// Honest closed hops @ default build — UCRS observation/accept prep spine (H1–H3).
pub const WIRE_HOPS_CLOSED_DEFAULT: u8 = 3;

/// Honest adoption tier for this module.
pub const POSTURE_TAG: &str = "honest-policy-present-prep";

/// One UCRS-side hop in the UARCS-004 present policy wire ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Uarcs004PolicyPresentWireHop {
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

/// UCRS lane wire inventory for UARCS-004 present policy.
pub const UARCS_004_POLICY_PRESENT_WIRE_HOPS: [Uarcs004PolicyPresentWireHop; WIRE_HOP_COUNT] = [
    Uarcs004PolicyPresentWireHop {
        hop: 1,
        wire_id: "ucrs_observed_at_tier2",
        surface: "observation::UcrsObservedAt",
        status: "LANDED",
        wired_default: true,
    },
    Uarcs004PolicyPresentWireHop {
        hop: 2,
        wire_id: "temporal_witness_from_agent",
        surface: "observation::TemporalWitness::from_agent",
        status: "LANDED",
        wired_default: true,
    },
    Uarcs004PolicyPresentWireHop {
        hop: 3,
        wire_id: "durable_accept_trust_warrant",
        surface: "accept::DurableAccept",
        status: "LANDED",
        wired_default: true,
    },
    Uarcs004PolicyPresentWireHop {
        hop: 4,
        wire_id: "present_policy_wired",
        surface: "uarcs_004_policy_present::uarcs_004_present_policy_wired",
        status: "OPEN",
        wired_default: false,
    },
    Uarcs004PolicyPresentWireHop {
        hop: 5,
        wire_id: "arcs_present_wired_cross_lane",
        surface: "umst-arcs::uarcs_004_present_wired::uarcs_004_present_wired_probe",
        status: "CROSS_LANE_OPEN",
        wired_default: false,
    },
];

/// Open hop surfaces blocking full UARCS-004 present master retick.
pub const UARCS_004_POLICY_PRESENT_OPEN_HOP_SURFACES: [&str; 2] = [
    "uarcs_004_policy_present::uarcs_004_present_policy_wired",
    "umst-web/src/informational_present.rs::informational_present_wired",
];

/// Constitutional present wire — stays **false** until WEB-026 measured ceremony.
#[must_use]
pub const fn uarcs_004_present_wired() -> bool {
    false
}

/// UARCS-004 org-refinement policy wire — stays **false** until arcs ceremony.
#[must_use]
pub const fn uarcs_004_policy_wired() -> bool {
    false
}

/// Production present wire — stays **false** until live measured flip.
#[must_use]
pub const fn uarcs_004_present_production_wired() -> bool {
    false
}

/// UARCS-004 present policy wire on UCRS lane — not flipped by this module.
#[must_use]
pub const fn uarcs_004_present_policy_wired() -> bool {
    false
}

/// Count wired hops on the default (no-feature) build.
#[must_use]
pub fn uarcs_004_policy_present_wire_hops_closed_count() -> u8 {
    UARCS_004_POLICY_PRESENT_WIRE_HOPS
        .iter()
        .filter(|h| h.wired_default)
        .count() as u8
}

/// Whether UCRS wire hop census pins are locked @ default build.
#[must_use]
pub fn uarcs_004_policy_present_wire_hops_honest() -> bool {
    uarcs_004_policy_present_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
        && UARCS_004_POLICY_PRESENT_WIRE_HOPS.len() == WIRE_HOP_COUNT
        && UARCS_004_POLICY_PRESENT_WIRE_HOPS[0].wired_default
        && UARCS_004_POLICY_PRESENT_WIRE_HOPS[1].wired_default
        && UARCS_004_POLICY_PRESENT_WIRE_HOPS[2].wired_default
        && !UARCS_004_POLICY_PRESENT_WIRE_HOPS[3].wired_default
        && !UARCS_004_POLICY_PRESENT_WIRE_HOPS[4].wired_default
}

/// Whether UCRS observation/accept prep symbols compile and reduce for present policy.
#[must_use]
pub fn uarcs_004_ucrs_present_prep_wired() -> bool {
    let config = AgentConfig::default();
    let mut witness = TemporalWitness::from_agent(&config);
    let stamp = witness.stamp();
    stamp.stamp_tier == StampTier::UcrsTier2
        && stamp.phase_entropy_bits_scale.unwrap_or(0) == WIRE_SCALE
        && TrustCipherSuite::nist_pqc_balanced_3()
            .kem
            .contains("ml-kem")
        && DURABLE_ACCEPT_SCHEMA_VERSION == "durable_accept.v0"
}

/// Whether UCRS present-policy prep path is closed (default build).
///
/// True when H1–H3 spine is wired and prep symbols reduce — does **not** claim
/// `present_wired`, `policy_wired`, or arcs/gateway cross-lane closure.
#[must_use]
pub fn uarcs_004_ucrs_present_prep_closed() -> bool {
    uarcs_004_ucrs_present_prep_wired()
        && uarcs_004_policy_present_wire_hops_honest()
        && uarcs_004_policy_present_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
}

/// AC21 adoption probe — UCRS owner lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Uarcs004PolicyPresentAc21Probe {
    /// UCRS wire hops closed on default build.
    pub wire_hops_closed: u8,
    /// UCRS observation/accept prep wired.
    pub ucrs_prep_wired: bool,
    /// Constitutional present wire.
    pub present_wired: bool,
    /// Org-refinement policy wire.
    pub policy_wired: bool,
    /// Production present wire.
    pub production_wired: bool,
    /// Present policy wire on UCRS lane.
    pub present_policy_wired: bool,
}

/// Emit AC21 UARCS-004 policy-present probe snapshot.
#[must_use]
pub fn uarcs_004_policy_present_ac21_probe() -> Uarcs004PolicyPresentAc21Probe {
    Uarcs004PolicyPresentAc21Probe {
        wire_hops_closed: uarcs_004_policy_present_wire_hops_closed_count(),
        ucrs_prep_wired: uarcs_004_ucrs_present_prep_wired(),
        present_wired: uarcs_004_present_wired(),
        policy_wired: uarcs_004_policy_wired(),
        production_wired: uarcs_004_present_production_wired(),
        present_policy_wired: uarcs_004_present_policy_wired(),
    }
}

/// AC21 honest residue — UCRS prep landed; present/policy/production false; cross-lanes open.
#[must_use]
pub fn uarcs_004_policy_present_ac21_residue_honest() -> bool {
    let probe = uarcs_004_policy_present_ac21_probe();
        probe.wire_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.ucrs_prep_wired
        && uarcs_004_policy_present_wire_hops_honest()
        && uarcs_004_ucrs_present_prep_closed()
        && !probe.present_wired
        && !probe.policy_wired
        && !probe.production_wired
        && !probe.present_policy_wired
}

/// Whether UARCS-004 policy-present adoption is honest — no fake production flip.
#[must_use]
pub fn uarcs_004_policy_present_honest() -> bool {
    uarcs_004_policy_present_ac21_residue_honest()
        && !uarcs_004_present_wired()
        && !uarcs_004_policy_wired()
        && !uarcs_004_present_production_wired()
}

/// One-line operator summary for AC21 receipts.
#[must_use]
pub fn uarcs_004_policy_present_ac21_summary() -> String {
    let probe = uarcs_004_policy_present_ac21_probe();
    format!(
        "AC21 UARCS-004 policy-present: hops_closed={}/{} ucrs_prep={} present_wired={} policy_wired={} production_wired={}",
        probe.wire_hops_closed,
        WIRE_HOP_COUNT,
        probe.ucrs_prep_wired,
        probe.present_wired,
        probe.policy_wired,
        probe.production_wired
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uarcs_004_metadata_constants() {
        assert_eq!(UARCS_004_POLICY_PRESENT_WIRE_HOPS.len(), 5);
        assert_eq!(uarcs_004_policy_present_wire_hops_closed_count(), 3);
    }

    #[test]
    fn uarcs_004_wire_hops_three_of_five_closed_default() {
        assert!(uarcs_004_policy_present_wire_hops_honest());
        assert_eq!(uarcs_004_policy_present_wire_hops_closed_count(), 3);
        assert!(UARCS_004_POLICY_PRESENT_WIRE_HOPS[0].wired_default);
        assert!(UARCS_004_POLICY_PRESENT_WIRE_HOPS[1].wired_default);
        assert!(UARCS_004_POLICY_PRESENT_WIRE_HOPS[2].wired_default);
        assert!(!UARCS_004_POLICY_PRESENT_WIRE_HOPS[3].wired_default);
        assert!(!UARCS_004_POLICY_PRESENT_WIRE_HOPS[4].wired_default);
    }

    #[test]
    fn uarcs_004_present_policy_production_false_honest() {
        assert!(!uarcs_004_present_wired());
        assert!(!uarcs_004_policy_wired());
        assert!(!uarcs_004_present_production_wired());
        assert!(!uarcs_004_present_policy_wired());
        let probe = uarcs_004_policy_present_ac21_probe();
        assert!(!probe.present_wired);
        assert!(!probe.policy_wired);
        assert!(!probe.production_wired);
        assert!(!probe.present_policy_wired);
    }

    #[test]
    fn uarcs_004_ucrs_present_prep_wired_on_default_build() {
        assert!(uarcs_004_ucrs_present_prep_wired());
        assert!(uarcs_004_ucrs_present_prep_closed());
    }

    #[test]
    fn uarcs_004_absorbed_prior_receipts_honest() {
        assert!(ARCS_UARCS_004_AUTHORITY.contains("uarcs_004_present_wired"));
    }

    #[test]
    fn uarcs_004_policy_present_residue_honest() {
        assert!(uarcs_004_policy_present_ac21_residue_honest());
        assert!(uarcs_004_policy_present_honest());
        let probe = uarcs_004_policy_present_ac21_probe();
        assert_eq!(probe.wire_hops_closed, 3);
        assert!(probe.ucrs_prep_wired);
        assert_eq!(
            UARCS_004_POLICY_PRESENT_OPEN_HOP_SURFACES[1],
            "umst-web/src/informational_present.rs::informational_present_wired"
        );
        assert!(uarcs_004_policy_present_ac21_summary().contains("present_wired=false"));
    }

    #[test]
    fn policy_present_honest_fence() {
        assert!(uarcs_004_policy_present_honest());
        let probe = uarcs_004_policy_present_ac21_probe();
        assert!(!probe.present_wired);
        assert_eq!(probe.wire_hops_closed, 3);
    }
}

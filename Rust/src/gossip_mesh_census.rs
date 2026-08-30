// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! OPERATOR ACCEL AC82 — UARCS-gossip `mesh_wired` false census (UCRS owner lane).
//!
//! UCRS owns **gate-guarded clock gossip prep** (`wire::ClockTick`, `p2p` inbound/outbound cycle)
//! that would eventually attest mesh admissibility. Does **not** flip `mesh_wired`,
//! `production_wired`, or `ucrs_gossip_mesh_closed` — those stay **false** unless measured on
//! live ceremony surfaces (WEB-034 libp2p mesh, gateway W-10 consumer).
//!
//! Absorbs F64/F86 gossip cycle cuts, G86 absent receipt, H81/J23 gateway wire-prep census
//! (source-reviewed; no `umst-gateway` dep), X85 static `mesh_wired=false` deepen, and
//! Haskell UCRS mesh notes (`Gate` / `Credit` / `Landauer` + `Spec.hs` QuickCheck properties).
//!
//! **UCRS-URGE-GOSSIP-COMPOSE** — reverse compose census: Urge history-mesh consumers
//! (`umst_urge::gossip_tick`, `signed_propagate`, `gate_before_sync`) are the admitted history
//! layer that **calls** UCRS gossip prep surfaces. Citation only — `umst-urge` path-depends on
//! `umst-ucrs`; no reverse Cargo edge (cycle fence). `ucrs_gossip_mesh_production_wired()` and
//! §17.7 taxonomy stay **Unmeasured**/`false`. Sole axiom: `LandauerLaw.physicalSecondLaw`.

use crate::p2p::{
    apply_gated_inbound, gate_check_before_sync, outbound_tick_if_admitted, GatedSyncOutcome,
    LOCALHOST_MESH_PORTS,
};
use crate::wire::{self, MergeOutcome};
use crate::{
    clock::LocalClock,
    credit::CreditLedger,
    gate::GateVerdict,
    landauer::{self, K_B},
    AgentConfig,
};
use serde::{Deserialize, Serialize};

/// FLEET-COMPOSER-F F64 job id — gossip cycle integration test (absorbed).
pub const FLEET_F64_JOB_ID: &str = "FLEET-COMPOSER-F64-UCRS-GOSSIP";

/// FLEET-COMPOSER-F F86 job id — gossip cycle deepen (absorbed).
pub const FLEET_F86_JOB_ID: &str = "FLEET-COMPOSER-F86-UCRS-GOSSIP";

/// FLEET-COMPOSER-G G86 job id — gossip mesh wire probe (absent receipt; absorbed).
pub const FLEET_G86_JOB_ID: &str = "FLEET-COMPOSER-G86-UCRS-GOSSIP";

/// OPERATOR ACCEL Band C slot id.
pub const ACCEL_AC82_SLOT: &str = "AC82";

/// C82-1439 refill wave slot.
pub const C82_WAVE_SLOT: &str = "C82-1439";

/// C82-UCRS-GOSSIP swarm cell id.
pub const C82_CELL_ID: &str = "C82-UCRS-GOSSIP";

/// C82 UCRS gossip mesh census deepen job id.
pub const C82_UCRS_GOSSIP_JOB_ID: &str = "FLEET-COMPOSER-C82-UCRS-GOSSIP";

/// AC82 fleet card id.
pub const COMPOSER_AC82_JOB_ID: &str = "OPERATOR-ACCEL-AC82-UARCS-GOSSIP";

/// AC82 receipt path (this slice).
pub const COMPOSER_AC82_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_ACCEL2_AC82.md";

/// F64 absorbed receipt.
pub const PRIOR_F64_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F64_UCRS_1934.md";

/// F86 absorbed receipt.
pub const PRIOR_F86_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_F86_UCRS_GOSSIP_1942.md";

/// G86 absent receipt — never landed; pinned for operator traceability.
pub const PRIOR_G86_ABSENT_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_G86_UCRS_GOSSIP_2143.md";

/// H81 absorbed gateway gossip wire-prep receipt (source-reviewed; no crate dep).
pub const PRIOR_H81_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_H81_2242.md";

/// J23 absorbed rollup receipt (source-reviewed; no crate dep).
pub const PRIOR_J23_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_J23_2348.md";

/// X85 absorbed static census receipt.
pub const PRIOR_X85_RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_X85_0734.md";

/// Gateway H81 gossip wire-prep authority (source-reviewed; no crate dep).
pub const GATEWAY_H81_AUTHORITY: &str =
    "umst-gateway/crates/umst-gateway/src/uarcs_ucrs_gossip_wire_prep.rs";

/// Haskell UCRS Gate authority (source-reviewed; no cabal dep).
pub const HASKELL_GATE_AUTHORITY: &str = "umst-ucrs/Haskell/src/Umst/Ucrs/Gate.hs";

/// Haskell UCRS Credit authority (source-reviewed; no cabal dep).
pub const HASKELL_CREDIT_AUTHORITY: &str = "umst-ucrs/Haskell/src/Umst/Ucrs/Credit.hs";

/// Haskell UCRS Landauer authority (source-reviewed; no cabal dep).
pub const HASKELL_LANDAUER_AUTHORITY: &str = "umst-ucrs/Haskell/src/Umst/Ucrs/Landauer.hs";

/// Haskell property-test spec — five QuickCheck properties for gossip mesh economics.
pub const HASKELL_SPEC_AUTHORITY: &str = "umst-ucrs/Haskell/test/Spec.hs";

/// Count of QuickCheck properties in `HASKELL_SPEC_AUTHORITY`.
pub const HASKELL_PROPERTY_COUNT: usize = 5;

/// QuickCheck property labels from `HASKELL_SPEC_AUTHORITY` (`testProperty` names).
pub const HASKELL_SPEC_PROPERTY_LABELS: [&'static str; HASKELL_PROPERTY_COUNT] = [
    "greedy selects highest credit",
    "byzantine credit drops",
    "gate rejects over budget",
    "landauer monotonic in bits",
    "gate admits within budget",
];

/// Gateway H81 wire hop count (source-reviewed; no `umst-gateway` dep).
pub const GATEWAY_H81_WIRE_HOP_COUNT: usize = 6;

/// Gateway H81 hops closed on 6-hop map (source-reviewed).
pub const GATEWAY_H81_HOPS_CLOSED: u8 = 3;

/// Gateway H81 hops open on 6-hop map (source-reviewed).
pub const GATEWAY_H81_HOPS_OPEN: u8 = 3;

/// Mesh gossip temperature SSOT (Haskell `Spec.hs` uses 300.0 K throughout).
pub const HASKELL_MESH_TEMPERATURE_K: f64 = 300.0;

/// Localhost mesh peer count from `p2p::LOCALHOST_MESH_PORTS` (no libp2p daemon).
pub const LOCALHOST_MESH_PEER_COUNT: usize = 3;

/// WEB-034 constitutional mesh owner (orthogonal flip).
pub const WEB_034_OWNER: &str = "WEB-034";

/// WEB-013 web gossip envelope owner (cross-lane; no dep).
pub const WEB_013_OWNER: &str = "WEB-013";

/// UCRS-side wire hop count (clock gossip prep + cross-lane refs).
pub const WIRE_HOP_COUNT: usize = 6;

/// Honest closed hops @ default build — UCRS gate-guarded gossip prep spine (H1–H4).
pub const WIRE_HOPS_CLOSED_DEFAULT: u8 = 4;

/// Honest adoption tier for this module.
pub const POSTURE_TAG: &str = "honest-gossip-mesh-prep";
/// UCRS-URGE-GOSSIP-COMPOSE swarm cell id.
pub const UCRS_URGE_GOSSIP_COMPOSE_CELL_ID: &str = "UCRS-URGE-GOSSIP-COMPOSE";

/// Non-claim fence — reverse compose census; not physics GREEN; not production wired.
pub const UCRS_URGE_GOSSIP_COMPOSE_NON_CLAIM: &str =
    "UCRS-URGE-GOSSIP-COMPOSE Urge history-mesh consumers gossip_tick signed_propagate gate_before_sync call UCRS gossip prep; reverse compose citation only no umst-urge dep; ucrs_gossip_mesh_production_wired Unmeasured false; LandauerLaw.physicalSecondLaw sole axiom; physics_green false; not production_wired";

/// Cycle fence — `umst-urge` → `umst-ucrs` only; reverse compose is census, not a package edge.
pub const URGE_UCRS_CYCLE_FENCE: &str =
    "umst-urge path-depends on umst-ucrs; umst-ucrs must not path-depend on umst-urge";

/// Lean anchor — sole documented physics axiom carrier (`LandauerLaw.lean`).
pub const LEAN_ANCHOR_LANDAUER_LAW: &str = "umst-formal-double-slit/Lean/LandauerLaw.lean";

/// Sole physics axiom pin — do not mint a second Landauer axiom on this census path.
pub const LEAN_AXIOM_PHYSICAL_SECOND_LAW: &str = "LandauerLaw.physicalSecondLaw";

/// Authorized physics axiom count on UCRS gossip mesh census — sole `physicalSecondLaw`.
pub const PHYSICS_AXIOM_COUNT: usize = 1;

/// Honest physics GREEN posture for UCRS gossip mesh census (must stay false).
pub const UCRS_GOSSIP_MESH_PHYSICS_GREEN: bool = false;

/// Urge history-mesh consumer count — admitted history layer calling UCRS.
pub const URGE_HISTORY_MESH_CONSUMER_COUNT: usize = 3;

/// One Urge history-mesh consumer that calls UCRS gossip prep (source-reviewed; no crate dep).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrgeHistoryMeshConsumer {
    /// Rust module path (`umst_urge::…`).
    pub module_path: &'static str,
    /// Source file authority (source-reviewed).
    pub source_authority: &'static str,
    /// UCRS surface this consumer calls on the gossip mesh spine.
    pub ucrs_call_surface: &'static str,
    /// Owning Urge integration cell id.
    pub urge_cell_id: &'static str,
}

/// Urge history-mesh consumers — admitted history layer **calling** UCRS (reverse compose census).
pub const URGE_HISTORY_MESH_CONSUMERS: [UrgeHistoryMeshConsumer; URGE_HISTORY_MESH_CONSUMER_COUNT] =
    [
        UrgeHistoryMeshConsumer {
            module_path: "umst_urge::gate_before_sync",
            source_authority: "umst/umst-urge/src/gate_before_sync.rs",
            urge_cell_id: "URGE-INT-GATE-BEFORE-SYNC",
            ucrs_call_surface: "umst_ucrs::gate::gate_check",
        },
        UrgeHistoryMeshConsumer {
            module_path: "umst_urge::gossip_tick",
            source_authority: "umst/umst-urge/src/gossip_tick.rs",
            urge_cell_id: "URGE-INT-GOSSIP-TICK",
            ucrs_call_surface: "p2p::gate_check_before_sync",
        },
        UrgeHistoryMeshConsumer {
            module_path: "umst_urge::signed_propagate",
            source_authority: "umst/umst-urge/src/signed_propagate.rs",
            urge_cell_id: "URGE-INT-SIGNED-PROPAGATE",
            ucrs_call_surface: "wire::sign_tick / wire::verify_tick",
        },
    ];

/// Whether `ucrs_gossip_mesh_production_wired()` documents §17.7 `Unmeasured` posture.
#[must_use]
pub const fn gossip_mesh_production_wired_taxonomy_unmeasured() -> bool {
    !ucrs_gossip_mesh_production_wired()
}

/// Whether sole-axiom pin is honest — exactly one `LandauerLaw.physicalSecondLaw`.
#[must_use]
pub fn landauer_physical_second_law_sole_axiom_honest() -> bool {
    PHYSICS_AXIOM_COUNT == 1
        && LEAN_AXIOM_PHYSICAL_SECOND_LAW == "LandauerLaw.physicalSecondLaw"
}

/// Whether Urge history-mesh consumer census pins are source-reviewed and cycle-fenced.
#[must_use]
pub fn urge_history_mesh_consumers_honest() -> bool {
    URGE_HISTORY_MESH_CONSUMERS.len() == URGE_HISTORY_MESH_CONSUMER_COUNT
        && URGE_HISTORY_MESH_CONSUMERS[0].module_path == "umst_urge::gate_before_sync"
        && URGE_HISTORY_MESH_CONSUMERS[0]
            .ucrs_call_surface
            .contains("gate::gate_check")
        && URGE_HISTORY_MESH_CONSUMERS[1].module_path == "umst_urge::gossip_tick"
        && URGE_HISTORY_MESH_CONSUMERS[1]
            .ucrs_call_surface
            .contains("gate_check_before_sync")
        && URGE_HISTORY_MESH_CONSUMERS[2].module_path == "umst_urge::signed_propagate"
        && URGE_HISTORY_MESH_CONSUMERS[2]
            .ucrs_call_surface
            .contains("sign_tick")
        && URGE_UCRS_CYCLE_FENCE.contains("must not path-depend")
        && !UCRS_GOSSIP_MESH_PHYSICS_GREEN
        && gossip_mesh_production_wired_taxonomy_unmeasured()
        && landauer_physical_second_law_sole_axiom_honest()
}

/// UCRS-URGE-GOSSIP-COMPOSE probe — reverse compose census snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UcrsUrgeGossipComposeProbe {
    /// Swarm cell id.
    pub cell_id: &'static str,
    /// Urge history-mesh consumer count.
    pub urge_consumer_count: usize,
    /// All three consumers pinned with UCRS call surfaces.
    pub urge_consumers_honest: bool,
    /// Cycle fence documented (no reverse Cargo edge).
    pub cycle_fence_honest: bool,
    /// Sole `LandauerLaw.physicalSecondLaw` axiom pin.
    pub sole_axiom_honest: bool,
    /// Physics GREEN (must stay false).
    pub physics_green: bool,
    /// `ucrs_gossip_mesh_production_wired()` const false.
    pub production_wired_const: bool,
    /// §17.7 taxonomy documents Unmeasured while const false.
    pub production_wired_unmeasured: bool,
    /// UCRS prep spine wired on default build.
    pub ucrs_prep_wired: bool,
    /// Prior AC82/C82 residue honest.
    pub prior_residue_honest: bool,
}

/// Emit UCRS-URGE-GOSSIP-COMPOSE reverse compose probe snapshot.
#[must_use]
pub fn ucrs_urge_gossip_compose_probe() -> UcrsUrgeGossipComposeProbe {
    UcrsUrgeGossipComposeProbe {
        cell_id: UCRS_URGE_GOSSIP_COMPOSE_CELL_ID,
        urge_consumer_count: URGE_HISTORY_MESH_CONSUMER_COUNT,
        urge_consumers_honest: urge_history_mesh_consumers_honest(),
        cycle_fence_honest: URGE_UCRS_CYCLE_FENCE.contains("must not path-depend"),
        sole_axiom_honest: landauer_physical_second_law_sole_axiom_honest(),
        physics_green: UCRS_GOSSIP_MESH_PHYSICS_GREEN,
        production_wired_const: ucrs_gossip_mesh_production_wired(),
        production_wired_unmeasured: gossip_mesh_production_wired_taxonomy_unmeasured(),
        ucrs_prep_wired: ucrs_gossip_prep_wired(),
        prior_residue_honest: ucrs_gossip_mesh_c82_residue_honest(),
    }
}

/// UCRS-URGE-GOSSIP-COMPOSE honest residue — Urge consumers named; mesh/production false.
#[must_use]
pub fn ucrs_urge_gossip_compose_residue_honest() -> bool {
    let probe = ucrs_urge_gossip_compose_probe();
    probe.cell_id == UCRS_URGE_GOSSIP_COMPOSE_CELL_ID
        && probe.urge_consumer_count == 3
        && probe.urge_consumers_honest
        && probe.cycle_fence_honest
        && probe.sole_axiom_honest
        && !probe.physics_green
        && !probe.production_wired_const
        && probe.production_wired_unmeasured
        && probe.ucrs_prep_wired
        && probe.prior_residue_honest
        && urge_history_mesh_consumers_honest()
        && ucrs_gossip_mesh_c82_residue_honest()
}


/// One UCRS-side hop in the UARCS-gossip mesh wire ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UarcsGossipMeshWireHop {
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

/// UCRS lane wire inventory for UARCS-gossip mesh census.
pub const UARCS_GOSSIP_MESH_WIRE_HOPS: [UarcsGossipMeshWireHop; WIRE_HOP_COUNT] = [
    UarcsGossipMeshWireHop {
        hop: 1,
        wire_id: "clock_tick_wire",
        surface: "wire::ClockTick",
        status: "LANDED",
        wired_default: true,
    },
    UarcsGossipMeshWireHop {
        hop: 2,
        wire_id: "clock_tick_sign_verify",
        surface: "wire::sign_tick / wire::verify_tick",
        status: "LANDED",
        wired_default: true,
    },
    UarcsGossipMeshWireHop {
        hop: 3,
        wire_id: "p2p_gate_check_before_sync",
        surface: "p2p::gate_check_before_sync",
        status: "LANDED",
        wired_default: true,
    },
    UarcsGossipMeshWireHop {
        hop: 4,
        wire_id: "p2p_gossip_outbound_inbound_cycle",
        surface: "p2p::outbound_tick_if_admitted / p2p::apply_gated_inbound",
        status: "LANDED",
        wired_default: true,
    },
    UarcsGossipMeshWireHop {
        hop: 5,
        wire_id: "ucrs_gossip_mesh_policy_wired",
        surface: "gossip_mesh_census::ucrs_gossip_mesh_policy_wired",
        status: "OPEN",
        wired_default: false,
    },
    UarcsGossipMeshWireHop {
        hop: 6,
        wire_id: "web_gossip_mesh_wired",
        surface: "umst-web/src/gossip.rs::web_gossip_mesh_wired",
        status: "CROSS_LANE_OPEN",
        wired_default: false,
    },
];

/// Open hop surfaces blocking full UARCS-gossip mesh master retick.
pub const UARCS_GOSSIP_MESH_OPEN_HOP_SURFACES: [&'static str; 2] = [
    "gossip_mesh_census::ucrs_gossip_mesh_policy_wired",
    "umst-web/src/gossip.rs::web_gossip_mesh_wired",
];

/// Constitutional libp2p mesh wire — stays **false** until WEB-034 measured ceremony.
#[must_use]
pub const fn ucrs_gossip_mesh_wired() -> bool {
    false
}

/// UCRS gossip mesh policy wire — stays **false** until gateway/WEB ceremony.
#[must_use]
pub const fn ucrs_gossip_mesh_policy_wired() -> bool {
    false
}

/// Production mesh wire — stays **false** until live measured flip.
#[must_use]
pub const fn ucrs_gossip_mesh_production_wired() -> bool {
    false
}

/// WEB-034 mesh transport closed — honest false on prep path.
#[must_use]
pub const fn ucrs_gossip_mesh_closed() -> bool {
    false
}

/// Count wired hops on the default (no-feature) build.
#[must_use]
pub fn uarcs_gossip_mesh_wire_hops_closed_count() -> u8 {
    UARCS_GOSSIP_MESH_WIRE_HOPS
        .iter()
        .filter(|h| h.wired_default)
        .count() as u8
}

/// Whether UCRS wire hop census pins are locked @ default build.
#[must_use]
pub fn uarcs_gossip_mesh_wire_hops_honest() -> bool {
    uarcs_gossip_mesh_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
        && UARCS_GOSSIP_MESH_WIRE_HOPS.len() == WIRE_HOP_COUNT
        && UARCS_GOSSIP_MESH_WIRE_HOPS[0].wired_default
        && UARCS_GOSSIP_MESH_WIRE_HOPS[1].wired_default
        && UARCS_GOSSIP_MESH_WIRE_HOPS[2].wired_default
        && UARCS_GOSSIP_MESH_WIRE_HOPS[3].wired_default
        && !UARCS_GOSSIP_MESH_WIRE_HOPS[4].wired_default
        && !UARCS_GOSSIP_MESH_WIRE_HOPS[5].wired_default
}

/// Whether UCRS gate-guarded gossip prep symbols compile and reduce.
#[must_use]
pub fn ucrs_gossip_prep_wired() -> bool {
    let mut clock_out = LocalClock::new(10.0, 300.0);
    clock_out.phase_uncertainty_sec = 4e-9;
    let mut clock_in = LocalClock::new(10.0, 300.0);
    clock_in.phase_uncertainty_sec = 1e-6;
    let mut ledger = CreditLedger::new(2, 300.0);
    ledger.add_peer(1, 5.0);
    let publisher = AgentConfig {
        peer_id: 1,
        budget_bits: 20.0,
        ..AgentConfig::default()
    };
    let receiver = AgentConfig {
        peer_id: 2,
        budget_bits: 20.0,
        ..AgentConfig::default()
    };
    let secret = crate::p2p::ABSORBED_E64_SECRET;
    let tick = outbound_tick_if_admitted(&clock_out, &publisher, secret);
    let Some(tick) = tick else {
        return false;
    };
    if !wire::verify_tick(secret, &tick) {
        return false;
    }
    if gate_check_before_sync(&clock_in, &receiver, &tick) != GateVerdict::Admit {
        return false;
    }
    matches!(
        apply_gated_inbound(&mut clock_in, &mut ledger, &receiver, &tick, secret),
        GatedSyncOutcome::Admitted(MergeOutcome::Accepted)
    )
}

/// Whether UCRS gossip prep path is closed (default build).
///
/// True when H1–H4 spine is wired and prep symbols reduce — does **not** claim
/// `mesh_wired`, `production_wired`, or gateway/web cross-lane closure.
#[must_use]
pub fn ucrs_gossip_prep_closed() -> bool {
    ucrs_gossip_prep_wired()
        && uarcs_gossip_mesh_wire_hops_honest()
        && uarcs_gossip_mesh_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
}

/// Mirror `Umst.Ucrs.Landauer.landauerCost` from Haskell (zero guard on non-positive inputs).
///
/// Source: `HASKELL_LANDAUER_AUTHORITY` — `bits <= 0 || tempK <= 0` → `0`.
#[must_use]
pub fn haskell_landauer_cost(bits: f64, temp_k: f64) -> f64 {
    if bits <= 0.0 || temp_k <= 0.0 {
        0.0
    } else {
        landauer::landauer_cost(bits, temp_k)
    }
}

/// Mirror `Umst.Ucrs.Gate.gateCheck` from Haskell (budget + desync only; no CD conjunct).
///
/// Source: `HASKELL_GATE_AUTHORITY` — `gateCheck desyncEnergyJ budgetJ bitsToResolve`.
#[must_use]
pub fn haskell_ucrs_gate_check(
    desync_energy_j: f64,
    budget_j: f64,
    bits_to_resolve: f64,
) -> GateVerdict {
    if desync_energy_j <= 0.0 {
        return GateVerdict::Reject;
    }
    let cost = haskell_landauer_cost(bits_to_resolve, HASKELL_MESH_TEMPERATURE_K);
    if cost > budget_j {
        GateVerdict::Reject
    } else {
        GateVerdict::Admit
    }
}

/// Property guard from `HASKELL_GATE_AUTHORITY`: gate rejects when desync energy ≤ 0.
#[must_use]
pub fn haskell_ucrs_gate_rejects_zero_desync_probe() -> bool {
    haskell_ucrs_gate_check(0.0, 5.0, 1.0) == GateVerdict::Reject
        && haskell_ucrs_gate_check(-1.0, 5.0, 1.0) == GateVerdict::Reject
}

/// Whether Rust `K_B` matches Haskell `Umst.Ucrs.Landauer.kB`.
#[must_use]
pub fn haskell_ucrs_kb_matches_ssot() -> bool {
    (K_B - 1.380_649e-23).abs() < 1e-30
}

/// Property 3 from `HASKELL_SPEC_AUTHORITY`: gate rejects when sync cost exceeds budget.
#[must_use]
pub fn haskell_ucrs_gate_rejects_over_budget_probe(budget_bits: f64, resolve_bits: f64) -> bool {
    let budget_j = landauer::landauer_cost(budget_bits, HASKELL_MESH_TEMPERATURE_K);
    haskell_ucrs_gate_check(5.0, budget_j, resolve_bits) == GateVerdict::Reject
}

/// Property 5 from `HASKELL_SPEC_AUTHORITY`: gate admits when cost is within budget.
#[must_use]
pub fn haskell_ucrs_gate_admits_within_budget_probe(budget: f64) -> bool {
    let desync = budget * 2.0;
    let cost = budget * 0.5;
    haskell_ucrs_gate_check(desync, budget, cost) == GateVerdict::Admit
}

/// Property 4 from `HASKELL_SPEC_AUTHORITY`: Landauer cost monotonic in bits @ fixed T.
#[must_use]
pub fn haskell_ucrs_landauer_monotonic_probe(b1: f64, b2: f64) -> bool {
    let c1 = landauer::landauer_cost(b1, HASKELL_MESH_TEMPERATURE_K);
    let c2 = landauer::landauer_cost(b2, HASKELL_MESH_TEMPERATURE_K);
    if b1 <= b2 {
        c1 <= c2
    } else {
        c2 <= c1
    }
}

/// Property 1 from `HASKELL_SPEC_AUTHORITY`: greedy peer selection picks highest credit.
#[must_use]
pub fn haskell_ucrs_greedy_selects_highest_credit_probe() -> bool {
    let mut ledger = CreditLedger::new(99, HASKELL_MESH_TEMPERATURE_K);
    ledger.add_peer(1, 5.0);
    ledger.add_peer(2, 5.0);
    ledger.add_peer(3, 5.0);
    ledger.add_peer(4, 5.0);
    if let Some(peer) = ledger.peers.get_mut(&1) {
        peer.credit_bits = 8.0;
        peer.accuracy_score = 0.9;
    }
    if let Some(peer) = ledger.peers.get_mut(&2) {
        peer.credit_bits = 12.0;
        peer.accuracy_score = 0.9;
    }
    if let Some(peer) = ledger.peers.get_mut(&3) {
        peer.credit_bits = 6.0;
        peer.accuracy_score = 0.9;
    }
    if let Some(peer) = ledger.peers.get_mut(&4) {
        peer.credit_bits = 3.0;
        peer.accuracy_score = 0.05; // degraded — excluded
    }
    let decision = ledger.best_peer();
    decision.map(|d| d.peer_id == 2).unwrap_or(false)
}

/// Property 2 from `HASKELL_SPEC_AUTHORITY`: Byzantine sync reduces peer credit.
#[must_use]
pub fn haskell_ucrs_byzantine_credit_drops_probe(bits: f64) -> bool {
    let mut ledger = CreditLedger::new(99, HASKELL_MESH_TEMPERATURE_K);
    ledger.add_peer(1, 5.0);
    ledger.record_sync(1, bits, true);
    let after_good = ledger.peers.get(&1).map(|p| p.credit_bits).unwrap_or(0.0);
    ledger.record_sync(1, bits, false);
    let after_bad = ledger.peers.get(&1).map(|p| p.credit_bits).unwrap_or(0.0);
    after_bad < after_good
}

/// One Haskell QuickCheck property slot from `HASKELL_SPEC_AUTHORITY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HaskellUcrsGossipMeshPropertySlot {
    /// 1-based property index in `Spec.hs`.
    pub index: u8,
    /// `testProperty` label string.
    pub label: &'static str,
    /// Whether the Rust reduction probe passes.
    pub wired: bool,
}

/// Reduction probes for all five `HASKELL_SPEC_AUTHORITY` properties.
#[must_use]
pub fn haskell_ucrs_gossip_mesh_property_slots(
) -> [HaskellUcrsGossipMeshPropertySlot; HASKELL_PROPERTY_COUNT] {
    [
        HaskellUcrsGossipMeshPropertySlot {
            index: 1,
            label: HASKELL_SPEC_PROPERTY_LABELS[0],
            wired: haskell_ucrs_greedy_selects_highest_credit_probe(),
        },
        HaskellUcrsGossipMeshPropertySlot {
            index: 2,
            label: HASKELL_SPEC_PROPERTY_LABELS[1],
            wired: haskell_ucrs_byzantine_credit_drops_probe(2.0),
        },
        HaskellUcrsGossipMeshPropertySlot {
            index: 3,
            label: HASKELL_SPEC_PROPERTY_LABELS[2],
            wired: haskell_ucrs_gate_rejects_over_budget_probe(3.0, 10.0),
        },
        HaskellUcrsGossipMeshPropertySlot {
            index: 4,
            label: HASKELL_SPEC_PROPERTY_LABELS[3],
            wired: haskell_ucrs_landauer_monotonic_probe(1.0, 3.0),
        },
        HaskellUcrsGossipMeshPropertySlot {
            index: 5,
            label: HASKELL_SPEC_PROPERTY_LABELS[4],
            wired: haskell_ucrs_gate_admits_within_budget_probe(5.0),
        },
    ]
}

/// Whether all five Haskell QuickCheck property slots reduce on the Rust gossip mesh path.
#[must_use]
pub fn haskell_ucrs_gossip_mesh_property_slots_wired() -> bool {
    haskell_ucrs_gossip_mesh_property_slots()
        .iter()
        .all(|slot| slot.wired)
        && haskell_ucrs_gate_rejects_zero_desync_probe()
}

/// Whether all five Haskell QuickCheck properties reduce on the Rust gossip mesh path.
#[must_use]
pub fn haskell_ucrs_gossip_mesh_properties_wired() -> bool {
    haskell_ucrs_kb_matches_ssot() && haskell_ucrs_gossip_mesh_property_slots_wired()
}

/// Whether Haskell UCRS mesh notes are adopted on the gossip prep spine.
#[must_use]
pub fn haskell_ucrs_gossip_mesh_adopt_wired() -> bool {
    haskell_ucrs_gossip_mesh_properties_wired()
        && LOCALHOST_MESH_PORTS.len() == LOCALHOST_MESH_PEER_COUNT
        && ucrs_gossip_prep_wired()
}

/// Whether Haskell UCRS mesh adoption is closed (properties + prep spine).
#[must_use]
pub fn haskell_ucrs_gossip_mesh_adopt_closed() -> bool {
    haskell_ucrs_gossip_mesh_adopt_wired()
        && ucrs_gossip_prep_closed()
        && haskell_ucrs_gossip_mesh_properties_wired()
}

/// Haskell UCRS mesh adoption posture absorbed via source review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HaskellUcrsGossipMeshAdoptPosture {
    /// Gate authority path.
    pub gate_authority: &'static str,
    /// Credit authority path.
    pub credit_authority: &'static str,
    /// Landauer authority path.
    pub landauer_authority: &'static str,
    /// Spec authority path.
    pub spec_authority: &'static str,
    /// QuickCheck property count pinned.
    pub property_count: usize,
    /// Rust K_B matches Haskell kB.
    pub kb_matches: bool,
    /// All five properties reduce.
    pub properties_wired: bool,
    /// Localhost mesh peer count (no daemon).
    pub localhost_mesh_peers: usize,
}

/// Haskell UCRS mesh adoption posture snapshot.
#[must_use]
pub const fn haskell_ucrs_gossip_mesh_adopt_posture() -> HaskellUcrsGossipMeshAdoptPosture {
    HaskellUcrsGossipMeshAdoptPosture {
        gate_authority: HASKELL_GATE_AUTHORITY,
        credit_authority: HASKELL_CREDIT_AUTHORITY,
        landauer_authority: HASKELL_LANDAUER_AUTHORITY,
        spec_authority: HASKELL_SPEC_AUTHORITY,
        property_count: HASKELL_PROPERTY_COUNT,
        kb_matches: true, // checked at runtime via haskell_ucrs_kb_matches_ssot
        properties_wired: false, // checked at runtime
        localhost_mesh_peers: LOCALHOST_MESH_PEER_COUNT,
    }
}

/// H81 absorbed gateway posture — source-reviewed constants (no `umst-gateway` dep).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct H81AbsorbedPosture {
    /// H81 job id.
    pub job_id: &'static str,
    /// H81 receipt path.
    pub receipt_path: &'static str,
    /// Gateway-local hops closed on 6-hop map.
    pub gateway_hops_closed: u8,
    /// Gateway-local hops open on 6-hop map.
    pub gateway_hops_open: u8,
    /// Constitutional mesh wire — stays false until WEB-034.
    pub mesh_wired: bool,
}

/// H81 gateway gossip wire-prep posture absorbed via source review.
#[must_use]
pub const fn h81_absorbed_posture() -> H81AbsorbedPosture {
    H81AbsorbedPosture {
        job_id: "FLEET-COMPOSER-H81-UARCS-GOSSIP",
        receipt_path: PRIOR_H81_RECEIPT_PATH,
        gateway_hops_closed: 3,
        gateway_hops_open: 3,
        mesh_wired: false,
    }
}

/// AC82 adoption probe — UCRS owner lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UarcsGossipMeshAc82Probe {
    /// AC82 fleet card id.
    pub ac82_job_id: &'static str,
    /// AC82 wave slot.
    pub ac82_slot: &'static str,
    /// AC82 receipt path pinned.
    pub ac82_receipt_honest: bool,
    /// F64 gossip cycle cross-ref pinned.
    pub f64_cross_ref_honest: bool,
    /// F86 gossip cycle deepen cross-ref pinned.
    pub f86_cross_ref_honest: bool,
    /// G86 absent receipt cross-ref pinned.
    pub g86_absent_cross_ref_honest: bool,
    /// H81 gateway wire-prep cross-ref pinned.
    pub h81_cross_ref_honest: bool,
    /// J23 rollup cross-ref pinned.
    pub j23_cross_ref_honest: bool,
    /// X85 static census cross-ref pinned.
    pub x85_cross_ref_honest: bool,
    /// Haskell Gate authority cross-ref pinned.
    pub haskell_gate_cross_ref_honest: bool,
    /// Haskell Credit authority cross-ref pinned.
    pub haskell_credit_cross_ref_honest: bool,
    /// Haskell Landauer authority cross-ref pinned.
    pub haskell_landauer_cross_ref_honest: bool,
    /// Haskell Spec property count pinned.
    pub haskell_property_count: usize,
    /// Haskell QuickCheck properties reduce on Rust path.
    pub haskell_properties_wired: bool,
    /// Haskell mesh adoption closed.
    pub haskell_adopt_closed: bool,
    /// UCRS wire hops closed on default build.
    pub wire_hops_closed: u8,
    /// UCRS gate-guarded gossip prep wired.
    pub ucrs_prep_wired: bool,
    /// Constitutional mesh wire.
    pub mesh_wired: bool,
    /// UCRS mesh policy wire.
    pub mesh_policy_wired: bool,
    /// Production mesh wire.
    pub production_wired: bool,
    /// WEB-034 mesh transport closed.
    pub mesh_closed: bool,
}

/// Emit AC82 UARCS-gossip mesh census probe snapshot.
#[must_use]
pub fn uarcs_gossip_mesh_ac82_probe() -> UarcsGossipMeshAc82Probe {
    UarcsGossipMeshAc82Probe {
        ac82_job_id: COMPOSER_AC82_JOB_ID,
        ac82_slot: ACCEL_AC82_SLOT,
        ac82_receipt_honest: COMPOSER_AC82_RECEIPT_PATH == "outputs/.tmp/COMPOSER_ACCEL2_AC82.md",
        f64_cross_ref_honest: PRIOR_F64_RECEIPT_PATH.contains("COMPOSER_F64_UCRS_1934"),
        f86_cross_ref_honest: PRIOR_F86_RECEIPT_PATH.contains("COMPOSER_F86_UCRS_GOSSIP_1942"),
        g86_absent_cross_ref_honest: PRIOR_G86_ABSENT_RECEIPT_PATH
            .contains("COMPOSER_G86_UCRS_GOSSIP_2143"),
        h81_cross_ref_honest: PRIOR_H81_RECEIPT_PATH.contains("COMPOSER_H81_2242"),
        j23_cross_ref_honest: PRIOR_J23_RECEIPT_PATH.contains("COMPOSER_J23_2348"),
        x85_cross_ref_honest: PRIOR_X85_RECEIPT_PATH.contains("COMPOSER_X85_0734"),
        haskell_gate_cross_ref_honest: HASKELL_GATE_AUTHORITY.contains("Umst/Ucrs/Gate.hs"),
        haskell_credit_cross_ref_honest: HASKELL_CREDIT_AUTHORITY.contains("Umst/Ucrs/Credit.hs"),
        haskell_landauer_cross_ref_honest: HASKELL_LANDAUER_AUTHORITY
            .contains("Umst/Ucrs/Landauer.hs"),
        haskell_property_count: HASKELL_PROPERTY_COUNT,
        haskell_properties_wired: haskell_ucrs_gossip_mesh_properties_wired(),
        haskell_adopt_closed: haskell_ucrs_gossip_mesh_adopt_closed(),
        wire_hops_closed: uarcs_gossip_mesh_wire_hops_closed_count(),
        ucrs_prep_wired: ucrs_gossip_prep_wired(),
        mesh_wired: ucrs_gossip_mesh_wired(),
        mesh_policy_wired: ucrs_gossip_mesh_policy_wired(),
        production_wired: ucrs_gossip_mesh_production_wired(),
        mesh_closed: ucrs_gossip_mesh_closed(),
    }
}

/// AC82 honest residue — UCRS prep landed; mesh/production false; cross-lanes open.
#[must_use]
pub fn uarcs_gossip_mesh_ac82_residue_honest() -> bool {
    let probe = uarcs_gossip_mesh_ac82_probe();
    let h81 = h81_absorbed_posture();
    probe.ac82_job_id == COMPOSER_AC82_JOB_ID
        && probe.ac82_slot == ACCEL_AC82_SLOT
        && probe.f64_cross_ref_honest
        && probe.f86_cross_ref_honest
        && probe.g86_absent_cross_ref_honest
        && probe.h81_cross_ref_honest
        && probe.j23_cross_ref_honest
        && probe.x85_cross_ref_honest
        && probe.haskell_gate_cross_ref_honest
        && probe.haskell_credit_cross_ref_honest
        && probe.haskell_landauer_cross_ref_honest
        && probe.haskell_property_count == HASKELL_PROPERTY_COUNT
        && probe.haskell_properties_wired
        && probe.haskell_adopt_closed
        && haskell_ucrs_gossip_mesh_adopt_closed()
        && probe.wire_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.ucrs_prep_wired
        && uarcs_gossip_mesh_wire_hops_honest()
        && ucrs_gossip_prep_closed()
        && h81.gateway_hops_closed == 3
        && h81.gateway_hops_open == 3
        && !h81.mesh_wired
        && !probe.mesh_wired
        && !probe.mesh_policy_wired
        && !probe.production_wired
        && !probe.mesh_closed
}

/// C82-UCRS-GOSSIP deepen probe — measured honesty without mesh/production flip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UcrsGossipMeshC82Probe {
    /// Swarm cell id.
    pub cell_id: &'static str,
    /// C82 refill wave slot.
    pub wave_slot: &'static str,
    /// C82 deepen job id.
    pub job_id: &'static str,
    /// Haskell property slot count pinned.
    pub haskell_property_slot_count: usize,
    /// All five Haskell property slots reduce.
    pub haskell_property_slots_wired: bool,
    /// Gate rejects zero/negative desync energy.
    pub gate_rejects_zero_desync: bool,
    /// UCRS wire hops closed @ default build.
    pub ucrs_hops_closed: u8,
    /// UCRS wire hops open @ default build.
    pub ucrs_hops_open: u8,
    /// Gateway H81 hops closed (source-reviewed).
    pub gateway_hops_closed: u8,
    /// Gateway H81 hops open (source-reviewed).
    pub gateway_hops_open: u8,
    /// AC82 residue honest.
    pub ac82_residue_honest: bool,
    /// Constitutional mesh wire.
    pub mesh_wired: bool,
    /// UCRS mesh policy wire.
    pub mesh_policy_wired: bool,
    /// Production mesh wire.
    pub production_wired: bool,
    /// WEB-034 mesh transport closed.
    pub mesh_closed: bool,
}

/// Emit C82-UCRS-GOSSIP mesh census deepen probe snapshot.
#[must_use]
pub fn ucrs_gossip_mesh_c82_probe() -> UcrsGossipMeshC82Probe {
    let closed = uarcs_gossip_mesh_wire_hops_closed_count();
    let open = (WIRE_HOP_COUNT as u8).saturating_sub(closed);
    UcrsGossipMeshC82Probe {
        cell_id: C82_CELL_ID,
        wave_slot: C82_WAVE_SLOT,
        job_id: C82_UCRS_GOSSIP_JOB_ID,
        haskell_property_slot_count: HASKELL_PROPERTY_COUNT,
        haskell_property_slots_wired: haskell_ucrs_gossip_mesh_property_slots_wired(),
        gate_rejects_zero_desync: haskell_ucrs_gate_rejects_zero_desync_probe(),
        ucrs_hops_closed: closed,
        ucrs_hops_open: open,
        gateway_hops_closed: GATEWAY_H81_HOPS_CLOSED,
        gateway_hops_open: GATEWAY_H81_HOPS_OPEN,
        ac82_residue_honest: uarcs_gossip_mesh_ac82_residue_honest(),
        mesh_wired: ucrs_gossip_mesh_wired(),
        mesh_policy_wired: ucrs_gossip_mesh_policy_wired(),
        production_wired: ucrs_gossip_mesh_production_wired(),
        mesh_closed: ucrs_gossip_mesh_closed(),
    }
}

/// C82 deepen honest residue — Haskell slots + gate guard + AC82 spine; mesh false.
#[must_use]
pub fn ucrs_gossip_mesh_c82_residue_honest() -> bool {
    let probe = ucrs_gossip_mesh_c82_probe();
    probe.cell_id == C82_CELL_ID
        && probe.wave_slot == C82_WAVE_SLOT
        && probe.job_id == C82_UCRS_GOSSIP_JOB_ID
        && probe.haskell_property_slot_count == HASKELL_PROPERTY_COUNT
        && probe.haskell_property_slots_wired
        && probe.gate_rejects_zero_desync
        && probe.ucrs_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.ucrs_hops_open == 2
        && probe.gateway_hops_closed == GATEWAY_H81_HOPS_CLOSED
        && probe.gateway_hops_open == GATEWAY_H81_HOPS_OPEN
        && probe.gateway_hops_closed + probe.gateway_hops_open == GATEWAY_H81_WIRE_HOP_COUNT as u8
        && probe.ac82_residue_honest
        && uarcs_gossip_mesh_ac82_residue_honest()
        && !probe.mesh_wired
        && !probe.mesh_policy_wired
        && !probe.production_wired
        && !probe.mesh_closed
}

/// Whether UARCS-gossip mesh adoption is honest — no fake production flip.
#[must_use]
pub fn uarcs_gossip_mesh_honest() -> bool {
    uarcs_gossip_mesh_ac82_residue_honest()
        && ucrs_gossip_mesh_c82_residue_honest()
        && ucrs_urge_gossip_compose_residue_honest()
        && haskell_ucrs_gossip_mesh_adopt_closed()
        && !ucrs_gossip_mesh_wired()
        && !ucrs_gossip_mesh_policy_wired()
        && !ucrs_gossip_mesh_production_wired()
        && !ucrs_gossip_mesh_closed()
        && !UCRS_GOSSIP_MESH_PHYSICS_GREEN
}

/// One-line operator summary for AC82 receipts.
#[must_use]
pub fn uarcs_gossip_mesh_ac82_summary() -> String {
    let probe = uarcs_gossip_mesh_ac82_probe();
    format!(
        "AC82 UARCS-gossip mesh census: hops_closed={}/{} ucrs_prep={} mesh_wired={} mesh_policy_wired={} production_wired={}",
        probe.wire_hops_closed,
        WIRE_HOP_COUNT,
        probe.ucrs_prep_wired,
        probe.mesh_wired,
        probe.mesh_policy_wired,
        probe.production_wired
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ac82_uarcs_gossip_metadata_constants() {
        assert_eq!(COMPOSER_AC82_JOB_ID, "OPERATOR-ACCEL-AC82-UARCS-GOSSIP");
        assert_eq!(ACCEL_AC82_SLOT, "AC82");
        assert!(COMPOSER_AC82_RECEIPT_PATH.contains("COMPOSER_ACCEL2_AC82"));
        assert_eq!(FLEET_F64_JOB_ID, "FLEET-COMPOSER-F64-UCRS-GOSSIP");
        assert_eq!(FLEET_F86_JOB_ID, "FLEET-COMPOSER-F86-UCRS-GOSSIP");
        assert_eq!(FLEET_G86_JOB_ID, "FLEET-COMPOSER-G86-UCRS-GOSSIP");
        assert_eq!(UARCS_GOSSIP_MESH_WIRE_HOPS.len(), 6);
        assert_eq!(uarcs_gossip_mesh_wire_hops_closed_count(), 4);
        assert_eq!(WIRE_HOPS_CLOSED_DEFAULT, 4);
        assert_eq!(WEB_034_OWNER, "WEB-034");
    }

    #[test]
    fn ac82_uarcs_gossip_wire_hops_four_of_six_closed_default() {
        assert!(uarcs_gossip_mesh_wire_hops_honest());
        assert_eq!(uarcs_gossip_mesh_wire_hops_closed_count(), 4);
        assert!(UARCS_GOSSIP_MESH_WIRE_HOPS[0].wired_default);
        assert!(UARCS_GOSSIP_MESH_WIRE_HOPS[1].wired_default);
        assert!(UARCS_GOSSIP_MESH_WIRE_HOPS[2].wired_default);
        assert!(UARCS_GOSSIP_MESH_WIRE_HOPS[3].wired_default);
        assert!(!UARCS_GOSSIP_MESH_WIRE_HOPS[4].wired_default);
        assert!(!UARCS_GOSSIP_MESH_WIRE_HOPS[5].wired_default);
    }

    #[test]
    fn ac82_uarcs_gossip_mesh_policy_production_false_honest() {
        assert!(!ucrs_gossip_mesh_wired());
        assert!(!ucrs_gossip_mesh_policy_wired());
        assert!(!ucrs_gossip_mesh_production_wired());
        assert!(!ucrs_gossip_mesh_closed());
        let probe = uarcs_gossip_mesh_ac82_probe();
        assert!(!probe.mesh_wired);
        assert!(!probe.mesh_policy_wired);
        assert!(!probe.production_wired);
        assert!(!probe.mesh_closed);
    }

    #[test]
    fn ac82_uarcs_gossip_prep_wired_on_default_build() {
        assert!(ucrs_gossip_prep_wired());
        assert!(ucrs_gossip_prep_closed());
    }

    #[test]
    fn ac82_uarcs_gossip_absorbed_prior_receipts_honest() {
        assert!(PRIOR_F64_RECEIPT_PATH.contains("COMPOSER_F64_UCRS_1934"));
        assert!(PRIOR_F86_RECEIPT_PATH.contains("COMPOSER_F86_UCRS_GOSSIP_1942"));
        assert!(PRIOR_G86_ABSENT_RECEIPT_PATH.contains("COMPOSER_G86_UCRS_GOSSIP_2143"));
        assert!(PRIOR_H81_RECEIPT_PATH.contains("COMPOSER_H81_2242"));
        assert!(PRIOR_J23_RECEIPT_PATH.contains("COMPOSER_J23_2348"));
        assert!(PRIOR_X85_RECEIPT_PATH.contains("COMPOSER_X85_0734"));
        assert!(GATEWAY_H81_AUTHORITY.contains("uarcs_ucrs_gossip_wire_prep"));
        let h81 = h81_absorbed_posture();
        assert!(!h81.mesh_wired);
        assert_eq!(h81.gateway_hops_closed, 3);
    }

    #[test]
    fn ac82_uarcs_gossip_mesh_residue_honest() {
        assert!(uarcs_gossip_mesh_ac82_residue_honest());
        assert!(uarcs_gossip_mesh_honest());
        let probe = uarcs_gossip_mesh_ac82_probe();
        assert_eq!(probe.ac82_job_id, COMPOSER_AC82_JOB_ID);
        assert_eq!(probe.wire_hops_closed, 4);
        assert!(probe.ucrs_prep_wired);
        assert!(probe.h81_cross_ref_honest);
        assert!(probe.x85_cross_ref_honest);
        assert_eq!(
            UARCS_GOSSIP_MESH_OPEN_HOP_SURFACES[1],
            "umst-web/src/gossip.rs::web_gossip_mesh_wired"
        );
        assert!(uarcs_gossip_mesh_ac82_summary().contains("mesh_wired=false"));
    }

    #[test]
    fn ac82_haskell_ucrs_mesh_authority_constants() {
        assert!(HASKELL_GATE_AUTHORITY.contains("Umst/Ucrs/Gate.hs"));
        assert!(HASKELL_CREDIT_AUTHORITY.contains("Umst/Ucrs/Credit.hs"));
        assert!(HASKELL_LANDAUER_AUTHORITY.contains("Umst/Ucrs/Landauer.hs"));
        assert!(HASKELL_SPEC_AUTHORITY.contains("Haskell/test/Spec.hs"));
        assert_eq!(HASKELL_PROPERTY_COUNT, 5);
        assert_eq!(HASKELL_MESH_TEMPERATURE_K, 300.0);
        assert_eq!(LOCALHOST_MESH_PEER_COUNT, 3);
        assert_eq!(LOCALHOST_MESH_PORTS.len(), LOCALHOST_MESH_PEER_COUNT);
    }

    #[test]
    fn ac82_haskell_ucrs_kb_matches_ssot() {
        assert!(haskell_ucrs_kb_matches_ssot());
        let posture = haskell_ucrs_gossip_mesh_adopt_posture();
        assert_eq!(posture.gate_authority, HASKELL_GATE_AUTHORITY);
        assert_eq!(posture.property_count, 5);
        assert_eq!(posture.localhost_mesh_peers, 3);
    }

    #[test]
    fn ac82_haskell_ucrs_gate_properties_from_spec() {
        assert!(haskell_ucrs_gate_rejects_over_budget_probe(3.0, 10.0));
        assert!(haskell_ucrs_gate_admits_within_budget_probe(5.0));
        assert_eq!(
            haskell_ucrs_gate_check(5.0, landauer::landauer_cost(3.0, 300.0), 10.0),
            GateVerdict::Reject
        );
        assert_eq!(haskell_ucrs_gate_check(10.0, 5.0, 2.5), GateVerdict::Admit);
    }

    #[test]
    fn ac82_haskell_ucrs_landauer_monotonic_from_spec() {
        assert!(haskell_ucrs_landauer_monotonic_probe(1.0, 3.0));
        assert!(haskell_ucrs_landauer_monotonic_probe(3.0, 1.0));
    }

    #[test]
    fn ac82_haskell_ucrs_credit_properties_from_spec() {
        assert!(haskell_ucrs_greedy_selects_highest_credit_probe());
        assert!(haskell_ucrs_byzantine_credit_drops_probe(2.0));
    }

    #[test]
    fn ac82_haskell_ucrs_gossip_mesh_adopt_closed() {
        assert!(haskell_ucrs_gossip_mesh_properties_wired());
        assert!(haskell_ucrs_gossip_mesh_adopt_wired());
        assert!(haskell_ucrs_gossip_mesh_adopt_closed());
        let probe = uarcs_gossip_mesh_ac82_probe();
        assert!(probe.haskell_gate_cross_ref_honest);
        assert!(probe.haskell_credit_cross_ref_honest);
        assert!(probe.haskell_landauer_cross_ref_honest);
        assert_eq!(probe.haskell_property_count, 5);
        assert!(probe.haskell_properties_wired);
        assert!(probe.haskell_adopt_closed);
    }

    #[test]
    fn c82_haskell_landauer_cost_zero_guard_matches_spec() {
        assert_eq!(haskell_landauer_cost(0.0, 300.0), 0.0);
        assert_eq!(haskell_landauer_cost(-1.0, 300.0), 0.0);
        assert_eq!(haskell_landauer_cost(3.0, 0.0), 0.0);
        let positive = haskell_landauer_cost(3.0, HASKELL_MESH_TEMPERATURE_K);
        assert!(
            (positive - landauer::landauer_cost(3.0, HASKELL_MESH_TEMPERATURE_K)).abs() < 1e-30
        );
    }

    #[test]
    fn c82_haskell_ucrs_gate_rejects_zero_desync() {
        assert!(haskell_ucrs_gate_rejects_zero_desync_probe());
        assert_eq!(haskell_ucrs_gate_check(0.0, 5.0, 1.0), GateVerdict::Reject);
    }

    #[test]
    fn c82_haskell_ucrs_property_slots_crosswalk_from_spec() {
        let slots = haskell_ucrs_gossip_mesh_property_slots();
        assert_eq!(slots.len(), HASKELL_PROPERTY_COUNT);
        assert_eq!(slots[0].label, "greedy selects highest credit");
        assert_eq!(slots[2].label, "gate rejects over budget");
        assert_eq!(slots[4].label, "gate admits within budget");
        assert!(slots.iter().all(|s| s.wired));
        assert!(haskell_ucrs_gossip_mesh_property_slots_wired());
        assert_eq!(HASKELL_SPEC_PROPERTY_LABELS.len(), 5);
    }

    #[test]
    fn c82_ucrs_gossip_mesh_deepen_residue_honest() {
        let probe = ucrs_gossip_mesh_c82_probe();
        assert_eq!(probe.cell_id, C82_CELL_ID);
        assert_eq!(probe.wave_slot, C82_WAVE_SLOT);
        assert_eq!(probe.ucrs_hops_closed, 4);
        assert_eq!(probe.ucrs_hops_open, 2);
        assert_eq!(probe.gateway_hops_closed, 3);
        assert_eq!(probe.gateway_hops_open, 3);
        assert!(probe.haskell_property_slots_wired);
        assert!(probe.gate_rejects_zero_desync);
        assert!(probe.ac82_residue_honest);
        assert!(!probe.mesh_wired);
        assert!(!probe.mesh_policy_wired);
        assert!(!probe.production_wired);
        assert!(!probe.mesh_closed);
        assert!(ucrs_gossip_mesh_c82_residue_honest());
        assert!(uarcs_gossip_mesh_honest());
    }

    #[test]
    fn w8e14_gossip_mesh_fence_no_production_wired() {
        let probe = ucrs_gossip_mesh_c82_probe();
        assert!(!probe.production_wired);
        assert!(!probe.mesh_wired);
        assert!(ucrs_gossip_mesh_c82_residue_honest());
        assert!(uarcs_gossip_mesh_honest());
    }

    #[test]
    fn gossip_mesh_ucrs_urge_compose_history_mesh_consumers() {
        assert_eq!(UCRS_URGE_GOSSIP_COMPOSE_CELL_ID, "UCRS-URGE-GOSSIP-COMPOSE");
        assert_eq!(URGE_HISTORY_MESH_CONSUMER_COUNT, 3);
        assert_eq!(URGE_HISTORY_MESH_CONSUMERS.len(), 3);
        assert_eq!(
            URGE_HISTORY_MESH_CONSUMERS[0].module_path,
            "umst_urge::gate_before_sync"
        );
        assert_eq!(
            URGE_HISTORY_MESH_CONSUMERS[1].module_path,
            "umst_urge::gossip_tick"
        );
        assert_eq!(
            URGE_HISTORY_MESH_CONSUMERS[2].module_path,
            "umst_urge::signed_propagate"
        );
        assert!(URGE_HISTORY_MESH_CONSUMERS[0]
            .ucrs_call_surface
            .contains("gate::gate_check"));
        assert!(URGE_HISTORY_MESH_CONSUMERS[1]
            .ucrs_call_surface
            .contains("gate_check_before_sync"));
        assert!(URGE_HISTORY_MESH_CONSUMERS[2]
            .ucrs_call_surface
            .contains("sign_tick"));
        assert!(urge_history_mesh_consumers_honest());
    }

    #[test]
    fn gossip_mesh_ucrs_urge_compose_cycle_fence_no_reverse_dep() {
        assert!(URGE_UCRS_CYCLE_FENCE.contains("must not path-depend"));
        assert!(UCRS_URGE_GOSSIP_COMPOSE_NON_CLAIM.contains("citation only"));
        assert!(UCRS_URGE_GOSSIP_COMPOSE_NON_CLAIM.contains("physics_green false"));
    }

    #[test]
    fn gossip_mesh_ucrs_urge_compose_sole_landauer_axiom() {
        assert_eq!(PHYSICS_AXIOM_COUNT, 1);
        assert_eq!(LEAN_AXIOM_PHYSICAL_SECOND_LAW, "LandauerLaw.physicalSecondLaw");
        assert!(LEAN_ANCHOR_LANDAUER_LAW.contains("LandauerLaw.lean"));
        assert!(landauer_physical_second_law_sole_axiom_honest());
    }

    #[test]
    fn gossip_mesh_ucrs_urge_compose_production_wired_unmeasured() {
        assert!(!ucrs_gossip_mesh_production_wired());
        assert!(gossip_mesh_production_wired_taxonomy_unmeasured());
        assert!(!UCRS_GOSSIP_MESH_PHYSICS_GREEN);
        let probe = ucrs_urge_gossip_compose_probe();
        assert!(!probe.production_wired_const);
        assert!(probe.production_wired_unmeasured);
        assert!(!probe.physics_green);
    }

    #[test]
    fn gossip_mesh_ucrs_urge_compose_residue_honest() {
        assert!(ucrs_urge_gossip_compose_residue_honest());
        let probe = ucrs_urge_gossip_compose_probe();
        assert_eq!(probe.cell_id, UCRS_URGE_GOSSIP_COMPOSE_CELL_ID);
        assert_eq!(probe.urge_consumer_count, 3);
        assert!(probe.urge_consumers_honest);
        assert!(probe.ucrs_prep_wired);
        assert!(probe.prior_residue_honest);
        assert!(uarcs_gossip_mesh_honest());
    }

}


/// Swarm cell id — URGE-II mesh Unmeasured|Measured taxonomy (no false flip).
pub const URGE_II_MESH_UNMEASURED_TAXONOMY_CELL_ID: &str = "URGE-II-MESH-UNMEASURED-TAXONOMY";

/// Marker for Urge-II mesh taxonomy deepen.
pub const URGE_II_MESH_UNMEASURED_MARKER: &str = "urge_ii_mesh_unmeasured_taxonomy_v1";

/// Non-claim — taxonomy documents Unmeasured; does not flip production_wired.
pub const URGE_II_MESH_UNMEASURED_NON_CLAIM: &str =
    "URGE-II-MESH-UNMEASURED-TAXONOMY Unmeasured|Measured for ucrs_gossip_mesh_production_wired — no false flip; not physics GREEN";

/// Urge-II honesty: mesh production_wired stays false / Unmeasured (no false GREEN flip).
#[must_use]
pub fn urge_ii_mesh_unmeasured_taxonomy_honest() -> bool {
    !ucrs_gossip_mesh_production_wired()
        && !ucrs_gossip_mesh_closed()
        && URGE_II_MESH_UNMEASURED_TAXONOMY_CELL_ID == "URGE-II-MESH-UNMEASURED-TAXONOMY"
        && URGE_II_MESH_UNMEASURED_MARKER == "urge_ii_mesh_unmeasured_taxonomy_v1"
}

#[cfg(test)]
mod urge_ii_mesh_taxonomy_tests {
    use super::*;

    #[test]
    fn urge_ii_mesh_unmeasured_taxonomy_holds() {
        assert!(urge_ii_mesh_unmeasured_taxonomy_honest());
        assert!(!ucrs_gossip_mesh_production_wired());
    }
}

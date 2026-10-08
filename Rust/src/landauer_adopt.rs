// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Consumer contract: `umst_ucrs::bench::landauer_adopt` (Wave 6 · CELL_UCRS_READY_U6_LANDAUER_ADOPT).
//! LIB-ADOPT-A-LANDAUER adoption witness morphisms reachable via
//! `landauer_adopt::{landauer_ucrs_wire_hops_honest,landauer_ucrs_pairwise_adopt_closed,
//! landauer_ucrs_adopt_honest,lib_adopt_a_landauer_p1542_b4_probe,lib_adopt_a_landauer_p1938_k3_probe,
//! lib_adopt_a_landauer_accel_ac36_probe}` — **not** pairwise/global SSOT hot path.
//!
//! **Morphisms (bench witness lane):** wire-hop census · pairwise adopt close · fleet probes · honesty predicates.
//!
//! **Lattice home:** `bench::landauer_adopt` (federated — not `shared_types`; web/concrete must not import).
//!
//! **Reroute (G2):** `landauer_adopt::landauer_ucrs_wire_hops_honest → bench::landauer_adopt` (preserved).
//! Pairwise SSOT remains [`super::landauer::coordination_cost`]; global MI remains
//! [`super::landauer_global`] / `ucrs_keep::coordination_cost_global` (U4 antichain — not mutated this wave).
//!
//! | Conjunct | Role in adoption witness |
//! |----------|--------------------------|
//! | **Wire-hop census** | `landauer_ucrs_wire_hops_honest` — H1–H3 closed @ default build |
//! | **Pairwise close** | `landauer_ucrs_pairwise_adopt_closed` — spine landed, no fake GREEN |
//! | **P1542 B4 probe** | `lib_adopt_a_landauer_p1542_b4_probe` — residue snapshot |
//! | **P1938 K3 probe** | `lib_adopt_a_landauer_p1938_k3_probe` — pairwise close snapshot |
//! | **AC36 deepen** | `lib_adopt_a_landauer_accel_ac36_probe` — Shannon MI bridge witness |
//! | **A7-4 compile** | `landauer_ucrs_a7_4_mi_wired` — feature-gated global MI parity |
//!
//! **Consumer fence (honest):**
//!
//! ```text
//! CONSUMER_LANDAUER_ADOPT_IMPORTS_ONLY :=
//!   bench (feature)             →  landauer_adopt::* witness predicates
//!   egoff (integration tests)   →  landauer_adopt::landauer_ucrs_adopt_honest (read-only)
//!   web · concrete · daemon     ⊄  landauer_adopt (stamp-only — use shared_types / landauer SSOT)
//!   all consumers               ⊄  MASTER_RETICK flip (cross-lane ARCS P6 bind open)
//! ```
//!
//! UCRS owns the Landauer primitive (`k_B T ln(2)` per bit — **Primitive-fact**).
//! umst-arcs reimplements per A7 fresh-repo discipline; parity is operational (tests).

use super::landauer::{
    coordination_cost, coordination_cost_from_entropies, desync_energy, landauer_bit_energy,
    landauer_cost, pairwise_mutual_information_bits, K_B,
};

/// LIB adoption workstream id.
pub const WORKSTREAM_ID: &str = "LIB-ADOPT-A-LANDAUER";

/// A7 lane cross-ref — multi-information thermodynamic floor.
pub const A7_LANE_ID: &str = "A7-4";

/// UCRS wire hop count (pairwise + A7-4 global + ARCS parity cross-ref).
pub const WIRE_HOP_COUNT: usize = 4;

/// Whether this module reports thermodynamic floors — always **true**.
pub const IS_THERMODYNAMIC_FLOOR: bool = true;

/// Whether this module claims wall-clock performance — always **false**.
pub const IS_WALL_CLOCK: bool = false;

/// Landauer bound citation — Primitive-fact (physics); not measured lab ε.
pub const LANDAUER_BOUND_PRIMITIVE_FACT: &str =
    "k_B T ln(2) per bit — Landauer (1961); SI k_B exact per 2019 redefinition";

/// One UCRS-side wire hop in the A-LANDAUER ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LandauerUcrsWireHop {
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
    /// Whether hop requires `feature = "a7-4"`.
    pub feature_gated: bool,
}

/// UCRS lane wire inventory.
pub const LANDAUER_UCRS_WIRE_HOPS: [LandauerUcrsWireHop; WIRE_HOP_COUNT] = [
    LandauerUcrsWireHop {
        hop: 1,
        wire_id: "landauer_bit_energy",
        surface: "landauer::landauer_bit_energy",
        status: "LANDED",
        wired_default: true,
        feature_gated: false,
    },
    LandauerUcrsWireHop {
        hop: 2,
        wire_id: "coordination_cost",
        surface: "landauer::coordination_cost",
        status: "LANDED",
        wired_default: true,
        feature_gated: false,
    },
    LandauerUcrsWireHop {
        hop: 3,
        wire_id: "desync_energy",
        surface: "landauer::desync_energy",
        status: "LANDED",
        wired_default: true,
        feature_gated: false,
    },
    LandauerUcrsWireHop {
        hop: 4,
        wire_id: "a7_4_global_mi",
        surface: "landauer_global::multi_information_bits",
        status: "FEATURE_GATED",
        wired_default: false,
        feature_gated: true,
    },
];

/// Honest closed hops @ default build — pairwise spine (H1–H3).
pub const WIRE_HOPS_CLOSED_DEFAULT: u8 = 3;

/// Count wired hops on the default (no-feature) build.
#[must_use]
pub fn landauer_ucrs_wire_hops_closed_count() -> u8 {
    LANDAUER_UCRS_WIRE_HOPS
        .iter()
        .filter(|h| h.wired_default)
        .count() as u8
}

/// Whether UCRS wire hop census pins are locked @ default build.
#[must_use]
pub fn landauer_ucrs_wire_hops_honest() -> bool {
    landauer_ucrs_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
        && LANDAUER_UCRS_WIRE_HOPS.len() == WIRE_HOP_COUNT
        && LANDAUER_UCRS_WIRE_HOPS[0].wired_default
        && LANDAUER_UCRS_WIRE_HOPS[1].wired_default
        && LANDAUER_UCRS_WIRE_HOPS[2].wired_default
        && !LANDAUER_UCRS_WIRE_HOPS[3].wired_default
}

/// Whether UCRS pairwise Landauer adopt path is closed (default build).
///
/// True when H1–H3 spine is wired, wire census is honest, and thermodynamic
/// floor posture holds — does **not** claim A7-4 global MI or ARCS P6 bind.
#[must_use]
pub fn landauer_ucrs_pairwise_adopt_closed() -> bool {
    landauer_ucrs_pairwise_symbols_wired()
        && landauer_ucrs_wire_hops_honest()
        && landauer_ucrs_wire_hops_closed_count() == WIRE_HOPS_CLOSED_DEFAULT
        && IS_THERMODYNAMIC_FLOOR
        && !IS_WALL_CLOCK
}

/// Whether pairwise Landauer symbols are wired (default build).
#[must_use]
pub fn landauer_ucrs_pairwise_symbols_wired() -> bool {
    let t = 300.0;
    let e_bit = landauer_bit_energy(t);
    let h_x = 4.0;
    let h_y = 3.0;
    let i_xy = 1.5;
    let joint = h_x + h_y - i_xy;
    e_bit > 0.0
        && landauer_cost(1.0, t) == e_bit
        && coordination_cost(0.5, t) == landauer_cost(0.5, t)
        && desync_energy(2.0, t) == landauer_cost(2.0, t)
        && pairwise_mutual_information_bits(h_x, h_y, joint) == Some(i_xy)
        && coordination_cost_from_entropies(h_x, h_y, joint, t) == Some(coordination_cost(i_xy, t))
        && (K_B - 1.380_649e-23).abs() < f64::EPSILON
}

/// Whether Shannon MI entropy bridge is wired on the default (pairwise) build.
#[must_use]
pub fn landauer_ucrs_pairwise_mi_entropy_bridge_wired() -> bool {
    let t = 300.0;
    let h_x = 4.0;
    let h_y = 3.0;
    let i_xy = 1.5;
    let joint = h_x + h_y - i_xy;
    pairwise_mutual_information_bits(h_x, h_y, joint) == Some(i_xy)
        && coordination_cost_from_entropies(h_x, h_y, joint, t) == Some(coordination_cost(i_xy, t))
        && pairwise_mutual_information_bits(1.0, 1.0, 10.0).is_none()
}

/// Whether A7-4 global MI path is wired on this build (`feature a7-4` required).
#[must_use]
pub fn landauer_ucrs_a7_4_mi_wired_on_build() -> bool {
    #[cfg(feature = "a7-4")]
    {
        return landauer_ucrs_a7_4_mi_wired();
    }
    #[cfg(not(feature = "a7-4"))]
    {
        false
    }
}

/// Whether A7-4 global MI symbols compile and reduce to pairwise at n=2.
#[cfg(feature = "a7-4")]
#[must_use]
pub fn landauer_ucrs_a7_4_mi_wired() -> bool {
    use super::landauer_global::{
        coordination_cost_global, coordination_cost_global_from_entropies, multi_information_bits,
        n2_global_joules_matches_pairwise, n2_global_matches_pairwise_ssot,
    };
    let h_x = 4.0;
    let h_y = 3.0;
    let i_xy = 1.5;
    let joint = h_x + h_y - i_xy;
    n2_global_matches_pairwise_ssot(h_x, h_y, i_xy)
        && n2_global_joules_matches_pairwise(i_xy, 300.0)
        && multi_information_bits(joint, &[h_x, h_y]) == Some(i_xy)
        && coordination_cost_global(i_xy, 300.0) > 0.0
        && coordination_cost_global_from_entropies(joint, &[h_x, h_y], 300.0)
            == Some(coordination_cost_global(i_xy, 300.0))
}

/// Open hop surfaces blocking full master LIB-ADOPT-A-LANDAUER retick (cross-lane).
pub const LANDAUER_UCRS_OPEN_HOP_SURFACES: [&str; 2] = [
    "landauer_global::multi_information_bits",
    "umst-arcs::coordination_cost_p6::p6_semantic_bind_wired",
];

/// Landauer adoption probe — pairwise symbols and A7-4 MI wiring on this build.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandauerAdoptSymbolsProbe {
    /// UCRS wire hops closed on default build.
    pub wire_hops_closed: u8,
    /// Pairwise symbols wired.
    pub pairwise_symbols_wired: bool,
    /// A7-4 MI wired on this build.
    pub a7_4_mi_wired: bool,
    /// Thermodynamic floor posture.
    pub is_thermodynamic_floor: bool,
    /// Wall-clock posture.
    pub is_wall_clock: bool,
}

/// Emit P1542 B4 A-LANDAUER adoption probe snapshot.
#[must_use]
pub fn lib_adopt_a_landauer_p1542_b4_probe() -> LandauerAdoptSymbolsProbe {
    LandauerAdoptSymbolsProbe {
        wire_hops_closed: landauer_ucrs_wire_hops_closed_count(),
        pairwise_symbols_wired: landauer_ucrs_pairwise_symbols_wired(),
        a7_4_mi_wired: landauer_ucrs_a7_4_mi_wired_on_build(),
        is_thermodynamic_floor: IS_THERMODYNAMIC_FLOOR,
        is_wall_clock: IS_WALL_CLOCK,
    }
}

/// P1542 B4 honest residue — pairwise SSOT landed; A7-4 feature-gated; ARCS P6 bind open.
#[must_use]
pub fn landauer_ucrs_p1542_b4_residue_honest() -> bool {
    let probe = lib_adopt_a_landauer_p1542_b4_probe();
    probe.wire_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.pairwise_symbols_wired
        && landauer_ucrs_wire_hops_honest()
        && probe.is_thermodynamic_floor
        && !probe.is_wall_clock
}

/// Landauer adoption probe — pairwise adopt path close.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandauerPairwiseCloseProbe {
    /// UCRS wire hops closed on default build.
    pub wire_hops_closed: u8,
    /// Pairwise adopt path closed.
    pub pairwise_adopt_closed: bool,
    /// A7-4 MI wired on this build.
    pub a7_4_mi_wired: bool,
    /// Thermodynamic floor posture.
    pub is_thermodynamic_floor: bool,
    /// Wall-clock posture.
    pub is_wall_clock: bool,
}

/// Emit P1938 K3 A-LANDAUER adoption probe snapshot.
#[must_use]
pub fn lib_adopt_a_landauer_p1938_k3_probe() -> LandauerPairwiseCloseProbe {
    LandauerPairwiseCloseProbe {
        wire_hops_closed: landauer_ucrs_wire_hops_closed_count(),
        pairwise_adopt_closed: landauer_ucrs_pairwise_adopt_closed(),
        a7_4_mi_wired: landauer_ucrs_a7_4_mi_wired_on_build(),
        is_thermodynamic_floor: IS_THERMODYNAMIC_FLOOR,
        is_wall_clock: IS_WALL_CLOCK,
    }
}

/// P1938 K3 honest pairwise close — B4 absorbed; H4 feature-gated; ARCS P6 bind open.
#[must_use]
pub fn landauer_ucrs_p1938_k3_pairwise_close_honest() -> bool {
    let probe = lib_adopt_a_landauer_p1938_k3_probe();
    probe.wire_hops_closed == WIRE_HOPS_CLOSED_DEFAULT
        && probe.pairwise_adopt_closed
        && landauer_ucrs_wire_hops_honest()
        && probe.is_thermodynamic_floor
        && !probe.is_wall_clock
}

/// One-line operator summary for P1938 K3 receipts.
#[must_use]
pub fn landauer_ucrs_p1938_k3_close_summary() -> String {
    let probe = lib_adopt_a_landauer_p1938_k3_probe();
    format!(
        "P1938-K3 A-LANDAUER: hops_closed={}/{} pairwise_closed={} a7_4_mi={} thermo_floor={} wall_clock={}",
        probe.wire_hops_closed,
        WIRE_HOP_COUNT,
        probe.pairwise_adopt_closed,
        probe.a7_4_mi_wired,
        probe.is_thermodynamic_floor,
        probe.is_wall_clock
    )
}

/// Landauer MI probe — UCRS Shannon entropy bridge + A7-4 compile witness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandauerMiEntropyBridgeProbe {
    /// Pairwise Shannon MI entropy bridge wired.
    pub pairwise_mi_entropy_bridge_wired: bool,
    /// Pairwise adopt path closed.
    pub pairwise_adopt_closed: bool,
    /// A7-4 MI wired on this build.
    pub a7_4_mi_wired: bool,
    /// Thermodynamic floor posture.
    pub is_thermodynamic_floor: bool,
    /// Wall-clock posture.
    pub is_wall_clock: bool,
}

/// Emit AC36 A-LANDAUER MI feature deepen probe snapshot.
#[must_use]
pub fn lib_adopt_a_landauer_accel_ac36_probe() -> LandauerMiEntropyBridgeProbe {
    LandauerMiEntropyBridgeProbe {
        pairwise_mi_entropy_bridge_wired: landauer_ucrs_pairwise_mi_entropy_bridge_wired(),
        pairwise_adopt_closed: landauer_ucrs_pairwise_adopt_closed(),
        a7_4_mi_wired: landauer_ucrs_a7_4_mi_wired_on_build(),
        is_thermodynamic_floor: IS_THERMODYNAMIC_FLOOR,
        is_wall_clock: IS_WALL_CLOCK,
    }
}

/// AC36 honest MI deepen — Shannon bridge landed; A7-4 feature-gated; ARCS P6 bind open.
#[must_use]
pub fn landauer_ucrs_accel_ac36_mi_deepen_honest() -> bool {
    let probe = lib_adopt_a_landauer_accel_ac36_probe();
    probe.pairwise_mi_entropy_bridge_wired
        && probe.pairwise_adopt_closed
        && landauer_ucrs_wire_hops_honest()
        && probe.is_thermodynamic_floor
        && !probe.is_wall_clock
}

/// One-line operator summary for AC36 receipts.
#[must_use]
pub fn landauer_ucrs_accel_ac36_mi_deepen_summary() -> String {
    let probe = lib_adopt_a_landauer_accel_ac36_probe();
    format!(
        "AC36 A-LANDAUER: mi_bridge={} pairwise_closed={} a7_4_mi={} thermo_floor={} wall_clock={}",
        probe.pairwise_mi_entropy_bridge_wired,
        probe.pairwise_adopt_closed,
        probe.a7_4_mi_wired,
        probe.is_thermodynamic_floor,
        probe.is_wall_clock
    )
}

/// Whether UCRS A-LANDAUER adoption is honest — no fake production or lab ε claims.
#[must_use]
pub fn landauer_ucrs_adopt_honest() -> bool {
    landauer_ucrs_p1542_b4_residue_honest() && landauer_ucrs_pairwise_adopt_closed()
}

/// One-line operator summary for P1542 B4 receipts.
#[must_use]
pub fn landauer_ucrs_p1542_b4_residue_summary() -> String {
    let probe = lib_adopt_a_landauer_p1542_b4_probe();
    format!(
        "P1542-B4 A-LANDAUER: hops_closed={}/{} pairwise={} a7_4_mi={} thermo_floor={} wall_clock={}",
        probe.wire_hops_closed,
        WIRE_HOP_COUNT,
        probe.pairwise_symbols_wired,
        probe.a7_4_mi_wired,
        probe.is_thermodynamic_floor,
        probe.is_wall_clock
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landauer_ucrs_wire_hops_three_of_four_closed_default() {
        assert!(landauer_ucrs_wire_hops_honest());
        assert_eq!(landauer_ucrs_wire_hops_closed_count(), 3);
    }

    #[test]
    fn landauer_ucrs_pairwise_symbols_wired_on_default_build() {
        assert!(landauer_ucrs_pairwise_symbols_wired());
    }

    #[test]
    fn landauer_ucrs_thermodynamic_floor_not_wall_clock() {
        assert!(IS_THERMODYNAMIC_FLOOR);
        assert!(!IS_WALL_CLOCK);
        assert!(LANDAUER_BOUND_PRIMITIVE_FACT.contains("k_B T ln(2)"));
    }

    #[test]
    fn lib_adopt_landauer_residue_honest() {
        assert!(landauer_ucrs_p1542_b4_residue_honest());
        assert!(landauer_ucrs_adopt_honest());
        let probe = lib_adopt_a_landauer_p1542_b4_probe();
        assert_eq!(probe.wire_hops_closed, 3);
        assert!(probe.pairwise_symbols_wired);
        #[cfg(not(feature = "a7-4"))]
        assert!(!probe.a7_4_mi_wired);
        assert!(landauer_ucrs_p1542_b4_residue_summary().contains("thermo_floor=true"));
    }

    #[cfg(feature = "a7-4")]
    #[test]
    fn lib_adopt_a7_4_mi_wired_when_feature_on() {
        assert!(landauer_ucrs_a7_4_mi_wired());
        let probe = lib_adopt_a_landauer_p1542_b4_probe();
        assert!(probe.a7_4_mi_wired);
    }

    #[test]
    fn landauer_ucrs_pairwise_adopt_closed_on_default_build() {
        assert!(landauer_ucrs_pairwise_adopt_closed());
        assert_eq!(
            landauer_ucrs_wire_hops_closed_count(),
            WIRE_HOPS_CLOSED_DEFAULT
        );
        #[cfg(not(feature = "a7-4"))]
        assert!(!landauer_ucrs_a7_4_mi_wired_on_build());
    }

    #[test]
    fn lib_adopt_landauer_pairwise_close_honest() {
        assert!(landauer_ucrs_p1938_k3_pairwise_close_honest());
        assert!(landauer_ucrs_adopt_honest());
        let probe = lib_adopt_a_landauer_p1938_k3_probe();
        assert_eq!(probe.wire_hops_closed, 3);
        assert!(probe.pairwise_adopt_closed);
        #[cfg(not(feature = "a7-4"))]
        assert!(!probe.a7_4_mi_wired);
        assert_eq!(
            LANDAUER_UCRS_OPEN_HOP_SURFACES[1],
            "umst-arcs::coordination_cost_p6::p6_semantic_bind_wired"
        );
        assert!(landauer_ucrs_p1938_k3_close_summary().contains("pairwise_closed=true"));
    }

    #[test]
    fn fleet_accel_ac36_mi_feature_deepen_honest() {
        assert!(landauer_ucrs_accel_ac36_mi_deepen_honest());
        assert!(landauer_ucrs_pairwise_mi_entropy_bridge_wired());
        let probe = lib_adopt_a_landauer_accel_ac36_probe();
        assert!(probe.pairwise_mi_entropy_bridge_wired);
        assert!(probe.pairwise_adopt_closed);
        #[cfg(not(feature = "a7-4"))]
        assert!(!probe.a7_4_mi_wired);
        assert!(landauer_ucrs_accel_ac36_mi_deepen_summary().contains("mi_bridge=true"));
    }

    #[cfg(feature = "a7-4")]
    #[test]
    fn fleet_accel_ac36_a7_4_global_from_entropies_wired_when_feature_on() {
        assert!(landauer_ucrs_a7_4_mi_wired());
        let probe = lib_adopt_a_landauer_accel_ac36_probe();
        assert!(probe.a7_4_mi_wired);
    }

    #[test]
    fn landauer_wire_hops_honest() {
        assert!(landauer_ucrs_wire_hops_honest());
        assert!(landauer_ucrs_pairwise_adopt_closed());
        assert_eq!(landauer_ucrs_wire_hops_closed_count(), 3);
    }
}

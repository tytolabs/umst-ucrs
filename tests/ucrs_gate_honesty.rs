// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! FLEET-COMPOSER-ACCEL-L AC392 — UCRS gate honesty probe.
//!
//! Read-only `umst-ucrs` thermodynamic sync gate witness @ workspace HEAD. Slot maps
//! `392 % 12 == 8` → gate-conjunct cluster **I** (CD + Landauer budget + monotone sync).
//! Band L slot (`AC369..=AC408`). Absorbs AC373 V-UCRS census + existing gate fixture
//! drift tests without flipping `production_wired=true`, writing `umst-web/**`, or
//! inventing GREEN close.
//!
//! Band L honest probe (`392 % 12 == 8`). Write-set isolated: `ucrs_gate_honesty.rs` only.
//! composer-2.5 NOT fast · MASTER_RETICK: no · no GREEN invent.
//!
//! Receipt SSOT: `outputs/.tmp/COMPOSER_ACCEL2_AC392.md`.
//! Fleet: `outputs/.tmp/FLEET_COMPOSER_ACCEL_L_2246.md` by ref.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use umst_math::clausius_duhem_admissible;
use umst_ucrs::gate::{gate_check, gated_sync, ClockThermState, GateVerdict};
use umst_ucrs::gossip_mesh_census::{
    ucrs_gossip_mesh_production_wired, uarcs_gossip_mesh_wire_hops_closed_count,
    uarcs_gossip_mesh_wire_hops_honest, WIRE_HOPS_CLOSED_DEFAULT as GOSSIP_WIRE_HOPS_CLOSED,
};
use umst_ucrs::landauer::{desync_energy, landauer_cost};
use umst_ucrs::landauer_adopt::{
    landauer_ucrs_adopt_honest, landauer_ucrs_pairwise_adopt_closed,
    landauer_ucrs_wire_hops_closed_count, landauer_ucrs_wire_hops_honest,
    WIRE_HOPS_CLOSED_DEFAULT as LANDAUER_WIRE_HOPS_CLOSED,
};
use umst_ucrs::uarcs_004_policy_present::{
    uarcs_004_policy_present_wire_hops_honest, uarcs_004_present_production_wired,
    WIRE_HOPS_CLOSED_DEFAULT as UARCS_004_WIRE_HOPS_CLOSED,
};
use umst_ucrs::uarcs_a7_4_policy_wire::{
    uarcs_a7_4_policy_wire_hops_honest, uarcs_a7_4_production_wired,
};

/// Honest adoption tier — mirrors UCRS gate census posture.
pub const POSTURE_TAG: &str = "honest-gate-census-only";

/// FLEET-COMPOSER-ACCEL-L parent fleet id (Band L · AC369..=AC408).
pub const FLEET_PARENT: &str = "FLEET-COMPOSER-ACCEL-L";

/// AC392 agent job id.
pub const JOB_ID: &str = "FLEET-COMPOSER-ACCEL-L-AC392-UCRS-GATE-HONESTY";

/// AC392 receipt path — SSOT for this pass.
pub const RECEIPT_PATH: &str = "outputs/.tmp/COMPOSER_ACCEL2_AC392.md";

/// Band-L slot floor (AC369 capstone).
pub const BAND_L_SLOT_FLOOR: u16 = 369;

/// Band-L slot ceiling (AC408).
pub const BAND_L_SLOT_CEILING: u16 = 408;

/// Slot cycle remainder — `392 % 12 == 8`.
pub const SLOT_CYCLE_REMAINDER: u8 = 8;

/// Target gate-conjunct cluster letter — maps remainder 8 → cluster **I**.
pub const TARGET_CLUSTER: char = 'I';

/// Honest verify posture — census probe only; no substrate GREEN invent.
pub const VERIFY_POSTURE: &str = "CENSUS_ONLY";

/// Honest sync-gate wire-hop counter — field advance does not fake N.
pub const GATE_WIRE_HOPS_CLOSED: &str = "3/4";

/// Gate authority (read-only crosswalk).
pub const GATE_AUTHORITY: &str = "umst-ucrs/Rust/src/gate.rs";

/// CD drift fixture authority (read-only crosswalk).
pub const GATE_CD_FIXTURE_AUTHORITY: &str = "umst-ucrs/Rust/tests/gate_cd_drift_fixture.rs";

/// Full conjunct drift fixture authority (read-only crosswalk).
pub const GATE_FULL_FIXTURE_AUTHORITY: &str = "umst-ucrs/Rust/tests/gate_full_conjunct_drift_fixture.rs";

/// Absorbed AC373 V-UCRS census receipt (read-only cite).
pub const ABSORBED_AC373_RECEIPT: &str = "outputs/.tmp/COMPOSER_ACCEL2_AC373.md";

/// UCRS consumer wire plan cross-ref (read-only cite).
pub const UCRS_CONSUMER_WIRE_PLAN: &str = "docs/UCRS_CONSUMER_WIRE_PLAN.md";

/// One sync-gate conjunct on the honest census map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UcrsGateWireHop {
    pub hop: u8,
    pub wire_id: &'static str,
    pub surface: &'static str,
    pub wired_default: bool,
}

/// UCRS sync-gate wire inventory @ AC392.
pub const UCRS_GATE_WIRE_HOPS: [UcrsGateWireHop; 4] = [
    UcrsGateWireHop {
        hop: 1,
        wire_id: "landauer_budget",
        surface: "gate::gate_check — sync_cost ≤ budget_j",
        wired_default: true,
    },
    UcrsGateWireHop {
        hop: 2,
        wire_id: "clausius_duhem",
        surface: "umst_math::clausius_duhem_admissible on desync ψ",
        wired_default: true,
    },
    UcrsGateWireHop {
        hop: 3,
        wire_id: "gated_sync_monotone",
        surface: "gate::gated_sync — total_sync_cost_j monotone",
        wired_default: true,
    },
    UcrsGateWireHop {
        hop: 4,
        wire_id: "p2p_daemon_production",
        surface: "p2p::apply_gated_inbound — feature p2p gated",
        wired_default: false,
    },
];

/// Open hop surface ids (honest residue).
pub const UCRS_GATE_OPEN_HOP_SURFACES: [&'static str; 1] = ["p2p_daemon_production"];

/// AC392 honest probe snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UcrsAc392Probe {
    pub job_id: &'static str,
    pub receipt_path: &'static str,
    pub cluster_letter: char,
    pub gate_wire_hops_closed: u8,
    pub gate_wire_hops_open: u8,
    pub production_wired: bool,
    pub master_retick_eligible: bool,
    pub verify_posture: &'static str,
    pub band_l_floor: u16,
    pub band_l_ceiling: u16,
}

/// AC392 census rollup for UCRS sync gate + deepen module fences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UcrsAc392Census {
    pub authority_present: bool,
    pub map_honest: bool,
    pub production_wired: bool,
    pub wire_honest: bool,
    pub master_retick_eligible: bool,
    pub cd_fixture_honest: bool,
    pub landauer_adopt_honest: bool,
    pub ac373_absorbed: bool,
}

#[derive(serde::Deserialize)]
struct CdVector {
    old_psi: f64,
    new_psi: f64,
    expect_admissible: bool,
}

fn workspace_root() -> PathBuf {
    if let Ok(root) = env::var("TYTO_WORKSPACE_ROOT") {
        let p = PathBuf::from(&root);
        if p.join("umst-ucrs/umst.toml").exists() {
            return p;
        }
    }
    for start in [
        env::current_dir().ok(),
        env::var("CARGO_MANIFEST_DIR").ok().map(PathBuf::from),
    ]
    .into_iter()
    .flatten()
    {
        let mut cur = start;
        loop {
            if cur.join("scripts/gen-residue-ledger.sh").exists()
                || cur.join("umst-ucrs/umst.toml").exists()
            {
                return cur;
            }
            if !cur.pop() {
                break;
            }
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[must_use]
pub const fn ucrs_ac392_production_wired() -> bool {
    false
}

#[must_use]
pub const fn ucrs_ac392_master_retick_eligible() -> bool {
    false
}

#[must_use]
pub fn ucrs_gate_wire_hops_closed_count() -> u8 {
    UCRS_GATE_WIRE_HOPS
        .iter()
        .filter(|h| h.wired_default)
        .count() as u8
}

#[must_use]
pub fn ucrs_gate_wire_hops_honest() -> bool {
    ucrs_gate_wire_hops_closed_count() == 3
        && UCRS_GATE_WIRE_HOPS.len() == 4
        && UCRS_GATE_WIRE_HOPS[0].wired_default
        && UCRS_GATE_WIRE_HOPS[1].wired_default
        && UCRS_GATE_WIRE_HOPS[2].wired_default
        && !UCRS_GATE_WIRE_HOPS[3].wired_default
}

#[must_use]
pub fn ucrs_ac392_probe_honest() -> UcrsAc392Probe {
    UcrsAc392Probe {
        job_id: JOB_ID,
        receipt_path: RECEIPT_PATH,
        cluster_letter: TARGET_CLUSTER,
        gate_wire_hops_closed: ucrs_gate_wire_hops_closed_count(),
        gate_wire_hops_open: (UCRS_GATE_WIRE_HOPS.len() as u8) - ucrs_gate_wire_hops_closed_count(),
        production_wired: false,
        master_retick_eligible: false,
        verify_posture: VERIFY_POSTURE,
        band_l_floor: BAND_L_SLOT_FLOOR,
        band_l_ceiling: BAND_L_SLOT_CEILING,
    }
}

#[must_use]
pub fn ucrs_gate_cd_conjunct_honest() -> bool {
    let t = 300.0;
    let state = ClockThermState {
        desync_energy_j: desync_energy(5.0, t),
        budget_j: landauer_cost(10.0, t),
        temperature_k: t,
        total_sync_cost_j: 0.0,
    };
    gate_check(&state, 3.0) == GateVerdict::Admit
        && gate_check(&state, 15.0) == GateVerdict::Reject
}

#[must_use]
pub fn ucrs_gate_gated_sync_monotone_honest() -> bool {
    let t = 300.0;
    let state = ClockThermState {
        desync_energy_j: desync_energy(5.0, t),
        budget_j: landauer_cost(10.0, t),
        temperature_k: t,
        total_sync_cost_j: 0.0,
    };
    let s1 = match gated_sync(&state, 2.0) {
        Some(s) => s,
        None => return false,
    };
    let s1_drifted = ClockThermState {
        desync_energy_j: desync_energy(3.0, t),
        ..s1
    };
    let s2 = match gated_sync(&s1_drifted, 3.0) {
        Some(s) => s,
        None => return false,
    };
    s2.total_sync_cost_j > s1.total_sync_cost_j
}

pub fn ucrs_gate_cd_fixture_honest(root: &Path) -> bool {
    let fixture = root.join("umst-ucrs/Rust/fixtures/ucrs_gate_cd_vectors.json");
    let raw = match fs::read_to_string(fixture) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let vectors: Vec<CdVector> = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return false,
    };
    if vectors.is_empty() {
        return false;
    }
    vectors.iter().all(|v| {
        clausius_duhem_admissible(v.old_psi, v.new_psi) == v.expect_admissible
    })
}

#[must_use]
pub fn ucrs_ac392_map_honest() -> bool {
    ucrs_gate_wire_hops_honest()
        && ucrs_gate_cd_conjunct_honest()
        && ucrs_gate_gated_sync_monotone_honest()
        && landauer_ucrs_wire_hops_honest()
        && uarcs_004_policy_present_wire_hops_honest()
        && uarcs_a7_4_policy_wire_hops_honest()
        && uarcs_gossip_mesh_wire_hops_honest()
        && !ucrs_ac392_production_wired()
}

pub fn census_ucrs_ac392(root: &Path) -> Result<UcrsAc392Census, String> {
    let gate_src = root.join(GATE_AUTHORITY);
    let map_honest = ucrs_ac392_map_honest();
    Ok(UcrsAc392Census {
        authority_present: gate_src.is_file(),
        map_honest,
        production_wired: ucrs_ac392_production_wired(),
        wire_honest: map_honest
            && ucrs_gate_wire_hops_closed_count() == 3
            && UCRS_GATE_OPEN_HOP_SURFACES == ["p2p_daemon_production"],
        master_retick_eligible: ucrs_ac392_master_retick_eligible(),
        cd_fixture_honest: ucrs_gate_cd_fixture_honest(root),
        landauer_adopt_honest: landauer_ucrs_adopt_honest(),
        ac373_absorbed: root.join(ABSORBED_AC373_RECEIPT).is_file()
            || root.join("crates/umst-meta/src/v_ucrs.rs").is_file(),
    })
}

pub fn verify_ucrs_ac392(root: &Path) -> Result<UcrsAc392Census, String> {
    let census = census_ucrs_ac392(root)?;
    if census.production_wired || census.master_retick_eligible {
        return Err("AC392 regression: census fences must stay false".into());
    }
    if uarcs_004_present_production_wired() {
        return Err("AC392 regression: uarcs_004_present_production_wired drift".into());
    }
    if uarcs_a7_4_production_wired() {
        return Err("AC392 regression: uarcs_a7_4_production_wired drift".into());
    }
    if ucrs_gossip_mesh_production_wired() {
        return Err("AC392 regression: ucrs_gossip_mesh_production_wired drift".into());
    }
    if !census.map_honest {
        return Err("AC392 regression: UCRS gate map honesty drift".into());
    }
    if !census.wire_honest {
        return Err("AC392 regression: gate wire hop honesty drift".into());
    }
    if !census.cd_fixture_honest {
        return Err("AC392 regression: CD fixture honesty drift".into());
    }
    if !census.landauer_adopt_honest {
        return Err("AC392 regression: landauer adopt honesty drift".into());
    }
    Ok(census)
}

pub fn probe_ucrs_ac392(root: &Path) -> Result<String, String> {
    let probe = ucrs_ac392_probe_honest();
    let census = verify_ucrs_ac392(root)?;
    Ok(format!(
        "job={} slot=AC392 cycle={} band_l={}..={} cluster={} authority={} \
         gate_wire_hops={} landauer_hops={}/{}/{} gossip_hops={}/{} \
         verify_posture={} production_wired=false map_honest={} wire_honest={} \
         cd_fixture_honest={} landauer_adopt_honest={} ac373_absorbed={} \
         master_retick_eligible=false absorbed_ac373={} posture={}",
        probe.job_id,
        SLOT_CYCLE_REMAINDER,
        probe.band_l_floor,
        probe.band_l_ceiling,
        probe.cluster_letter,
        census.authority_present,
        GATE_WIRE_HOPS_CLOSED,
        landauer_ucrs_wire_hops_closed_count(),
        LANDAUER_WIRE_HOPS_CLOSED,
        UARCS_004_WIRE_HOPS_CLOSED,
        uarcs_gossip_mesh_wire_hops_closed_count(),
        GOSSIP_WIRE_HOPS_CLOSED,
        probe.verify_posture,
        census.map_honest,
        census.wire_honest,
        census.cd_fixture_honest,
        census.landauer_adopt_honest,
        census.ac373_absorbed,
        ABSORBED_AC373_RECEIPT,
        POSTURE_TAG,
    ))
}

#[test]
fn ac392_probe_honest_fences_hold() {
    let probe = ucrs_ac392_probe_honest();
    assert_eq!(probe.job_id, JOB_ID);
    assert_eq!(probe.receipt_path, RECEIPT_PATH);
    assert_eq!(u16::from(SLOT_CYCLE_REMAINDER), 392 % 12);
    assert_eq!(probe.band_l_floor, BAND_L_SLOT_FLOOR);
    assert_eq!(probe.band_l_ceiling, BAND_L_SLOT_CEILING);
    assert_eq!(probe.cluster_letter, TARGET_CLUSTER);
    assert_eq!(probe.verify_posture, VERIFY_POSTURE);
    assert!(!probe.production_wired);
    assert!(!probe.master_retick_eligible);
    assert!(!ucrs_ac392_production_wired());
    assert!(!ucrs_ac392_master_retick_eligible());
    assert_eq!(probe.gate_wire_hops_closed, 3);
    assert_eq!(probe.gate_wire_hops_open, 1);
}

#[test]
fn ac392_band_l_slot_honest() {
    assert!((BAND_L_SLOT_FLOOR..=BAND_L_SLOT_CEILING).contains(&392));
    assert_eq!(392 % 12, u16::from(SLOT_CYCLE_REMAINDER));
    assert_eq!(TARGET_CLUSTER, 'I');
}

#[test]
fn ac392_gate_wire_hops_three_of_four_closed() {
    assert!(ucrs_gate_wire_hops_honest());
    assert_eq!(ucrs_gate_wire_hops_closed_count(), 3);
    assert_eq!(UCRS_GATE_OPEN_HOP_SURFACES, ["p2p_daemon_production"]);
    assert!(!UCRS_GATE_WIRE_HOPS[3].wired_default);
}

#[test]
fn ac392_gate_cd_conjunct_matches_umst_math_ssot() {
    assert!(ucrs_gate_cd_conjunct_honest());
    assert!(clausius_duhem_admissible(10.0, 5.0));
    assert!(!clausius_duhem_admissible(5.0, 10.0));
}

#[test]
fn ac392_gated_sync_monotone_cost_honest() {
    assert!(ucrs_gate_gated_sync_monotone_honest());
}

#[test]
fn ac392_gate_cd_fixture_vectors_honest() {
    let root = workspace_root();
    assert!(ucrs_gate_cd_fixture_honest(&root));
    let fixture = root.join("umst-ucrs/Rust/fixtures/ucrs_gate_cd_vectors.json");
    assert!(fixture.is_file(), "CD fixture must exist");
}

#[test]
fn ac392_gate_fixture_authorities_pinned() {
    let root = workspace_root();
    for authority in [
        GATE_AUTHORITY,
        GATE_CD_FIXTURE_AUTHORITY,
        GATE_FULL_FIXTURE_AUTHORITY,
    ] {
        assert!(root.join(authority).is_file(), "missing {authority}");
    }
    let gate_src = fs::read_to_string(root.join(GATE_AUTHORITY)).expect("gate.rs");
    assert!(gate_src.contains("clausius_duhem_admissible"));
    assert!(gate_src.contains("gate_check"));
}

#[test]
fn ac392_landauer_and_deepen_wire_hops_honest() {
    assert!(landauer_ucrs_wire_hops_honest());
    assert_eq!(landauer_ucrs_wire_hops_closed_count(), LANDAUER_WIRE_HOPS_CLOSED);
    assert!(landauer_ucrs_pairwise_adopt_closed());
    assert!(uarcs_004_policy_present_wire_hops_honest());
    assert!(uarcs_a7_4_policy_wire_hops_honest());
    assert!(uarcs_gossip_mesh_wire_hops_honest());
    assert!(landauer_ucrs_adopt_honest());
}

#[test]
fn ac392_no_production_wired_invent() {
    assert!(!ucrs_ac392_production_wired());
    assert!(!uarcs_004_present_production_wired());
    assert!(!uarcs_a7_4_production_wired());
    assert!(!ucrs_gossip_mesh_production_wired());
}

#[test]
fn ac392_ac373_band_l_residue_absorb_honest() {
    let root = workspace_root();
    let v_ucrs = root.join("crates/umst-meta/src/v_ucrs.rs");
    if v_ucrs.is_file() {
        let text = fs::read_to_string(&v_ucrs).expect("v_ucrs.rs");
        assert!(text.contains("V-UCRS"));
        assert!(text.contains("production_wired"));
        assert!(text.contains("AC373"));
    }
    let wire_plan = root.join(UCRS_CONSUMER_WIRE_PLAN);
    if wire_plan.is_file() {
        let text = fs::read_to_string(&wire_plan).expect("wire plan");
        assert!(text.contains("gate::gate_check"));
        assert!(text.contains("production_wired"));
    }
    let probe = ucrs_ac392_probe_honest();
    assert!(!probe.master_retick_eligible);
    assert!(!probe.production_wired);
}

#[test]
fn ac392_head_census_clean() {
    let root = workspace_root();
    let census = verify_ucrs_ac392(&root).expect("HEAD census");
    assert!(census.authority_present);
    assert!(census.map_honest);
    assert!(census.wire_honest);
    assert!(census.cd_fixture_honest);
    assert!(census.landauer_adopt_honest);
    assert!(census.ac373_absorbed);
    assert!(!census.production_wired);
    assert!(!census.master_retick_eligible);
}

#[test]
fn ac392_probe_line_honest() {
    let root = workspace_root();
    let msg = probe_ucrs_ac392(&root).expect("probe");
    assert!(msg.contains("production_wired=false"), "fence: {msg}");
    assert!(msg.contains("master_retick_eligible=false"), "retick: {msg}");
    assert!(msg.contains("cluster=I"), "cluster: {msg}");
    assert!(msg.contains("band_l=369..=408"), "band: {msg}");
    assert!(msg.contains("verify_posture=CENSUS_ONLY"), "verify: {msg}");
    assert!(msg.contains("gate_wire_hops=3/4"), "hops: {msg}");
    assert!(msg.contains(ABSORBED_AC373_RECEIPT), "ac373: {msg}");
    assert!(msg.contains("honest-gate-census-only"), "posture: {msg}");
}

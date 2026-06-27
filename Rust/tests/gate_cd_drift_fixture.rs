//! U1 — Clausius–Duhem drift fixture: ucrs gate + umst-math SSOT vs shared vectors.

use std::fs;
use std::path::PathBuf;

use umst_math::clausius_duhem_admissible;
use umst_ucrs::gate::{gate_check, ClockThermState, GateVerdict};
use umst_ucrs::landauer;

#[derive(serde::Deserialize)]
struct CdVector {
    old_psi: f64,
    new_psi: f64,
    expect_admissible: bool,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/ucrs_gate_cd_vectors.json")
}

#[test]
fn gate_cd_drift_fixture_matches_umst_math_ssot() {
    let raw = fs::read_to_string(fixture_path()).expect("fixture json");
    let vectors: Vec<CdVector> = serde_json::from_str(&raw).expect("parse fixture");
    assert!(!vectors.is_empty(), "fixture must be non-empty");
    for (i, v) in vectors.iter().enumerate() {
        let got = clausius_duhem_admissible(v.old_psi, v.new_psi);
        assert_eq!(
            got, v.expect_admissible,
            "vector {i}: clausius_duhem_admissible({}, {})",
            v.old_psi, v.new_psi
        );
    }
}

#[test]
fn ucrs_gate_uses_cd_conjunct_on_desync_energy() {
    let t = 300.0;
    let state = ClockThermState {
        desync_energy_j: landauer::desync_energy(5.0, t),
        budget_j: landauer::landauer_cost(10.0, t),
        temperature_k: t,
        total_sync_cost_j: 0.0,
    };
    // Resolving 3 bits reduces ψ — CD admits
    assert_eq!(gate_check(&state, 3.0), GateVerdict::Admit);
    // Over-budget rejects before CD
    assert_eq!(gate_check(&state, 15.0), GateVerdict::Reject);
}

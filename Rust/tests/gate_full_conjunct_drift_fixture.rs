//! Full four-conjunct gate drift: umst-math gate_sdf ↔ formal thermo snapshots.

use std::fs;
use std::path::PathBuf;

use umst_math::manifold::csg::{gate_sdf, ThermoGateState};

#[derive(serde::Deserialize, Clone)]
struct ThermoStateWire {
    density: f64,
    free_energy: f64,
    hydration: f64,
    strength: f64,
    max_strength: f64,
}

#[derive(serde::Deserialize)]
struct GateVector {
    old: ThermoStateWire,
    new: ThermoStateWire,
    dt: f64,
    expect_admissible: bool,
}

fn to_state(w: ThermoStateWire) -> ThermoGateState {
    ThermoGateState {
        density: w.density,
        free_energy: w.free_energy,
        hydration: w.hydration,
        strength: w.strength,
        max_strength: w.max_strength,
    }
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/ucrs_gate_full_vectors.json")
}

#[test]
fn gate_full_conjunct_drift_fixture_matches_gate_sdf() {
    let raw = fs::read_to_string(fixture_path()).expect("fixture json");
    let vectors: Vec<GateVector> = serde_json::from_str(&raw).expect("parse fixture");
    assert!(!vectors.is_empty(), "fixture must be non-empty");
    const TOL: f64 = 1e-6;
    for (i, v) in vectors.iter().enumerate() {
        let g = gate_sdf(&to_state(v.old.clone()), &to_state(v.new.clone()));
        let admissible = g <= TOL;
        assert_eq!(
            admissible, v.expect_admissible,
            "vector {i}: gate_sdf={g} expect={}",
            v.expect_admissible
        );
    }
}

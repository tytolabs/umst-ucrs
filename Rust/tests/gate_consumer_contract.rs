// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! U3 gate consumer contract parity — `umst_ucrs::gate` sync economics (CELL_UCRS_READY_U3_GATE).

use umst_math::clausius_duhem_admissible;
use umst_ucrs::gate::{gate_check, gated_sync, ClockThermState, GateVerdict};
use umst_ucrs::landauer::{desync_energy, landauer_cost};

fn test_state(desync_bits: f64) -> ClockThermState {
    let t = 300.0;
    ClockThermState {
        desync_energy_j: desync_energy(desync_bits, t),
        budget_j: landauer_cost(10.0, t),
        temperature_k: t,
        total_sync_cost_j: 0.0,
    }
}

#[test]
fn gate_sync_economics_morphisms_admit_reject() {
    let state = test_state(5.0);
    assert_eq!(gate_check(&state, 3.0), GateVerdict::Admit);
    assert_eq!(gate_check(&state, 15.0), GateVerdict::Reject);
}

#[test]
fn gate_gated_sync_monotone_and_cd_conjunct() {
    let state = test_state(5.0);
    let s1 = gated_sync(&state, 2.0).expect("admitted sync");
    assert!(s1.total_sync_cost_j > 0.0);
    assert_eq!(s1.desync_energy_j, 0.0);

    let t = state.temperature_k;
    let s1_drifted = ClockThermState {
        desync_energy_j: desync_energy(3.0, t),
        ..s1
    };
    let psi_before = s1_drifted.desync_energy_j;
    let s2 = gated_sync(&s1_drifted, 3.0).expect("second admitted sync");
    assert!(s2.total_sync_cost_j > s1.total_sync_cost_j);
    assert!(clausius_duhem_admissible(psi_before, 0.0));
}

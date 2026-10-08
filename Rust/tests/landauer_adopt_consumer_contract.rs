// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! U6 landauer_adopt consumer contract parity — `umst_ucrs::landauer_adopt` bench witness lane
//! (CELL_UCRS_READY_U6_LANDAUER_ADOPT).

use umst_ucrs::landauer_adopt::{
    landauer_ucrs_adopt_honest, landauer_ucrs_pairwise_adopt_closed,
    landauer_ucrs_wire_hops_closed_count, landauer_ucrs_wire_hops_honest,
    lib_adopt_a_landauer_accel_ac36_probe, lib_adopt_a_landauer_p1542_b4_probe,
    lib_adopt_a_landauer_p1938_k3_probe, WIRE_HOPS_CLOSED_DEFAULT, WIRE_HOP_COUNT,
};

#[test]
fn landauer_adopt_witness_morphisms_reachable() {
    assert!(landauer_ucrs_wire_hops_honest());
    assert_eq!(
        landauer_ucrs_wire_hops_closed_count(),
        WIRE_HOPS_CLOSED_DEFAULT
    );
    assert!(landauer_ucrs_pairwise_adopt_closed());
    assert!(landauer_ucrs_adopt_honest());
    assert_eq!(WIRE_HOP_COUNT, 4);
}

#[test]
fn landauer_adopt_probe_snapshots_honest() {
    let b4 = lib_adopt_a_landauer_p1542_b4_probe();
    assert!(b4.pairwise_symbols_wired);
    assert_eq!(b4.wire_hops_closed, WIRE_HOPS_CLOSED_DEFAULT);
    assert!(b4.is_thermodynamic_floor);
    assert!(!b4.is_wall_clock);

    let k3 = lib_adopt_a_landauer_p1938_k3_probe();
    assert!(k3.pairwise_adopt_closed);

    let ac36 = lib_adopt_a_landauer_accel_ac36_probe();
    assert!(ac36.pairwise_mi_entropy_bridge_wired);
    assert!(ac36.pairwise_adopt_closed);
}

#[cfg(feature = "a7-4")]
#[test]
fn landauer_adopt_a7_4_mi_wired_when_feature_on() {
    use umst_ucrs::landauer_adopt::landauer_ucrs_a7_4_mi_wired;
    assert!(landauer_ucrs_a7_4_mi_wired());
}

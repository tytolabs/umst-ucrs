// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! U4 landauer_global consumer contract parity — `umst_ucrs::ucrs_keep::coordination_cost_global`
//! (CELL_UCRS_READY_U4_LANDAUER_GLOBAL · feature `a7-4`).

#![cfg(feature = "a7-4")]

use umst_ucrs::landauer_global::{
    multi_information_bits, n2_global_joules_matches_pairwise, n2_global_matches_pairwise_ssot,
};
use umst_ucrs::ucrs_keep::coordination_cost_global;

const T_ROOM: f64 = 300.0;

#[test]
fn landauer_global_mi_morphisms_reachable() {
    let h_x = 4.0;
    let h_y = 3.0;
    let i_xy = 1.5;
    let joint = h_x + h_y - i_xy;
    let mi = multi_information_bits(joint, &[h_x, h_y]).expect("valid n=2");
    assert!((mi - i_xy).abs() < f64::EPSILON);
    assert!(coordination_cost_global(mi, T_ROOM) > 0.0);
}

#[test]
fn landauer_global_n2_reduction_matches_pairwise_ssot() {
    let h_x = 4.0;
    let h_y = 3.0;
    let i_xy = 1.5;
    assert!(n2_global_matches_pairwise_ssot(h_x, h_y, i_xy));
    assert!(n2_global_joules_matches_pairwise(i_xy, T_ROOM));
}

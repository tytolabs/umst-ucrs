// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! U4 clock consumer contract parity — `umst_ucrs::ucrs_keep::LocalClock` (CELL_UCRS_READY_U4_CLOCK).

use umst_ucrs::ucrs_keep::LocalClock;

#[test]
fn clock_local_oscillator_morphisms_reachable() {
    let mut clock = LocalClock::new(10.0, 300.0);
    assert_eq!(clock.phase_entropy_bits(), 0.0);
    assert_eq!(clock.desync_energy_joules(), 0.0);

    clock.phase_uncertainty_sec = 10e-9;
    let bits = clock.phase_entropy_bits();
    assert!(bits > 0.0);
    assert!(clock.desync_energy_joules() > 0.0);
    assert!(clock.time_since_sync().as_secs() < 5);
}

#[test]
fn clock_sync_reset_and_landauer_floor() {
    let mut clock = LocalClock::new(10.0, 300.0);
    clock.phase_uncertainty_sec = 100e-9;
    let e_before = clock.desync_energy_joules();
    assert!(e_before > 0.0);

    clock.record_sync();
    assert_eq!(clock.phase_uncertainty_sec, 0.0);
    assert_eq!(clock.desync_energy_joules(), 0.0);

    clock.update_uncertainty();
    assert!(clock.predicted_error_at(1.0) >= 0.0);
}

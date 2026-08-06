// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! FLEET-COMPOSER-F F64/F86 — gate-guarded gossip outbound→inbound cycle (no libp2p).
//!
//! Absorbs E64 unit probe; exercises `p2p::{outbound_tick_if_admitted, apply_gated_inbound}`
//! across distinct publisher/receiver peers without network I/O.

use umst_ucrs::clock::LocalClock;
use umst_ucrs::credit::CreditLedger;
use umst_ucrs::gossip_mesh_census::{
    uarcs_gossip_mesh_honest, ucrs_gossip_mesh_c82_probe, ucrs_gossip_mesh_c82_residue_honest,
    ucrs_gossip_mesh_closed, ucrs_gossip_mesh_policy_wired, ucrs_gossip_mesh_production_wired,
    ucrs_gossip_mesh_wired, ucrs_gossip_prep_closed, ucrs_gossip_prep_wired,
    UARCS_GOSSIP_MESH_OPEN_HOP_SURFACES, WIRE_HOPS_CLOSED_DEFAULT,
};
use umst_ucrs::p2p::{
    apply_gated_inbound, gate_check_before_sync, localhost_mesh_bootstrap, localhost_peer_config,
    outbound_tick_if_admitted, ABSORBED_E64_SECRET, FLEET_COMPOSER_F64_JOB_ID,
    FLEET_COMPOSER_F64_RECEIPT_PATH, FLEET_COMPOSER_F86_JOB_ID, FLEET_COMPOSER_F86_RECEIPT_PATH,
    FLEET_COMPOSER_G86_JOB_ID, FLEET_COMPOSER_G86_RECEIPT_PATH, GatedSyncOutcome,
    LOCALHOST_MESH_PORTS,
};
use umst_ucrs::wire::{self, verify_tick, ClockTick, MergeOutcome};
use umst_ucrs::{gate::GateVerdict, AgentConfig};

#[test]
fn gossip_cycle_outbound_inbound_respects_gate() {
    let mut clock_out = LocalClock::new(10.0, 300.0);
    clock_out.phase_uncertainty_sec = 1e-6;
    let mut clock_in = LocalClock::new(10.0, 300.0);
    clock_in.phase_uncertainty_sec = 1e-6;
    let mut ledger = CreditLedger::new(2, 300.0);
    ledger.add_peer(1, 5.0);
    let publisher = localhost_peer_config(0, 300.0);
    let receiver = localhost_peer_config(1, 300.0);

    let tick = outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET);
    assert!(tick.is_some(), "outbound tick admitted under budget");
    let tick = tick.unwrap();
    assert_eq!(tick.agent_id, publisher.peer_id);

    let outcome = apply_gated_inbound(
        &mut clock_in,
        &mut ledger,
        &receiver,
        &tick,
        ABSORBED_E64_SECRET,
    );
    assert_eq!(outcome, GatedSyncOutcome::Admitted(MergeOutcome::Accepted));
}

#[test]
fn gossip_cycle_second_round_monotonic() {
    let mut clock_out = LocalClock::new(10.0, 300.0);
    clock_out.phase_uncertainty_sec = 1e-6;
    let mut clock_in = LocalClock::new(10.0, 300.0);
    clock_in.phase_uncertainty_sec = 1e-6;
    let mut ledger = CreditLedger::new(2, 300.0);
    ledger.add_peer(1, 5.0);
    let publisher = localhost_peer_config(0, 300.0);
    let receiver = localhost_peer_config(1, 300.0);

    let t1 = outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET)
        .expect("first outbound");
    assert_eq!(
        apply_gated_inbound(&mut clock_in, &mut ledger, &receiver, &t1, ABSORBED_E64_SECRET),
        GatedSyncOutcome::Admitted(MergeOutcome::Accepted)
    );

    // Receiver accumulates drift before the second gossip round.
    clock_out.phase_uncertainty_sec = 5e-6;
    clock_in.phase_uncertainty_sec = 5e-6;
    let t2 = outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET)
        .expect("second outbound");
    assert_eq!(
        apply_gated_inbound(&mut clock_in, &mut ledger, &receiver, &t2, ABSORBED_E64_SECRET),
        GatedSyncOutcome::Admitted(MergeOutcome::Accepted)
    );
}

#[test]
fn gossip_cycle_rejects_over_budget_inbound() {
    let clock = LocalClock::new(10.0, 300.0);
    let mut clock_mut = LocalClock::new(10.0, 300.0);
    let mut ledger = CreditLedger::new(1, 300.0);
    let config = AgentConfig {
        budget_bits: 2.0,
        ..AgentConfig::default()
    };
    let mut tick = ClockTick {
        agent_id: 2,
        phase_entropy_bits: 50.0,
        landauer_cost_j: 0.0,
        accuracy_score: 0.9,
        sig: [0; 32],
    };
    wire::sign_tick(b"gate-reject", &mut tick);
    assert_eq!(
        gate_check_before_sync(&clock, &config, &tick),
        GateVerdict::Reject
    );
    assert_eq!(
        apply_gated_inbound(&mut clock_mut, &mut ledger, &config, &tick, b"gate-reject"),
        GatedSyncOutcome::RejectedByGate
    );
}

#[test]
fn gossip_cycle_localhost_mesh_bootstrap() {
    assert_eq!(LOCALHOST_MESH_PORTS.len(), 3);
    let b1 = localhost_mesh_bootstrap(1);
    assert_eq!(b1.len(), 2);
    assert!(b1.iter().all(|a| a.contains("4001") || a.contains("4003")));
}

#[test]
fn gossip_cycle_f64_metadata() {
    assert_eq!(FLEET_COMPOSER_F64_JOB_ID, "FLEET-COMPOSER-F64-UCRS-GOSSIP");
    assert!(FLEET_COMPOSER_F64_RECEIPT_PATH.contains("COMPOSER_F64_UCRS_1934"));
}

#[test]
fn gossip_cycle_f86_metadata() {
    assert_eq!(FLEET_COMPOSER_F86_JOB_ID, "FLEET-COMPOSER-F86-UCRS-GOSSIP");
    assert!(FLEET_COMPOSER_F86_RECEIPT_PATH.contains("COMPOSER_F86_UCRS_GOSSIP_1942"));
}

#[test]
fn gossip_cycle_g86_metadata() {
    assert_eq!(FLEET_COMPOSER_G86_JOB_ID, "FLEET-COMPOSER-G86-UCRS-GOSSIP");
    assert!(FLEET_COMPOSER_G86_RECEIPT_PATH.contains("COMPOSER_G86_UCRS_GOSSIP_2143"));
}

#[test]
fn gossip_cycle_outbound_tick_verifies_signature() {
    let mut clock_out = LocalClock::new(10.0, 300.0);
    clock_out.phase_uncertainty_sec = 1e-6;
    let publisher = localhost_peer_config(0, 300.0);
    let tick = outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET)
        .expect("outbound admitted");
    assert!(verify_tick(ABSORBED_E64_SECRET, &tick));
    assert!(!verify_tick(b"wrong-secret", &tick));
}

#[test]
fn gossip_cycle_outbound_none_when_entropy_below_floor() {
    let clock_out = LocalClock::new(10.0, 300.0);
    let publisher = localhost_peer_config(0, 300.0);
    assert!(
        outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET).is_none(),
        "outbound must refuse when phase entropy is below the 0.1-bit publish floor"
    );
}

#[test]
fn gossip_cycle_wrong_secret_rejects_inbound() {
    let mut clock_out = LocalClock::new(10.0, 300.0);
    clock_out.phase_uncertainty_sec = 1e-6;
    let mut clock_in = LocalClock::new(10.0, 300.0);
    clock_in.phase_uncertainty_sec = 1e-6;
    let mut ledger = CreditLedger::new(2, 300.0);
    ledger.add_peer(1, 5.0);
    let publisher = localhost_peer_config(0, 300.0);
    let receiver = localhost_peer_config(1, 300.0);
    let tick = outbound_tick_if_admitted(&clock_out, &publisher, ABSORBED_E64_SECRET)
        .expect("outbound admitted");
    assert_eq!(
        apply_gated_inbound(&mut clock_in, &mut ledger, &receiver, &tick, b"wrong-secret"),
        GatedSyncOutcome::Admitted(MergeOutcome::RejectedBadSig)
    );
}

#[test]
fn gossip_cycle_honesty_fences_mesh_not_wired() {
    assert!(!ucrs_gossip_mesh_wired());
    assert!(!ucrs_gossip_mesh_policy_wired());
    assert!(!ucrs_gossip_mesh_production_wired());
    assert!(!ucrs_gossip_mesh_closed());
    let probe = ucrs_gossip_mesh_c82_probe();
    assert!(!probe.mesh_wired);
    assert!(!probe.mesh_policy_wired);
    assert!(!probe.production_wired);
    assert!(!probe.mesh_closed);
    assert_eq!(probe.ucrs_hops_closed, WIRE_HOPS_CLOSED_DEFAULT);
    assert_eq!(UARCS_GOSSIP_MESH_OPEN_HOP_SURFACES.len(), 2);
}

#[test]
fn gossip_cycle_prep_closed_without_daemon() {
    assert!(ucrs_gossip_prep_wired());
    assert!(ucrs_gossip_prep_closed());
    assert!(ucrs_gossip_mesh_c82_residue_honest());
    assert!(uarcs_gossip_mesh_honest());
}

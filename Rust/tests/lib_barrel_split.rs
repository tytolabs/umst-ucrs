// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! U1 lib barrel split parity — `shared_types` + `ucrs_keep` consumer fence (CELL_UCRS_READY_U1_LIB).

use umst_ucrs::shared_types::observation::{TemporalWitness, UcrsObservedAt};
use umst_ucrs::ucrs_keep::{agent_tick, witness_for_agent, AgentConfig, CreditLedger, LocalClock};

#[test]
fn shared_types_observation_morphisms_reachable() {
    let at = UcrsObservedAt::wall_only();
    assert!(at.wall_ms.unwrap_or(0) > 0);
    let mut witness = TemporalWitness::new(1);
    let stamped = witness.stamp();
    assert!(stamped.ucrs_seq.unwrap_or(0) > 0);
}

#[test]
fn ucrs_keep_agent_morphisms_roundtrip() {
    let config = AgentConfig::default();
    let mut witness = witness_for_agent(&config);
    let stamped = witness.stamp();
    assert!(stamped.wall_ms.unwrap_or(0) > 0);

    let mut clock = LocalClock::new(10.0, 300.0);
    let mut ledger = CreditLedger::new(1, 300.0);
    ledger.add_peer(2, 5.0);
    ledger.record_sync(2, 5.0, true);
    clock.phase_uncertainty_sec = 1e-6;
    clock.last_sync = std::time::Instant::now() - std::time::Duration::from_secs(100);

    let result = agent_tick(&mut clock, &mut ledger, &config);
    assert!(result.is_some());
}

#[test]
fn root_compat_shim_still_exports_agent_config() {
    let config = umst_ucrs::AgentConfig::default();
    assert_eq!(config.peer_id, 1);
}

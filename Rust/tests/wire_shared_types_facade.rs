// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! U1 wire facade parity — `shared_types::wire` consumer contract (CELL_UCRS_READY_U1_WIRE).

use umst_ucrs::shared_types::wire::{sign_tick, verify_tick, ClockTick, MergeOutcome};

#[test]
fn shared_types_wire_stamp_morphisms_roundtrip() {
    let mut tick = ClockTick {
        agent_id: 42,
        phase_entropy_bits: 2.5,
        landauer_cost_j: 1e-12,
        accuracy_score: 0.9,
        sig: [0; 32],
    };
    sign_tick(b"u1-facade", &mut tick);
    assert!(verify_tick(b"u1-facade", &tick));

    let json = serde_json::to_string(&tick).expect("ClockTick serializes");
    let back: ClockTick = serde_json::from_str(&json).expect("ClockTick deserializes");
    assert_eq!(tick, back);
    assert!(verify_tick(b"u1-facade", &back));
}

#[test]
fn shared_types_wire_merge_outcome_variants_exist() {
    assert_eq!(MergeOutcome::Accepted, MergeOutcome::Accepted);
    assert_ne!(MergeOutcome::RejectedBadSig, MergeOutcome::RejectedGate);
    assert_ne!(MergeOutcome::RejectedSelf, MergeOutcome::Accepted);
}

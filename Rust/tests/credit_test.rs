// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Integration tests for thermodynamic credit ledger (greedy, convergence, Byzantine).

use umst_ucrs::credit::CreditLedger;
use umst_ucrs::landauer;

#[test]
fn greedy_credit_selects_highest_credit_peer() {
    let mut ledger = CreditLedger::new(0, 300.0);
    ledger.add_peer(1, 5.0);
    ledger.add_peer(2, 50.0);
    ledger.add_peer(3, 12.0);

    ledger.record_sync(1, 8.0, true);
    ledger.record_sync(1, 4.0, true);
    ledger.record_sync(2, 2.0, true);
    ledger.record_sync(3, 10.0, true);

    let decision = ledger.best_peer().expect("peer available");
    assert_eq!(decision.peer_id, 1, "greedy selects max credit (peer 1)");
}

#[test]
fn credit_converges_on_repeated_good_syncs() {
    let mut ledger = CreditLedger::new(0, 300.0);
    ledger.add_peer(1, 5.0);

    let mut prev = 0.0_f64;
    for i in 1..=20 {
        ledger.record_sync(1, 2.0, true);
        let credit = ledger.peers[&1].credit_bits;
        assert!(
            credit >= prev,
            "credit should be non-decreasing on good syncs (round {i})"
        );
        prev = credit;
    }
    assert!(
        ledger.peers[&1].credit_bits >= 40.0,
        "20 good syncs should accumulate meaningful credit"
    );
    assert!(
        ledger.peers[&1].accuracy_score > 0.5,
        "accuracy should remain healthy after good syncs"
    );
}

#[test]
fn byzantine_peer_isolated_via_credit_collapse() {
    let mut ledger = CreditLedger::new(0, 300.0);
    ledger.add_peer(1, 5.0);
    ledger.add_peer(2, 5.0);

    for _ in 0..10 {
        ledger.record_sync(1, 3.0, true);
    }
    for _ in 0..10 {
        ledger.record_sync(2, 3.0, false);
    }

    let suspects = ledger.suspect_peers(0.5);
    assert!(suspects.contains(&2), "Byzantine peer 2 flagged");
    assert!(!suspects.contains(&1), "Honest peer 1 not flagged");

    let greedy = ledger.best_peer().expect("honest peer selectable");
    assert_eq!(greedy.peer_id, 1);
    assert!(
        ledger.peers[&2].credit_bits < ledger.peers[&1].credit_bits,
        "Byzantine credit should trail honest peer"
    );
}

#[test]
fn total_landauer_cost_matches_bits_received() {
    let mut ledger = CreditLedger::new(0, 300.0);
    ledger.add_peer(1, 5.0);
    ledger.record_sync(1, 12.5, true);

    let cost = ledger.total_network_cost_joules();
    let expected = landauer::landauer_cost(12.5, 300.0);
    assert!((cost - expected).abs() < 1e-12);
}

#[test]
fn degraded_peer_excluded_from_greedy_selection() {
    let mut ledger = CreditLedger::new(0, 300.0);
    ledger.add_peer(1, 5.0);
    ledger.add_peer(2, 5.0);

    ledger.record_sync(1, 1.0, true);
    for _ in 0..15 {
        ledger.record_sync(2, 2.0, false);
    }

    let decision = ledger.best_peer();
    assert!(
        decision.map(|d| d.peer_id) != Some(2),
        "degraded peer should not win greedy selection"
    );
}

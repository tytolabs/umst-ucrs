// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! The runtime refines the formal coordination contract.
//!
//! `umst-formal` proves the contract in Lean, Coq and Agda (`CoordinationContract`) and runs it as Haskell
//! properties; this suite states the same laws of the runtime's own functions, so a change here that breaks a
//! proved law fails. Floating arithmetic is compared with a relative tolerance only where the law is an equality
//! of sums; order laws hold exactly.
//!
//! | Law (umst-formal `CoordinationContract`) | Runtime function |
//! |---|---|
//! | `landauerCostJoules_nonneg`, `landauerCostJoules_add` | `landauer::landauer_cost` |
//! | `admits` (budget and Clausius–Duhem on desync) | `gate::gate_check` (both directions) |
//! | `admitted_cost_bounded` | `gate::gate_check` |
//! | `gatedSync_second_law` | `gate::gated_sync` |
//! | peer credit model (`bestPeer`, `recordSync`) | `credit::CreditLedger` |

use proptest::prelude::*;
use umst_ucrs::credit::CreditLedger;
use umst_ucrs::gate::{gate_check, gated_sync, ClockThermState, GateVerdict};
use umst_ucrs::landauer::landauer_cost;

fn kelvin() -> impl Strategy<Value = f64> {
    1.0f64..2000.0
}

fn bits() -> impl Strategy<Value = f64> {
    0.0f64..1.0e4
}

proptest! {
    #[test]
    fn landauer_cost_is_nonnegative(b in bits(), t in kelvin()) {
        prop_assert!(landauer_cost(b, t) >= 0.0);
    }

    #[test]
    fn landauer_cost_adds_over_bits(a in bits(), b in bits(), t in kelvin()) {
        let (sum, parts) = (landauer_cost(a + b, t), landauer_cost(a, t) + landauer_cost(b, t));
        prop_assert!((sum - parts).abs() <= 1e-12 * sum.abs().max(f64::MIN_POSITIVE));
    }

    #[test]
    fn gate_admits_exactly_the_contract(b in bits(), t in kelvin(), budget_bits in 0.0f64..2.0e4,
                                        desync_bits in 0.0f64..2.0e4) {
        let s = ClockThermState {
            desync_energy_j: landauer_cost(desync_bits, t),
            budget_j: landauer_cost(budget_bits, t),
            temperature_k: t,
            total_sync_cost_j: 0.0,
        };
        let cost = landauer_cost(b, t);
        let contract = cost <= s.budget_j && cost <= s.desync_energy_j;
        prop_assert_eq!(gate_check(&s, b) == GateVerdict::Admit, contract);
        if contract {
            prop_assert!(0.0 <= cost && cost <= s.budget_j && cost <= s.desync_energy_j);
        }
    }

    #[test]
    fn admitted_sync_obeys_the_second_law(b in bits(), t in kelvin(), slack_budget in 0.0f64..1.0e-18,
                                          slack_desync in 0.0f64..1.0e-18, spent in 0.0f64..1.0e-15) {
        // built inside admission: budget and desync are the cost plus a slack
        let cost = landauer_cost(b, t);
        let s = ClockThermState {
            desync_energy_j: cost + slack_desync,
            budget_j: cost + slack_budget,
            temperature_k: t,
            total_sync_cost_j: spent,
        };
        let next = gated_sync(&s, b).expect("a state built inside admission is admitted");
        prop_assert!(next.desync_energy_j <= s.desync_energy_j);
        prop_assert!(s.total_sync_cost_j <= next.total_sync_cost_j);
    }

    #[test]
    fn best_peer_has_the_highest_healthy_credit(credits in prop::collection::vec(0.0f64..100.0, 1..12)) {
        let mut ledger = CreditLedger::new(0, 300.0);
        for (i, c) in credits.iter().enumerate() {
            ledger.add_peer(i as u64 + 1, 1.0);
            ledger.peers.get_mut(&(i as u64 + 1)).expect("peer").credit_bits = *c;
        }
        let best = ledger.best_peer().expect("every peer starts healthy");
        prop_assert!(ledger.peers.values().all(|p| best.peer_credit_before >= p.credit_bits));
    }

    #[test]
    fn a_failed_sync_costs_credit(b in 0.001f64..100.0) {
        let mut ledger = CreditLedger::new(0, 300.0);
        ledger.add_peer(1, 1.0);
        ledger.record_sync(1, b, true);
        let up = ledger.peers[&1].credit_bits;
        ledger.record_sync(1, b, false);
        prop_assert!(ledger.peers[&1].credit_bits < up);
    }
}

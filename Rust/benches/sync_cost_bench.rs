// SPDX-License-Identifier: MIT
// Placeholder bench — will measure real sync energy costs.

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_landauer_cost(c: &mut Criterion) {
    c.bench_function("landauer_cost_1bit_300K", |b| {
        b.iter(|| umst_ucrs::landauer::landauer_cost(1.0, 300.0))
    });
}

fn bench_credit_selection(c: &mut Criterion) {
    c.bench_function("credit_best_peer_100_peers", |b| {
        let mut ledger = umst_ucrs::credit::CreditLedger::new(0, 300.0);
        for i in 1..=100 {
            ledger.add_peer(i, 5.0 + i as f64);
            ledger.record_sync(i, (i % 10) as f64 + 1.0, true);
        }
        b.iter(|| ledger.best_peer())
    });
}

criterion_group!(benches, bench_landauer_cost, bench_credit_selection);
criterion_main!(benches);

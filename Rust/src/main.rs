// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UMST-UCRS daemon binary.
//!
//! Thin wrapper around the `umst_ucrs` library. The core logic (clock, gate, credit
//! ledger, Landauer cost, telemetry) lives in `lib.rs` so it can be consumed by
//! downstream crates embedding the library.

use tracing::info;

use umst_ucrs::{clock::LocalClock, credit::CreditLedger, landauer, AgentConfig};

fn main() {
    tracing_subscriber::fmt::init();

    info!("UMST-UCRS daemon v{}", env!("CARGO_PKG_VERSION"));
    info!(
        "Landauer bit energy at 300K: {:.3e} J",
        landauer::landauer_bit_energy(300.0)
    );
    info!(
        "Mass equivalent per bit at 300K: {:.3e} kg",
        landauer::mass_equivalent_per_bit(300.0)
    );

    // TODO: Parse CLI args, start Tokio runtime, launch libp2p swarm (feature = "p2p").
    // For now, demonstrate the core loop.
    let config = AgentConfig::default();
    let _clock = LocalClock::new(config.drift_ppb, config.temperature_k);
    let mut ledger = CreditLedger::new(config.peer_id, config.temperature_k);

    ledger.add_peer(2, 5.0);
    ledger.record_sync(2, 1.0, true);

    info!(
        "Agent {} initialized. Drift: {} ppb, Budget: {} bits",
        config.peer_id, config.drift_ppb, config.budget_bits
    );
    info!("Run with --help for P2P network options (coming soon).");
}

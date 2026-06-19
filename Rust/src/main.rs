// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UMST-UCRS daemon binary.
//!
//! Thin wrapper around the `umst_ucrs` library. Parses CLI flags, optionally
//! starts the Prometheus metrics server, and runs the Tokio agent loop.

use std::time::Duration;

use clap::Parser;
use tracing::info;

use umst_ucrs::clock::LocalClock;
use umst_ucrs::credit::CreditLedger;
use umst_ucrs::landauer;
use umst_ucrs::telemetry;
use umst_ucrs::{agent_tick, AgentConfig};

/// UMST-UCRS thermodynamic clock daemon.
#[derive(Debug, Parser)]
#[command(
    name = "umst-ucrs",
    version,
    about = "UMST Universal Calendar Resolution Spine daemon"
)]
struct Cli {
    /// Local peer identifier in the credit ledger.
    #[arg(long, default_value_t = 1)]
    peer_id: u64,
    /// TCP listen port (P2P swarm when `p2p` feature enabled; logged in stub mode).
    #[arg(long, default_value_t = 4001)]
    port: u16,
    /// Bootstrap multiaddr for libp2p mesh (repeatable).
    #[arg(long = "bootstrap")]
    bootstrap: Vec<String>,
    /// Landauer sync budget per window (bits).
    #[arg(long, default_value_t = 20.0)]
    budget_bits: f64,
    /// Agent tick interval in seconds.
    #[arg(long, default_value_t = 60)]
    tick_secs: u64,
    /// Prometheus scrape bind address.
    #[arg(long, default_value = "0.0.0.0:9090")]
    metrics_addr: String,
    /// Disable Prometheus HTTP server even when `daemon` feature is enabled.
    #[arg(long)]
    no_metrics: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    info!("UMST-UCRS daemon v{}", env!("CARGO_PKG_VERSION"));
    info!(
        "Landauer bit energy at 300K: {:.3e} J",
        landauer::landauer_bit_energy(300.0)
    );

    let config = AgentConfig {
        peer_id: cli.peer_id,
        budget_bits: cli.budget_bits,
        sync_interval_sec: cli.tick_secs as f64,
        ..AgentConfig::default()
    };

    let mut clock = LocalClock::new(config.drift_ppb, config.temperature_k);
    let mut ledger = CreditLedger::new(config.peer_id, config.temperature_k);

    if !cli.bootstrap.is_empty() {
        info!(count = cli.bootstrap.len(), "bootstrap peers configured");
        for addr in &cli.bootstrap {
            info!(%addr, "bootstrap");
        }
    }

    info!(
        peer_id = config.peer_id,
        port = cli.port,
        budget_bits = config.budget_bits,
        "agent configured"
    );

    let metrics_enabled = !cli.no_metrics
        && (cfg!(feature = "prometheus")
            || cfg!(feature = "daemon")
            || std::env::var("UMST_UCRS_METRICS").is_ok());

    if metrics_enabled {
        let addr = cli.metrics_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = telemetry::serve_metrics(&addr).await {
                tracing::error!(%addr, ?e, "metrics server failed");
            }
        });
        info!(addr = %cli.metrics_addr, "Prometheus metrics on /metrics");
    }

    #[cfg(feature = "p2p")]
    {
        info!(
            port = cli.port,
            "P2P feature enabled — use `umst-ucrs-p2p` for full swarm"
        );
    }

    let mut interval = tokio::time::interval(Duration::from_secs(cli.tick_secs));
    loop {
        interval.tick().await;
        clock.update_uncertainty();

        let phase_entropy = clock.phase_entropy_bits();
        let desync_energy = clock.desync_energy_joules();
        telemetry::update_gauges(
            phase_entropy,
            desync_energy,
            ledger.peers.len(),
            ledger.total_credit(),
        );

        if let Some(record) = agent_tick(&mut clock, &mut ledger, &config) {
            umst_ucrs::rapl::export_sync_overhead(&record);
        }
    }
}

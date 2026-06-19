// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Prometheus-compatible metrics for UMST-UCRS.
//!
//! Exports real-time measurements for:
//! - Sync event count and frequency
//! - Landauer floor vs. actual energy (RAPL)
//! - Credit distribution across peers
//! - Desync energy accumulation
//! - Byzantine detection events

use prometheus::{
    register_counter, register_gauge, register_histogram, Counter, Gauge, Histogram, HistogramOpts,
};
use std::sync::LazyLock;

// --- Counters ---

pub static SYNC_EVENTS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "ucrs_sync_events_total",
        "Total number of clock sync events"
    )
    .unwrap()
});

pub static BITS_RESOLVED_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "ucrs_bits_resolved_total",
        "Total bits of phase uncertainty resolved across all syncs"
    )
    .unwrap()
});

pub static BYZANTINE_DETECTIONS: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "ucrs_byzantine_detections_total",
        "Number of peers flagged as potentially Byzantine"
    )
    .unwrap()
});

// --- Gauges ---

pub static DESYNC_ENERGY_JOULES: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_desync_energy_joules",
        "Current desync energy (Landauer cost to resolve phase uncertainty)"
    )
    .unwrap()
});

pub static PHASE_ENTROPY_BITS: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_phase_entropy_bits",
        "Current phase uncertainty in bits"
    )
    .unwrap()
});

pub static PEER_COUNT: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_peer_count",
        "Number of known peers in the credit ledger"
    )
    .unwrap()
});

pub static TOTAL_CREDIT: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_total_credit_bits",
        "Sum of all peer credits (should be roughly conserved)"
    )
    .unwrap()
});

// --- Histograms ---

pub static SYNC_COST_RATIO: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(HistogramOpts::new(
        "ucrs_sync_overhead_ratio",
        "Ratio of measured energy to Landauer floor per sync"
    )
    .buckets(vec![1.0, 10.0, 100.0, 1e3, 1e6, 1e9, 1e12]))
    .unwrap()
});

pub static SYNC_BITS_HISTOGRAM: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(HistogramOpts::new(
        "ucrs_sync_bits_per_event",
        "Bits resolved per sync event"
    )
    .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0]))
    .unwrap()
});

/// Record a sync event in all relevant metrics.
pub fn record_sync_event(record: &crate::rapl::SyncEnergyRecord) {
    SYNC_EVENTS_TOTAL.inc();
    BITS_RESOLVED_TOTAL.inc_by(record.bits_resolved);
    SYNC_BITS_HISTOGRAM.observe(record.bits_resolved);

    if let Some(ratio) = record.overhead_ratio {
        SYNC_COST_RATIO.observe(ratio);
    }
}

/// Update gauge metrics from current agent state.
pub fn update_gauges(phase_entropy: f64, desync_energy: f64, peer_count: usize, total_credit: f64) {
    PHASE_ENTROPY_BITS.set(phase_entropy);
    DESYNC_ENERGY_JOULES.set(desync_energy);
    PEER_COUNT.set(peer_count as f64);
    TOTAL_CREDIT.set(total_credit);
}

/// Increment the Byzantine detection counter (called from credit ledger).
pub fn record_byzantine_detection() {
    BYZANTINE_DETECTIONS.inc();
}

/// Render all registered metrics as Prometheus text exposition.
pub fn gather_text() -> String {
    // Ensure lazy-registered metrics appear in exposition even before first event.
    let _ = (
        &*SYNC_EVENTS_TOTAL,
        &*BITS_RESOLVED_TOTAL,
        &*BYZANTINE_DETECTIONS,
        &*DESYNC_ENERGY_JOULES,
        &*PHASE_ENTROPY_BITS,
        &*PEER_COUNT,
        &*TOTAL_CREDIT,
        &*SYNC_COST_RATIO,
        &*SYNC_BITS_HISTOGRAM,
    );
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("prometheus encode");
    String::from_utf8(buffer).expect("utf8 metrics")
}

/// Minimal HTTP/1.1 Prometheus scrape server on `addr` (e.g. `0.0.0.0:9090`).
///
/// Serves `GET /metrics` with `text/plain; version=0.0.4` body.
pub async fn serve_metrics(addr: &str) -> std::io::Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "prometheus metrics listening");

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let (status, body) = if req.starts_with("GET /metrics") || req.starts_with("GET / ") {
                ("200 OK", gather_text())
            } else {
                ("404 Not Found", String::new())
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_text_contains_sync_counter() {
        let _ = &*SYNC_EVENTS_TOTAL;
        let text = gather_text();
        assert!(text.contains("ucrs_sync_events_total"));
    }
}

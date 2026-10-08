// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
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

pub static SYNC_EVENTS_TOTAL: LazyLock<Option<Counter>> = LazyLock::new(|| {
    register_counter!(
        "ucrs_sync_events_total",
        "Total number of clock sync events"
    )
    .ok()
});

pub static BITS_RESOLVED_TOTAL: LazyLock<Option<Counter>> = LazyLock::new(|| {
    register_counter!(
        "ucrs_bits_resolved_total",
        "Total bits of phase uncertainty resolved across all syncs"
    )
    .ok()
});

pub static BYZANTINE_DETECTIONS: LazyLock<Option<Counter>> = LazyLock::new(|| {
    register_counter!(
        "ucrs_byzantine_detections_total",
        "Number of peers flagged as potentially Byzantine"
    )
    .ok()
});

// --- Gauges ---

pub static DESYNC_ENERGY_JOULES: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_desync_energy_joules",
        "Current desync energy (Landauer cost to resolve phase uncertainty)"
    )
    .ok()
});

pub static PHASE_ENTROPY_BITS: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_phase_entropy_bits",
        "Current phase uncertainty in bits"
    )
    .ok()
});

pub static PEER_COUNT: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_peer_count",
        "Number of known peers in the credit ledger"
    )
    .ok()
});

pub static TOTAL_CREDIT: LazyLock<Option<Gauge>> = LazyLock::new(|| {
    register_gauge!(
        "ucrs_total_credit_bits",
        "Sum of all peer credits (should be roughly conserved)"
    )
    .ok()
});

// --- Histograms ---

pub static SYNC_COST_RATIO: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    register_histogram!(HistogramOpts::new(
        "ucrs_sync_overhead_ratio",
        "Ratio of measured energy to Landauer floor per sync"
    )
    .buckets(vec![1.0, 10.0, 100.0, 1e3, 1e6, 1e9, 1e12]))
    .ok()
});

pub static SYNC_BITS_HISTOGRAM: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    register_histogram!(HistogramOpts::new(
        "ucrs_sync_bits_per_event",
        "Bits resolved per sync event"
    )
    .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0]))
    .ok()
});

/// Record a sync event in all relevant metrics.
pub fn record_sync_event(record: &crate::rapl::SyncEnergyRecord) {
    if let Some(c) = SYNC_EVENTS_TOTAL.as_ref() {
        c.inc();
    }
    if let Some(c) = BITS_RESOLVED_TOTAL.as_ref() {
        c.inc_by(record.bits_resolved);
    }
    if let Some(h) = SYNC_BITS_HISTOGRAM.as_ref() {
        h.observe(record.bits_resolved);
    }
    if let (Some(ratio), Some(h)) = (record.overhead_ratio, SYNC_COST_RATIO.as_ref()) {
        h.observe(ratio);
    }
}

/// Update gauge metrics from current agent state.
pub fn update_gauges(phase_entropy: f64, desync_energy: f64, peer_count: usize, total_credit: f64) {
    for (gauge, value) in [
        (&*PHASE_ENTROPY_BITS, phase_entropy),
        (&*DESYNC_ENERGY_JOULES, desync_energy),
        (&*PEER_COUNT, peer_count as f64),
        (&*TOTAL_CREDIT, total_credit),
    ] {
        if let Some(g) = gauge {
            g.set(value);
        }
    }
}

/// Increment the Byzantine detection counter (called from credit ledger).
pub fn record_byzantine_detection() {
    if let Some(c) = BYZANTINE_DETECTIONS.as_ref() {
        c.inc();
    }
}

/// Render all registered metrics as Prometheus text exposition.
///
/// # Errors
/// Returns the encoder error when the registry cannot be rendered.
pub fn gather_text() -> Result<String, prometheus::Error> {
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
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
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
                match gather_text() {
                    Ok(text) => ("200 OK", text),
                    Err(e) => (
                        "500 Internal Server Error",
                        format!("metrics encode failed: {e}\n"),
                    ),
                }
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
        assert!(SYNC_EVENTS_TOTAL.is_some());
        let text = gather_text();
        assert!(text.is_ok_and(|t| t.contains("ucrs_sync_events_total")));
    }

    #[test]
    fn gather_text_nonempty() {
        let text = gather_text();
        assert!(text.is_ok_and(|t| !t.is_empty() && t.contains("# HELP")));
    }
}

// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Immutable observation stamps for durable logs (`UcrsObservedAt`).
//!
//! Wire shape aligns with [`contribution.v1`](https://github.com/tytolabs/umst-concrete-cartridge/schemas/contribution.v1.json)
//! `observed_at` and [`Docs/LOGGING_POLICY.md`](../../Docs/LOGGING_POLICY.md).

use crate::clock::LocalClock;
use crate::credit::CreditLedger;
use crate::AgentConfig;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Fixed-point scale for entropy / credit fields on the public wire.
pub const WIRE_SCALE: i64 = 1_000_000;

/// Tier tag for observation stamps (never omit on durable writes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StampTier {
    UcrsTier2,
    WallOnly,
    Absent,
    Synthetic,
}

impl StampTier {
    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::UcrsTier2 => "UcrsTier2",
            Self::WallOnly => "WallOnly",
            Self::Absent => "Absent",
            Self::Synthetic => "Synthetic",
        }
    }
}

/// Canonical UCRS observation stamp (Tier-2 default for open contributors).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UcrsObservedAt {
    pub stamp_tier: StampTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucrs_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<u64>,
}

/// Integer-only `observed_at.v2` wire (matches cartridge `ObservedAtV2`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedAtV2Wire {
    pub schema_version: String,
    pub stamp_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ucrs_seq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase_entropy_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_q: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credit_head_bits_scale: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<u64>,
}

/// Minimum contributor credit (bits) required for inbox promote / promotion (U2 witness gate).
pub const MIN_PROMOTION_CREDIT_BITS: f64 = 1.0;

/// Decode fixed-point credit head from contribution `observed_at` wire fields.
#[must_use]
pub fn credit_head_bits_from_wire(q: Option<i64>, scale: Option<i64>) -> Option<f64> {
    let q = q?;
    let scale = scale?;
    if scale <= 0 {
        return None;
    }
    Some(q as f64 / scale as f64)
}

/// Credit admits promotion when head ≥ `min_bits` (Byzantine / low-trust quarantine otherwise).
#[must_use]
pub fn credit_admits_promotion(credit_bits: Option<f64>, min_bits: f64) -> bool {
    credit_bits.is_some_and(|c| c.is_finite() && c >= min_bits)
}

impl UcrsObservedAt {
    /// Wire v2 JSON shape (`observed_at.v2` integer fields — cartridge parity).
    #[must_use]
    pub fn to_v2_wire(&self) -> ObservedAtV2Wire {
        ObservedAtV2Wire {
            schema_version: "observed_at.v2".into(),
            stamp_tier: self.stamp_tier.as_wire_str().into(),
            ucrs_seq: self.ucrs_seq,
            phase_entropy_bits_q: self.phase_entropy_bits_q,
            phase_entropy_bits_scale: self.phase_entropy_bits_scale,
            credit_head_bits_q: self.credit_head_bits_q,
            credit_head_bits_scale: self.credit_head_bits_scale,
            wall_ms: self.wall_ms,
        }
    }

    /// Parse v2 wire JSON (shared fixture roundtrip with cartridge `wire_v2.rs`).
    pub fn from_v2_wire(w: &ObservedAtV2Wire) -> Self {
        let stamp_tier = match w.stamp_tier.as_str() {
            "UcrsTier2" => StampTier::UcrsTier2,
            "WallOnly" => StampTier::WallOnly,
            "Absent" => StampTier::Absent,
            _ => StampTier::Synthetic,
        };
        Self {
            stamp_tier,
            ucrs_seq: w.ucrs_seq,
            phase_entropy_bits_q: w.phase_entropy_bits_q,
            phase_entropy_bits_scale: w.phase_entropy_bits_scale,
            credit_head_bits_q: w.credit_head_bits_q,
            credit_head_bits_scale: w.credit_head_bits_scale,
            wall_ms: w.wall_ms,
        }
    }

    /// Wall-clock-only fallback when UCRS agent loop is unavailable.
    #[must_use]
    pub fn wall_only() -> Self {
        Self {
            stamp_tier: StampTier::WallOnly,
            ucrs_seq: None,
            phase_entropy_bits_q: None,
            phase_entropy_bits_scale: None,
            credit_head_bits_q: None,
            credit_head_bits_scale: None,
            wall_ms: Some(wall_epoch_ms()),
        }
    }

    /// Deterministic test / fixture stamp (isolated from production merge).
    #[must_use]
    pub fn synthetic(seq: u64, phase_entropy_bits: f64) -> Self {
        let q = (phase_entropy_bits * (WIRE_SCALE as f64)).round() as i64;
        Self {
            stamp_tier: StampTier::Synthetic,
            ucrs_seq: Some(seq),
            phase_entropy_bits_q: Some(q),
            phase_entropy_bits_scale: Some(WIRE_SCALE),
            credit_head_bits_q: Some(0),
            credit_head_bits_scale: Some(WIRE_SCALE),
            wall_ms: Some(wall_epoch_ms()),
        }
    }
}

/// Witness that binds a durable event to thermodynamic time.
#[derive(Debug)]
pub struct TemporalWitness {
    clock: LocalClock,
    ledger: CreditLedger,
    seq: u64,
}

impl TemporalWitness {
    /// Fresh witness with default drift / temperature.
    #[must_use]
    pub fn new(peer_id: u64) -> Self {
        Self {
            clock: LocalClock::new(10.0, 300.0),
            ledger: CreditLedger::new(peer_id, 300.0),
            seq: 0,
        }
    }

    /// Construct witness from agent configuration (total function on `AgentConfig`).
    #[must_use]
    pub fn from_agent(config: &AgentConfig) -> Self {
        Self {
            clock: LocalClock::new(config.drift_ppb, config.temperature_k),
            ledger: CreditLedger::new(config.peer_id, config.temperature_k),
            seq: 0,
        }
    }

    /// Advance local clock uncertainty and emit a Tier-2 stamp.
    pub fn stamp(&mut self) -> UcrsObservedAt {
        self.clock.update_uncertainty();
        self.seq = self.seq.saturating_add(1);
        let phase = self.clock.phase_entropy_bits();
        let phase_q = (phase * (WIRE_SCALE as f64)).round() as i64;
        let credit_head = self
            .ledger
            .peers
            .values()
            .map(|p| p.credit_bits)
            .fold(0.0_f64, f64::max);
        let credit_q = (credit_head * (WIRE_SCALE as f64)).round() as i64;
        UcrsObservedAt {
            stamp_tier: StampTier::UcrsTier2,
            ucrs_seq: Some(self.seq),
            phase_entropy_bits_q: Some(phase_q),
            phase_entropy_bits_scale: Some(WIRE_SCALE),
            credit_head_bits_q: Some(credit_q),
            credit_head_bits_scale: Some(WIRE_SCALE),
            wall_ms: Some(wall_epoch_ms()),
        }
    }
}

#[must_use]
pub fn wall_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn stamp_monotonic_seq() {
        let mut w = TemporalWitness::new(1);
        let a = w.stamp();
        let b = w.stamp();
        assert!(b.ucrs_seq.unwrap() > a.ucrs_seq.unwrap());
    }

    #[test]
    fn stamp_phase_nonzero_when_drift_forced() {
        let mut w = TemporalWitness::new(1);
        w.clock.phase_uncertainty_sec = 1e-6;
        w.clock.last_sync = std::time::Instant::now() - Duration::from_secs(100);
        let s = w.stamp();
        assert!(s.phase_entropy_bits_q.unwrap_or(0) > 0);
    }

    #[test]
    fn from_agent_stamps_tier2() {
        let config = AgentConfig::default();
        let mut w = TemporalWitness::from_agent(&config);
        let s = w.stamp();
        assert_eq!(s.stamp_tier, StampTier::UcrsTier2);
        assert_eq!(s.ucrs_seq, Some(1));
    }
}

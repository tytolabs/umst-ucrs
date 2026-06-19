// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Immutable observation stamps for durable logs (`UcrsObservedAt`).
//!
//! Wire shape aligns with [`contribution.v1`](https://github.com/tytolabs/umst-concrete-cartridge/schemas/contribution.v1.json)
//! `observed_at` and [`outputs/ucrs-logging-policy.md`](../../outputs/ucrs-logging-policy.md).

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

impl UcrsObservedAt {
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

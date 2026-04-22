// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Local oscillator model with drift tracking.
//!
//! Each agent maintains a local clock that drifts from the true reference.
//! Drift accumulates entropy (phase uncertainty), which has a measurable
//! Landauer cost to resolve via sync.

use crate::landauer;
use std::time::{Duration, Instant};

/// A local clock with drift model.
#[derive(Debug, Clone)]
pub struct LocalClock {
    /// When this clock was last synced to a reference.
    pub last_sync: Instant,
    /// Estimated drift rate in parts-per-billion (ppb).
    /// Typical quartz: 1-50 ppb. Typical TCXO: 0.1-5 ppb.
    pub drift_ppb: f64,
    /// Accumulated phase uncertainty since last sync (seconds).
    pub phase_uncertainty_sec: f64,
    /// Temperature of the compute environment (Kelvin).
    pub temperature_k: f64,
}

impl LocalClock {
    /// Create a new clock, freshly synced.
    pub fn new(drift_ppb: f64, temperature_k: f64) -> Self {
        Self {
            last_sync: Instant::now(),
            drift_ppb,
            phase_uncertainty_sec: 0.0,
            temperature_k,
        }
    }

    /// Update phase uncertainty based on elapsed time since last sync.
    ///
    /// Uncertainty grows linearly with time: δt · drift_ppb · 1e-9
    pub fn update_uncertainty(&mut self) {
        let elapsed = self.last_sync.elapsed().as_secs_f64();
        self.phase_uncertainty_sec = elapsed * self.drift_ppb * 1e-9;
    }

    /// Phase uncertainty converted to bits of entropy.
    ///
    /// Uses the Shannon entropy of a uniform distribution over the
    /// uncertainty interval: H = log2(uncertainty / resolution).
    /// Resolution = 1 nanosecond (typical NTP precision).
    pub fn phase_entropy_bits(&self) -> f64 {
        let resolution_sec = 1e-9; // 1 ns resolution
        if self.phase_uncertainty_sec <= resolution_sec {
            return 0.0; // already at measurement resolution
        }
        (self.phase_uncertainty_sec / resolution_sec).log2()
    }

    /// Desync energy: minimum thermodynamic cost to know the true phase.
    ///
    /// D(clock, T) = k_B T ln(2) · H(phase)
    pub fn desync_energy_joules(&self) -> f64 {
        landauer::desync_energy(self.phase_entropy_bits(), self.temperature_k)
    }

    /// Record a sync event: reset uncertainty, update timestamp.
    pub fn record_sync(&mut self) {
        self.phase_uncertainty_sec = 0.0;
        self.last_sync = Instant::now();
    }

    /// Time since last sync.
    pub fn time_since_sync(&self) -> Duration {
        self.last_sync.elapsed()
    }

    /// Predicted phase error at a future time (seconds from now).
    pub fn predicted_error_at(&self, seconds_ahead: f64) -> f64 {
        let total_elapsed = self.last_sync.elapsed().as_secs_f64() + seconds_ahead;
        total_elapsed * self.drift_ppb * 1e-9
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_clock_zero_entropy() {
        let clock = LocalClock::new(10.0, 300.0);
        assert_eq!(clock.phase_entropy_bits(), 0.0);
        assert_eq!(clock.desync_energy_joules(), 0.0);
    }

    #[test]
    fn drift_accumulates_entropy() {
        let mut clock = LocalClock::new(10.0, 300.0);
        // Simulate 1 second of drift at 10 ppb = 10 ns uncertainty
        clock.phase_uncertainty_sec = 10e-9; // 10 ns
        let bits = clock.phase_entropy_bits();
        // log2(10ns / 1ns) = log2(10) ≈ 3.32 bits
        assert!((bits - 3.32).abs() < 0.1, "Expected ~3.32 bits, got {bits}");
    }

    #[test]
    fn desync_energy_scales_with_temperature() {
        let mut clock_300 = LocalClock::new(10.0, 300.0);
        let mut clock_600 = LocalClock::new(10.0, 600.0);
        clock_300.phase_uncertainty_sec = 100e-9;
        clock_600.phase_uncertainty_sec = 100e-9;

        let e300 = clock_300.desync_energy_joules();
        let e600 = clock_600.desync_energy_joules();

        assert!(
            (e600 / e300 - 2.0).abs() < 0.01,
            "Desync energy should scale linearly with T"
        );
    }

    #[test]
    fn sync_resets_uncertainty() {
        let mut clock = LocalClock::new(10.0, 300.0);
        clock.phase_uncertainty_sec = 100e-9;
        assert!(clock.phase_entropy_bits() > 0.0);

        clock.record_sync();
        assert_eq!(clock.phase_uncertainty_sec, 0.0);
        assert_eq!(clock.phase_entropy_bits(), 0.0);
    }
}

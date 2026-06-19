// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Intel RAPL (Running Average Power Limit) energy telemetry.
//!
//! Reads real energy consumption from the CPU's power meters.
//! This gives us ground truth for comparing actual sync energy
//! against the Landauer theoretical floor.
//!
//! ## Platform paths
//!
//! | OS | Source | Path / command |
//! |----|--------|----------------|
//! | **Linux (Intel)** | sysfs energy counter | `/sys/class/powercap/intel-rapl:0/energy_uj` |
//! | **Linux (AMD)** | sysfs energy counter | `/sys/class/hwmon/hwmon*/energy*_input` (platform-specific) |
//! | **macOS** | powermetrics (root) | `sudo powermetrics --samplers cpu_power -i 1 -n 1` — not wired in CI; returns [`RaplError::NotAvailable`] |
//! | **Other** | external meter | Monsoon, INA219, or mocked readings in tests |
//!
//! RAPL is available on Intel (Sandy Bridge+) and AMD (Zen+) CPUs.
//! On macOS, direct RAPL sysfs is unavailable; use powermetrics or an external meter.

use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaplError {
    #[error("RAPL not available on this platform")]
    NotAvailable,
    #[error("IO error reading RAPL: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to parse RAPL value: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

/// RAPL energy reading in microjoules.
#[derive(Debug, Clone, Copy)]
pub struct EnergyReading {
    /// Energy counter value in microjoules.
    pub microjoules: u64,
}

impl EnergyReading {
    /// Convert to joules.
    pub fn joules(&self) -> f64 {
        self.microjoules as f64 * 1e-6
    }

    /// Difference between two readings (handles counter wrap).
    pub fn delta(&self, previous: &EnergyReading) -> f64 {
        if self.microjoules >= previous.microjoules {
            (self.microjoules - previous.microjoules) as f64 * 1e-6
        } else {
            // Counter wrapped (rare, but handle it)
            (u64::MAX - previous.microjoules + self.microjoules) as f64 * 1e-6
        }
    }
}

/// Read the current package energy from RAPL (Linux).
///
/// On Linux, RAPL is exposed at:
/// `/sys/class/powercap/intel-rapl:0/energy_uj`
///
/// On other platforms, returns NotAvailable.
pub fn read_package_energy() -> Result<EnergyReading, RaplError> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        let path = "/sys/class/powercap/intel-rapl:0/energy_uj";
        let content = fs::read_to_string(path)?;
        let microjoules: u64 = content.trim().parse()?;
        Ok(EnergyReading { microjoules })
    }

    #[cfg(all(target_os = "macos", not(ucrs_skip_powermetrics)))]
    {
        read_powermetrics_energy()
    }

    #[cfg(all(
        not(target_os = "linux"),
        any(not(target_os = "macos"), ucrs_skip_powermetrics)
    ))]
    {
        Err(RaplError::NotAvailable)
    }
}

/// macOS fallback: parse cumulative package energy from `powermetrics` (requires root).
#[cfg(all(target_os = "macos", not(ucrs_skip_powermetrics)))]
fn read_powermetrics_energy() -> Result<EnergyReading, RaplError> {
    use std::process::Command;
    let output = Command::new("powermetrics")
        .args(["--samplers", "cpu_power", "-i", "1", "-n", "1"])
        .output()
        .map_err(RaplError::Io)?;
    if !output.status.success() {
        return Err(RaplError::NotAvailable);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("CPU Power: ") {
            if let Some(mj) = rest.split_whitespace().next() {
                if let Ok(millijoules) = mj.parse::<f64>() {
                    let microjoules = (millijoules * 1_000.0).round() as u64;
                    return Ok(EnergyReading { microjoules });
                }
            }
        }
    }
    Err(RaplError::NotAvailable)
}

/// Measure the energy cost of a closure.
///
/// Returns (result, energy_joules). On non-Linux platforms, energy
/// is reported as None.
pub fn measure_energy<F, R>(f: F) -> (R, Option<f64>)
where
    F: FnOnce() -> R,
{
    let before = read_package_energy().ok();
    let result = f();
    let after = read_package_energy().ok();

    let energy = match (before, after) {
        (Some(b), Some(a)) => Some(a.delta(&b)),
        _ => None,
    };

    (result, energy)
}

/// Sync energy measurement record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncEnergyRecord {
    /// Bits of phase uncertainty resolved in this sync.
    pub bits_resolved: f64,
    /// Theoretical Landauer floor (joules).
    pub landauer_floor_j: f64,
    /// Actual measured energy (joules), if RAPL available.
    pub measured_j: Option<f64>,
    /// Overhead ratio: measured / Landauer floor.
    /// A real system always has overhead >> 1 (typically 10^6 to 10^12).
    pub overhead_ratio: Option<f64>,
}

impl SyncEnergyRecord {
    pub fn new(bits_resolved: f64, temperature_k: f64, measured_j: Option<f64>) -> Self {
        let landauer_floor_j = crate::landauer::landauer_cost(bits_resolved, temperature_k);
        let overhead_ratio = measured_j.map(|m| m / landauer_floor_j);

        Self {
            bits_resolved,
            landauer_floor_j,
            measured_j,
            overhead_ratio,
        }
    }

    /// Check the Second Law: measured energy must exceed Landauer floor.
    pub fn second_law_satisfied(&self) -> Option<bool> {
        self.measured_j.map(|m| m >= self.landauer_floor_j)
    }
}

/// Export `sync_overhead_ratio` to Prometheus (`ucrs_sync_overhead_ratio` histogram).
///
/// Called after each sync event when RAPL measurement is available (or simulated).
pub fn export_sync_overhead(record: &SyncEnergyRecord) {
    if let Some(ratio) = record.overhead_ratio {
        crate::telemetry::SYNC_COST_RATIO.observe(ratio);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_reading_delta() {
        let a = EnergyReading { microjoules: 1000 };
        let b = EnergyReading { microjoules: 5000 };
        assert!((b.delta(&a) - 0.004).abs() < 1e-10);
    }

    #[test]
    fn sync_record_second_law() {
        let record = SyncEnergyRecord::new(
            10.0,        // 10 bits
            300.0,       // room temp
            Some(1e-15), // 1 femtojoule (way above Landauer floor)
        );
        assert!(record.second_law_satisfied().unwrap());
        assert!(record.overhead_ratio.unwrap() > 1.0);
    }

    #[test]
    fn sync_record_floor() {
        let record = SyncEnergyRecord::new(1.0, 300.0, None);
        // 1 bit at 300K: ~2.87e-21 J
        assert!((record.landauer_floor_j - 2.87e-21).abs() < 1e-22);
    }
}

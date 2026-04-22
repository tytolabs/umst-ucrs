// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Landauer thermodynamic cost computation.
//!
//! Mirrors the Lean-verified definitions from `UMSTCore.lean` and
//! `LandauerBound.lean`. Every constant here matches the formally
//! verified value to full IEEE-754 f64 precision.

/// Boltzmann constant in SI (J/K) — exact per 2019 SI redefinition.
pub const K_B: f64 = 1.380_649e-23;

/// Speed of light in SI (m/s) — exact.
pub const C_SI: f64 = 299_792_458.0;

/// Landauer energy cost per bit at temperature T (joules).
///
/// `E_bit = k_B · T · ln(2)`
///
/// Formally verified in `umst-formal/Lean/LandauerLaw.lean`:
/// `landauerBound : proc.work ≥ proc.bath.bathTemp.val * log 2`
#[inline]
pub fn landauer_bit_energy(temperature_kelvin: f64) -> f64 {
    K_B * temperature_kelvin * f64::ln(2.0)
}

/// Landauer cost for resolving `bits` of uncertainty at temperature T.
///
/// Formally verified in `umst-formal-double-slit/Lean/LandauerBound.lean`:
/// `landauerCostDiagonal_n hn ρ T = landauerBitEnergy T * pathEntropyBits_n hn ρ`
#[inline]
pub fn landauer_cost(bits: f64, temperature_kelvin: f64) -> f64 {
    landauer_bit_energy(temperature_kelvin) * bits
}

/// Mass equivalent of Landauer energy via E = mc².
///
/// Formally verified in `umst-formal/Lean/LandauerEinsteinBridge.lean`:
/// `massEquivalent T = landauerBitEnergy T / c²`
#[inline]
pub fn mass_equivalent_per_bit(temperature_kelvin: f64) -> f64 {
    landauer_bit_energy(temperature_kelvin) / (C_SI * C_SI)
}

/// Desync energy: minimum thermodynamic cost to know the clock's true phase.
///
/// `D(clock, T) = k_B T ln(2) · H(phase)`
///
/// where H(phase) is the entropy of the phase estimate in bits.
/// Derived from the Galois connection (EpistemicGalois.lean):
/// `requiredEnergy T I ≤ E ↔ I ≤ acquirableInfo T E`
#[inline]
pub fn desync_energy(phase_entropy_bits: f64, temperature_kelvin: f64) -> f64 {
    landauer_cost(phase_entropy_bits, temperature_kelvin)
}

/// Coordination cost: thermodynamic price of ignoring inter-agent correlations.
///
/// `CoordCost(A,B,T) = E(A,T) + E(B,T) - E(AB,T) = k_B T ln(2) · I(A:B)`
///
/// Formally: this is the multi-agent analogue of the Cost-Coherence Identity.
/// When I(A:B) > 0, joint erasure is cheaper than independent erasure.
#[inline]
pub fn coordination_cost(mutual_info_bits: f64, temperature_kelvin: f64) -> f64 {
    landauer_cost(mutual_info_bits, temperature_kelvin)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T_ROOM: f64 = 300.0; // room temperature in Kelvin

    #[test]
    fn landauer_bit_energy_at_300k() {
        let e = landauer_bit_energy(T_ROOM);
        // k_B · 300 · ln(2) ≈ 2.87 × 10⁻²¹ J
        assert!(
            (e - 2.872e-21).abs() < 1e-23,
            "Landauer bit energy at 300K: {e}"
        );
    }

    #[test]
    fn landauer_cost_one_bit() {
        let cost = landauer_cost(1.0, T_ROOM);
        let bit_energy = landauer_bit_energy(T_ROOM);
        assert!((cost - bit_energy).abs() < f64::EPSILON);
    }

    #[test]
    fn landauer_cost_zero_bits() {
        assert_eq!(landauer_cost(0.0, T_ROOM), 0.0);
    }

    #[test]
    fn desync_energy_proportional_to_entropy() {
        let e1 = desync_energy(1.0, T_ROOM);
        let e2 = desync_energy(2.0, T_ROOM);
        assert!((e2 - 2.0 * e1).abs() < f64::EPSILON);
    }

    #[test]
    fn mass_equivalent_at_300k() {
        let m = mass_equivalent_per_bit(T_ROOM);
        // ≈ 3.19 × 10⁻³⁸ kg (matches LandauerEinsteinBridge.lean)
        assert!((m - 3.19e-38).abs() < 1e-39, "Mass equivalent at 300K: {m}");
    }

    #[test]
    fn coordination_cost_nonneg() {
        // Mutual information is always ≥ 0, so coordination cost ≥ 0
        assert!(coordination_cost(0.5, T_ROOM) >= 0.0);
        assert_eq!(coordination_cost(0.0, T_ROOM), 0.0);
    }

    #[test]
    fn zero_temperature_zero_cost() {
        assert_eq!(landauer_bit_energy(0.0), 0.0);
        assert_eq!(desync_energy(10.0, 0.0), 0.0);
    }
}

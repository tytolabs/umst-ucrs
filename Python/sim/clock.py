"""
UMST-UCRS  SimClock
===================
Simulates a clock with configurable drift and tracks phase uncertainty
so that the Landauer cost of re-synchronisation can be computed.
"""

from __future__ import annotations

import numpy as np

from .landauer import desync_energy as _desync_energy

# Default resolution used to convert phase uncertainty to entropy
_NS_RESOLUTION: float = 1e-9  # 1 ns


class SimClock:
    """A simulated clock that accumulates drift over time.

    Parameters
    ----------
    drift_ppb : float
        Fractional frequency drift in parts-per-billion.
    phase_uncertainty_sec : float
        Initial phase uncertainty (seconds).  Defaults to 0.
    temperature_k : float
        Ambient temperature in Kelvin.  Defaults to 300 K.
    """

    def __init__(
        self,
        drift_ppb: float = 0.0,
        phase_uncertainty_sec: float = 0.0,
        temperature_k: float = 300.0,
    ) -> None:
        self.drift_ppb: float = drift_ppb
        self.phase_uncertainty_sec: float = phase_uncertainty_sec
        self.temperature_k: float = temperature_k
        self._elapsed: float = 0.0  # total simulated seconds

    # ------------------------------------------------------------------
    # Simulation step
    # ------------------------------------------------------------------

    def advance(self, seconds: float) -> None:
        """Advance the clock by *seconds*, accumulating drift."""
        drift_sec = abs(self.drift_ppb) * 1e-9 * seconds
        self.phase_uncertainty_sec += drift_sec
        self._elapsed += seconds

    # ------------------------------------------------------------------
    # Entropy / energy helpers
    # ------------------------------------------------------------------

    def phase_entropy_bits(self) -> float:
        """Shannon entropy (bits) of the phase uncertainty.

        Computed as log2(uncertainty / 1-ns resolution).
        Returns 0 when uncertainty is at or below the resolution floor.
        """
        ratio = self.phase_uncertainty_sec / _NS_RESOLUTION
        if ratio <= 1.0:
            return 0.0
        return float(np.log2(ratio))

    def desync_energy_joules(self) -> float:
        """Landauer cost (J) to erase the current phase uncertainty."""
        return _desync_energy(self.phase_entropy_bits(), self.temperature_k)

    # ------------------------------------------------------------------
    # Sync
    # ------------------------------------------------------------------

    def record_sync(self) -> None:
        """Record a synchronisation event — resets phase uncertainty."""
        self.phase_uncertainty_sec = 0.0

    # ------------------------------------------------------------------
    # Convenience
    # ------------------------------------------------------------------

    @property
    def elapsed(self) -> float:
        return self._elapsed

    def __repr__(self) -> str:
        return (
            f"SimClock(drift_ppb={self.drift_ppb}, "
            f"phase_uncertainty={self.phase_uncertainty_sec:.3e} s, "
            f"T={self.temperature_k} K)"
        )

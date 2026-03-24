"""
UMST-UCRS  Landauer-limit utilities
===================================
Shared physical constants and thermodynamic cost functions used
throughout the simulation suite.
"""

import numpy as np

# ---------------------------------------------------------------------------
# Physical constants
# ---------------------------------------------------------------------------
K_B: float = 1.380649e-23          # Boltzmann constant  (J / K)
C_SI: float = 299_792_458.0        # Speed of light      (m / s)


# ---------------------------------------------------------------------------
# Core energy functions
# ---------------------------------------------------------------------------

def landauer_bit_energy(T: float) -> float:
    """Minimum energy to erase one bit at temperature *T* (Kelvin).

    E_bit = k_B * T * ln(2)
    """
    return K_B * T * np.log(2)


def landauer_cost(bits: float, T: float) -> float:
    """Total Landauer cost for erasing *bits* bits at temperature *T*."""
    return landauer_bit_energy(T) * bits


def desync_energy(phase_entropy_bits: float, T: float) -> float:
    """Energy cost of a clock desynchronisation event whose phase
    uncertainty carries *phase_entropy_bits* bits of Shannon entropy,
    evaluated at temperature *T*.

    This is simply the Landauer cost of erasing that uncertainty.
    """
    return landauer_cost(phase_entropy_bits, T)

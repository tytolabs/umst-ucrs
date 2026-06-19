"""Credit convergence stub — Phase E2 placeholder."""

from __future__ import annotations

from .credit import CreditLedger


def simulate_convergence(rounds: int = 10) -> float:
    ledger = CreditLedger()
    ledger.add_peer("honest", drift_ppb=5.0)
    for _ in range(rounds):
        ledger.record_sync("honest", credit_delta=1.0, observed_drift_ppb=5.0)
    return ledger.get_peer("honest").credit_bits  # type: ignore[union-attr]

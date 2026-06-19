"""Byzantine credit collapse stub — Phase E2 placeholder."""

from __future__ import annotations

from .credit import CreditLedger


def byzantine_suspect_count(bad_rounds: int = 10) -> int:
    ledger = CreditLedger()
    ledger.add_peer("bad", drift_ppb=5.0)
    for _ in range(bad_rounds):
        ledger.record_sync("bad", credit_delta=-2.0, observed_drift_ppb=100.0)
    return len(ledger.suspect_peers(threshold=0.5))

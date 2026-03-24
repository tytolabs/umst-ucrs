"""
UMST-UCRS  Credit Ledger
=========================
Manages peer credit accounting for the greedy clock-sync protocol.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Optional

import numpy as np


@dataclass
class PeerCredit:
    """Bookkeeping record for a single peer."""

    peer_id: str
    credit_bits: float = 0.0
    drift_ppb: float = 0.0
    accuracy_score: float = 1.0


class CreditLedger:
    """Maintains credit balances for a set of peers and supports
    greedy peer selection for clock synchronisation.
    """

    def __init__(self) -> None:
        self._peers: Dict[str, PeerCredit] = {}
        self._history: Dict[str, List[float]] = {}  # peer_id -> [credit snapshots]

    # ------------------------------------------------------------------
    # Peer management
    # ------------------------------------------------------------------

    def add_peer(self, peer_id: str, drift_ppb: float = 0.0) -> PeerCredit:
        pc = PeerCredit(peer_id=peer_id, drift_ppb=drift_ppb)
        self._peers[peer_id] = pc
        self._history[peer_id] = [pc.credit_bits]
        return pc

    def get_peer(self, peer_id: str) -> Optional[PeerCredit]:
        return self._peers.get(peer_id)

    @property
    def peers(self) -> List[PeerCredit]:
        return list(self._peers.values())

    # ------------------------------------------------------------------
    # Greedy selection
    # ------------------------------------------------------------------

    def best_peer(self, exclude: Optional[str] = None) -> Optional[PeerCredit]:
        """Return the peer with the highest accuracy score (greedy).

        *exclude* allows the caller to skip its own id.
        """
        candidates = [
            p for p in self._peers.values()
            if p.peer_id != exclude and p.accuracy_score > 0
        ]
        if not candidates:
            return None
        return max(candidates, key=lambda p: p.accuracy_score)

    # ------------------------------------------------------------------
    # Sync bookkeeping
    # ------------------------------------------------------------------

    def record_sync(
        self,
        peer_id: str,
        credit_delta: float,
        observed_drift_ppb: float,
    ) -> None:
        """Update a peer after a sync round.

        Parameters
        ----------
        peer_id : str
        credit_delta : float
            Positive = peer was helpful, negative = peer was inaccurate.
        observed_drift_ppb : float
            Most-recent measured drift for this peer.
        """
        pc = self._peers[peer_id]
        pc.credit_bits += credit_delta
        pc.drift_ppb = observed_drift_ppb
        # Accuracy score: inverse of drift magnitude, clamped to [0, 1]
        pc.accuracy_score = 1.0 / (1.0 + abs(observed_drift_ppb))
        self._history[peer_id].append(pc.credit_bits)

    # ------------------------------------------------------------------
    # Anomaly detection
    # ------------------------------------------------------------------

    def suspect_peers(self, threshold: float = 0.1) -> List[PeerCredit]:
        """Return peers whose accuracy score is below *threshold*."""
        return [p for p in self._peers.values() if p.accuracy_score < threshold]

    # ------------------------------------------------------------------
    # History access
    # ------------------------------------------------------------------

    def credit_history(self, peer_id: str) -> List[float]:
        return list(self._history.get(peer_id, []))

    def snapshot(self) -> None:
        """Append the current credit of every peer to its history."""
        for pid, pc in self._peers.items():
            self._history[pid].append(pc.credit_bits)

    # ------------------------------------------------------------------
    # Aggregate
    # ------------------------------------------------------------------

    def total_credit(self) -> float:
        return sum(p.credit_bits for p in self._peers.values())

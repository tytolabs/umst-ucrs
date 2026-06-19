"""Epoch overflow / reindex stub — Phase E2 placeholder."""

from __future__ import annotations

EPOCH_UNIX_2038: int = 2_147_483_647


def safe_reindex(epoch_sec: int) -> int:
    """Typed zero-cost reindex placeholder (no crash on boundary)."""
    if epoch_sec >= EPOCH_UNIX_2038:
        return epoch_sec - EPOCH_UNIX_2038
    return epoch_sec

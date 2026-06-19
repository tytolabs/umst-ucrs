"""Smoke tests for Phase E2 simulation stubs."""

from __future__ import annotations

from sim.byzantine_test import byzantine_suspect_count
from sim.credit_convergence import simulate_convergence
from sim.drift_monte_carlo import run_drift_monte_carlo
from sim.epoch_overflow import safe_reindex, EPOCH_UNIX_2038


def test_drift_monte_carlo_stub() -> None:
    out = run_drift_monte_carlo(samples=50)
    assert out["samples"] == 50.0
    assert out["stub"] == 1.0


def test_credit_convergence_stub() -> None:
    assert simulate_convergence(5) >= 5.0


def test_byzantine_stub() -> None:
    assert byzantine_suspect_count(8) >= 1


def test_epoch_overflow_stub() -> None:
    assert safe_reindex(EPOCH_UNIX_2038) == 0
    assert safe_reindex(100) == 100

"""E1 topology sweep — greedy sync should beat random on total cost."""

from __future__ import annotations

import pytest

from sim.network_topology import create_network, run_greedy_protocol, run_random_protocol, sweep


def test_greedy_lt_random_star() -> None:
    net_g = create_network("star", 25)
    net_r = create_network("star", 25)
    greedy = run_greedy_protocol(net_g, rounds=50)
    random_ = run_random_protocol(net_r, rounds=50)
    assert greedy <= random_, f"greedy {greedy} should be <= random {random_}"


def test_greedy_lt_random_ring() -> None:
    net_g = create_network("ring", 20)
    net_r = create_network("ring", 20)
    assert run_greedy_protocol(net_g, rounds=40) <= run_random_protocol(net_r, rounds=40)


def test_greedy_lt_random_mesh() -> None:
    net_g = create_network("mesh", 10)
    net_r = create_network("mesh", 10)
    assert run_greedy_protocol(net_g, rounds=30) <= run_random_protocol(net_r, rounds=30)


def test_sweep_produces_rows() -> None:
    results = sweep(peer_counts=[10], rounds=20)
    assert len(results) == 4  # star, ring, mesh, erdos_renyi
    for row in results:
        assert row["greedy_cost_J"] <= row["random_cost_J"] + 1e-9


def test_create_network_rejects_unknown() -> None:
    with pytest.raises(ValueError, match="Unknown topology"):
        create_network("hypercube", 5)

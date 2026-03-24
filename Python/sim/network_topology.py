"""
UMST-UCRS  Experiment E1 — Network Topology Comparison
======================================================
Creates star / ring / mesh / Erdos-Renyi topologies, then sweeps
greedy vs random sync protocols and records total Landauer cost.

When run as ``__main__`` the script produces:
  * ``sim/fig1_topology_cost.png``
  * ``sim/e1_results.csv``
"""

from __future__ import annotations

import csv
import os
from dataclasses import dataclass, field
from typing import Dict, List, Tuple

import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

from .clock import SimClock
from .credit import CreditLedger
from .landauer import landauer_cost

# ---------------------------------------------------------------------------
# UMST colour palette
# ---------------------------------------------------------------------------
UMST_BLUE = "#0047AB"
ACCENT_TEAL = "#008080"
SAFE_GREEN = "#228B22"
WARNING_RED = "#B22222"
GOLD_HIGHLIGHT = "#DAA520"


# ---------------------------------------------------------------------------
# Network helpers
# ---------------------------------------------------------------------------

@dataclass
class Network:
    """Adjacency-list representation of a peer network."""

    topology: str
    clocks: List[SimClock] = field(default_factory=list)
    adj: Dict[int, List[int]] = field(default_factory=dict)


def create_network(
    topology: str,
    n_peers: int,
    drift_dist: Tuple[float, float] = (10.0, 5.0),
    rng: np.random.Generator | None = None,
) -> Network:
    """Build a network of *n_peers* with the given *topology*.

    Parameters
    ----------
    topology : str
        One of ``"star"``, ``"ring"``, ``"mesh"``, ``"erdos_renyi"``.
    n_peers : int
        Number of peers (nodes).
    drift_dist : tuple(float, float)
        (mean, std) for lognormal drift distribution in ppb.
    rng : numpy Generator, optional
    """
    if rng is None:
        rng = np.random.default_rng(42)

    drifts = np.abs(rng.lognormal(np.log(drift_dist[0]), drift_dist[1] / drift_dist[0], n_peers))
    clocks = [SimClock(drift_ppb=d) for d in drifts]
    adj: Dict[int, List[int]] = {i: [] for i in range(n_peers)}

    if topology == "star":
        for i in range(1, n_peers):
            adj[0].append(i)
            adj[i].append(0)
    elif topology == "ring":
        for i in range(n_peers):
            nxt = (i + 1) % n_peers
            adj[i].append(nxt)
            adj[nxt].append(i)
    elif topology == "mesh":
        for i in range(n_peers):
            for j in range(i + 1, n_peers):
                adj[i].append(j)
                adj[j].append(i)
    elif topology == "erdos_renyi":
        p = min(1.0, 2.0 * np.log(n_peers) / n_peers) if n_peers > 1 else 1.0
        for i in range(n_peers):
            for j in range(i + 1, n_peers):
                if rng.random() < p:
                    adj[i].append(j)
                    adj[j].append(i)
        # ensure connected: add edges for isolated nodes
        for i in range(n_peers):
            if not adj[i]:
                j = (i + 1) % n_peers
                adj[i].append(j)
                adj[j].append(i)
    else:
        raise ValueError(f"Unknown topology: {topology}")

    return Network(topology=topology, clocks=clocks, adj=adj)


# ---------------------------------------------------------------------------
# Protocol runners
# ---------------------------------------------------------------------------

def _sync_pair(net: Network, i: int, j: int, T: float) -> float:
    """Sync clock *i* to clock *j*, return energy cost."""
    cost = net.clocks[i].desync_energy_joules()
    net.clocks[i].record_sync()
    return cost


def run_greedy_protocol(
    network: Network, T: float = 300.0, rounds: int = 100, interval: float = 1.0,
) -> float:
    """Run greedy sync: each node picks the neighbour with lowest drift.

    Returns total energy cost (Joules).
    """
    total_cost = 0.0
    n = len(network.clocks)
    for _ in range(rounds):
        # advance all clocks
        for c in network.clocks:
            c.advance(interval)
        # each node syncs to its best (lowest-drift) neighbour
        for i in range(n):
            neighbours = network.adj[i]
            if not neighbours:
                continue
            best_j = min(neighbours, key=lambda j: network.clocks[j].drift_ppb)
            total_cost += _sync_pair(network, i, best_j, T)
    return total_cost


def run_random_protocol(
    network: Network, T: float = 300.0, rounds: int = 100, interval: float = 1.0,
    rng: np.random.Generator | None = None,
) -> float:
    """Run random sync: each node picks a random neighbour.

    Returns total energy cost (Joules).
    """
    if rng is None:
        rng = np.random.default_rng(99)
    total_cost = 0.0
    n = len(network.clocks)
    for _ in range(rounds):
        for c in network.clocks:
            c.advance(interval)
        for i in range(n):
            neighbours = network.adj[i]
            if not neighbours:
                continue
            j = rng.choice(neighbours)
            total_cost += _sync_pair(network, i, j, T)
    return total_cost


# ---------------------------------------------------------------------------
# Sweep & plotting
# ---------------------------------------------------------------------------

def sweep(
    topologies: List[str] | None = None,
    peer_counts: List[int] | None = None,
    rounds: int = 100,
    T: float = 300.0,
) -> List[dict]:
    """Sweep topologies x peer counts, return list of result dicts."""
    if topologies is None:
        topologies = ["star", "ring", "mesh", "erdos_renyi"]
    if peer_counts is None:
        peer_counts = [10, 25, 50]

    results: List[dict] = []
    for topo in topologies:
        for n in peer_counts:
            net_g = create_network(topo, n, rng=np.random.default_rng(42))
            net_r = create_network(topo, n, rng=np.random.default_rng(42))
            cost_g = run_greedy_protocol(net_g, T=T, rounds=rounds)
            cost_r = run_random_protocol(net_r, T=T, rounds=rounds)
            results.append(
                dict(
                    topology=topo,
                    n_peers=n,
                    greedy_cost_J=cost_g,
                    random_cost_J=cost_r,
                    ratio=cost_r / cost_g if cost_g > 0 else float("inf"),
                )
            )
    return results


def save_csv(results: List[dict], path: str) -> None:
    keys = ["topology", "n_peers", "greedy_cost_J", "random_cost_J", "ratio"]
    with open(path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=keys)
        w.writeheader()
        w.writerows(results)


def plot_topology_cost(results: List[dict], save_path: str) -> None:
    """Bar chart: greedy vs random cost grouped by topology (n=25)."""
    subset = [r for r in results if r["n_peers"] == 25]
    if not subset:
        subset = results  # fallback

    topos = [r["topology"] for r in subset]
    greedy = [r["greedy_cost_J"] for r in subset]
    random_ = [r["random_cost_J"] for r in subset]

    x = np.arange(len(topos))
    width = 0.35

    fig, ax = plt.subplots(figsize=(8, 5))
    ax.bar(x - width / 2, greedy, width, label="Greedy", color=UMST_BLUE)
    ax.bar(x + width / 2, random_, width, label="Random", color=WARNING_RED)
    ax.set_xticks(x)
    ax.set_xticklabels(topos, fontsize=11)
    ax.set_ylabel("Total sync cost (J)", fontsize=12)
    ax.set_title("E1: Greedy vs Random Sync Cost by Topology (n=25)", fontsize=13, fontweight="bold")
    ax.legend(fontsize=11)
    ax.grid(axis="y", alpha=0.3)
    fig.tight_layout()
    fig.savefig(save_path, dpi=200)
    plt.close(fig)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> None:
    out_dir = os.path.dirname(os.path.abspath(__file__))
    results = sweep()
    save_csv(results, os.path.join(out_dir, "e1_results.csv"))
    plot_topology_cost(results, os.path.join(out_dir, "fig1_topology_cost.png"))
    print("E1 complete.")


if __name__ == "__main__":
    main()

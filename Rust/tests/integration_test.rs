// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! In-process star / mesh / ring topology sync simulation (no network).

use std::collections::HashMap;

use umst_ucrs::clock::LocalClock;
use umst_ucrs::credit::{CreditLedger, PeerId};
use umst_ucrs::{agent_tick, AgentConfig};

struct SimNode {
    clock: LocalClock,
    ledger: CreditLedger,
}

fn build_topology(topology: &str, n: usize) -> (Vec<SimNode>, Vec<(PeerId, PeerId)>) {
    let mut nodes: Vec<SimNode> = (0..n)
        .map(|id| {
            let drift = 5.0 + (id as f64) * 3.0;
            SimNode {
                clock: LocalClock::new(drift, 300.0),
                ledger: CreditLedger::new(id as PeerId, 300.0),
            }
        })
        .collect();

    let drifts: Vec<f64> = nodes.iter().map(|n| n.clock.drift_ppb).collect();
    let mut edges = Vec::new();
    match topology {
        "star" => {
            for i in 1..n {
                nodes[0].ledger.add_peer(i as PeerId, drifts[i]);
                nodes[i].ledger.add_peer(0, drifts[0]);
                edges.push((0, i as PeerId));
            }
        }
        "ring" => {
            for i in 0..n {
                let j = (i + 1) % n;
                nodes[i].ledger.add_peer(j as PeerId, drifts[j]);
                nodes[j].ledger.add_peer(i as PeerId, drifts[i]);
                edges.push((i as PeerId, j as PeerId));
            }
        }
        "mesh" => {
            for i in 0..n {
                for j in (i + 1)..n {
                    nodes[i].ledger.add_peer(j as PeerId, drifts[j]);
                    nodes[j].ledger.add_peer(i as PeerId, drifts[i]);
                    edges.push((i as PeerId, j as PeerId));
                }
            }
        }
        other => panic!("unknown topology: {other}"),
    }

    (nodes, edges)
}

fn force_drift(node: &mut SimNode) {
    node.clock.phase_uncertainty_sec = 1e-5;
    node.clock.last_sync = std::time::Instant::now() - std::time::Duration::from_secs(120);
}

fn run_rounds(nodes: &mut [SimNode], rounds: usize) -> f64 {
    let config = AgentConfig {
        budget_bits: 50.0,
        ..AgentConfig::default()
    };
    let mut total_cost = 0.0_f64;

    for _ in 0..rounds {
        for node in nodes.iter_mut() {
            force_drift(node);
            if let Some(record) = agent_tick(&mut node.clock, &mut node.ledger, &config) {
                total_cost += record
                    .measured_j
                    .map(|m| m.max(record.landauer_floor_j))
                    .unwrap_or(record.landauer_floor_j);
            }
        }
    }
    total_cost
}

#[test]
fn star_topology_sync_accumulates_cost() {
    let (mut nodes, _) = build_topology("star", 5);
    let cost = run_rounds(&mut nodes, 10);
    assert!(cost > 0.0, "star should incur Landauer cost over rounds");
    assert!(nodes[0].ledger.peers.len() >= 4);
}

#[test]
fn mesh_topology_higher_peer_fanout_than_ring() {
    let (mut mesh, _) = build_topology("mesh", 4);
    let (mut ring, _) = build_topology("ring", 4);

    let mesh_peers: usize = mesh.iter().map(|n| n.ledger.peers.len()).sum();
    let ring_peers: usize = ring.iter().map(|n| n.ledger.peers.len()).sum();
    assert!(mesh_peers > ring_peers, "mesh has more adjacency than ring");

    let mesh_cost = run_rounds(&mut mesh, 8);
    let ring_cost = run_rounds(&mut ring, 8);
    assert!(mesh_cost > 0.0 && ring_cost > 0.0);
}

#[test]
fn ring_topology_credit_transfer_is_local() {
    let (mut nodes, edges) = build_topology("ring", 6);
    let mut edge_counts: HashMap<(PeerId, PeerId), u64> = HashMap::new();
    for (a, b) in edges {
        *edge_counts.entry((a, b)).or_insert(0) += 1;
    }
    assert_eq!(edge_counts.len(), 6, "ring has one directed edge per node");

    run_rounds(&mut nodes, 5);
    let total_credit: f64 = nodes.iter().map(|n| n.ledger.total_credit()).sum();
    assert!(total_credit >= 0.0);
}

#[test]
fn all_topologies_finish_without_panic() {
    for topo in ["star", "ring", "mesh"] {
        let (mut nodes, _) = build_topology(topo, 5);
        let cost = run_rounds(&mut nodes, 3);
        assert!(cost.is_finite(), "{topo} cost finite");
    }
}

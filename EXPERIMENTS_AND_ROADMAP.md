# FCP-VI: Experiments and Remaining Work

> Towards UMST VI — Compositional Thermodynamic Accounting for
> Multi-Agent Constitutional Systems with Decentralized Coalgebraic
> Time Synchronization

**Repo:** `tytolabs/umst-ucrs`
**Paper:** `paper6_2026/Paper6_Draft.tex`
**Status:** Rust foundation complete (24 tests). Lean proofs, P2P networking, and experiments pending.

---

## 1. Remaining Lean Proofs

All new theorems chain from existing FCP-DS infrastructure (**486** theorems + **30** lemmas in **52** roots, **526** line-start decls in all `Lean/*.lean`, **0** `sorry`, **1** documented project `axiom` — see upstream `PROOF-STATUS.md` and `scripts/lean_declaration_stats.py`).
No new axioms required.

### Priority 0 — Core Theorems (paper blockers)

| # | Theorem | File | Depends on | Difficulty |
|---|---------|------|------------|------------|
| L1 | `landauerCost_tensor_product` | `TensorLandauer.lean` | `tensorDensity` + `landauerCostDiagonal_n` | Low |
| L2 | `landauerCost_joint_le_sum_marginals` | `TensorLandauer.lean` | Subadditivity of S(ρ) | **Medium** — needs `S(AB) ≤ S(A)+S(B)` |
| L3 | `coordinationCost_eq_mutualInfo_energy` | `CoordinationCost.lean` | Pure algebra on L1, L2 | Low |
| L4 | `coordinationCost_nonneg` | `CoordinationCost.lean` | `quantumMutualInfo ≥ 0` (from L2) | Low |

### Priority 1 — Clock & Credit Theorems

| # | Theorem | File | Depends on | Difficulty |
|---|---------|------|------------|------------|
| L5 | `desyncEnergy_bounded` | `DesyncEnergy.lean` | `EpistemicGalois` + `Gate` | Low |
| L6 | `epochPatch_wellTyped` | `EpochPatch.lean` | `Gate.lean` admissibility | Low |
| L7 | `multiScaleCost_bounded` | `ClockCoalgebra.lean` | `kleisliFoldWellTypedN` | Low |
| L8 | `credit_greedy_optimal` | `CreditOptimality.lean` | Galois + submodularity | **Hard** — may need new lemmas |

### Priority 2 — Stretch Goals

| # | Theorem | Notes |
|---|---------|-------|
| L9 | `byzantine_credit_diverges` | Exponential credit decay for dishonest peers |
| L10 | `multipartite_coordination_cost` | n-party generalization of Coordination Cost Identity |

### Lean Setup

```bash
cd umst-ucrs/Lean
# lakefile.lean imports Mathlib (same version as umst-formal-double-slit)
lake build
```

**Critical constraint:** Do NOT import umst-formal-double-slit as a Lake dependency.
Re-state foundational types (DensityMatrix, KrausChannel) via type-compatible
definitions. Reference prior results by citation, not build-time linking.

---

## 2. Rust Implementation Roadmap

### Phase 1 — P2P Networking (Week 1-2)

| Task | File | Notes |
|------|------|-------|
| libp2p swarm setup | `Rust/src/p2p.rs` | Tokio + noise + yamux + mDNS |
| Sync message protocol (protobuf or serde) | `Rust/src/p2p.rs` | Fields: timestamp, drift_ppb, credit, peer_id |
| Peer discovery (mDNS for LAN) | `Rust/src/p2p.rs` | Auto-discover peers on local network |
| GossipSub for credit broadcasts | `Rust/src/p2p.rs` | Peers announce credit scores |
| CLI args (peer-id, port, bootstrap) | `Rust/src/main.rs` | Use `clap` |

**Acceptance:** 3 peers on localhost sync clocks and exchange credit.

### Phase 2 — RAPL Integration (Week 2-3)

| Task | File | Notes |
|------|------|-------|
| Linux RAPL reading (production) | `Rust/src/rapl.rs` | `/sys/class/powercap/intel-rapl:0/energy_uj` |
| macOS fallback (powermetrics) | `Rust/src/rapl.rs` | Subprocess call to `powermetrics` |
| Energy-bracketed sync measurement | `Rust/src/rapl.rs` | Before/after RAPL around each sync |
| Prometheus HTTP endpoint | `Rust/src/telemetry.rs` | Expose metrics at `:9090/metrics` |
| Grafana dashboard JSON | `scripts/grafana_dashboard.json` | Pre-built panels for sync metrics |

**Acceptance:** Real RAPL energy readings per sync event on Linux; overhead ratio plotted.

### Phase 3 — Multi-Peer Simulation (Week 3-4)

| Task | File | Notes |
|------|------|-------|
| Deterministic simulation harness | `Rust/tests/integration_test.rs` | No real network; simulated message passing |
| 3-peer, 10-peer, 50-peer topologies | `Rust/tests/integration_test.rs` | Star, mesh, ring |
| Credit convergence test | `Rust/tests/credit_test.rs` | Verify credit stabilizes |
| Byzantine injection test | `Rust/tests/credit_test.rs` | Inject 1 lying peer, verify isolation |

**Acceptance:** All topology tests pass; Byzantine peer isolated within 10 sync rounds.

### Phase 4 — CLI & Deployment (Week 4-5)

| Task | File | Notes |
|------|------|-------|
| `cargo install umst-ucrs` support | `Rust/Cargo.toml` | Publish to crates.io |
| Docker image | `Dockerfile` | Multi-stage build |
| SystemD service file | `scripts/umst-ucrs.service` | Auto-start on Linux |
| ROS 2 bridge (optional) | `Rust/src/ros2_bridge.rs` | Integrate with umst-prototype fleet |

---

## 3. Python Simulations

### Experiment E1: Network Topology Sweep

**Goal:** Measure total Landauer cost as a function of network topology.

```python
# Python/sim/network_topology.py
topologies = ['star', 'ring', 'mesh', 'random_erdos_renyi']
peer_counts = [3, 5, 10, 20, 50, 100]
drift_distributions = ['uniform_1_50ppb', 'bimodal_1ppb_100ppb']
temperature_K = 300.0

for topo in topologies:
    for n in peer_counts:
        for drift_dist in drift_distributions:
            network = create_network(topo, n, drift_dist)
            total_cost = run_credit_protocol(network, T=temperature_K, rounds=1000)
            greedy_cost = run_greedy_protocol(network, T=temperature_K, rounds=1000)
            random_cost = run_random_protocol(network, T=temperature_K, rounds=1000)
            record(topo, n, drift_dist, total_cost, greedy_cost, random_cost)
```

**Success criterion:** `greedy_cost < random_cost` for all configurations.
**Falsification:** If `random_cost ≤ greedy_cost` for any configuration,
the credit optimality theorem is wrong.

**Figure:** Fig. 1 in paper — bar chart of total Landauer cost by topology.

### Experiment E2: Drift Monte Carlo

**Goal:** Statistical validation of desync energy scaling.

```python
# Python/sim/drift_monte_carlo.py
N_RUNS = 10_000
drift_ppb = np.random.lognormal(mean=2.0, sigma=1.0, size=N_RUNS)  # realistic quartz
T = 300.0
sync_interval_sec = 60.0

for run in range(N_RUNS):
    clock = SimClock(drift_ppb=drift_ppb[run])
    clock.advance(sync_interval_sec)
    entropy_bits = clock.phase_entropy_bits()
    desync_energy = K_B * T * np.log(2) * entropy_bits
    record(drift_ppb[run], entropy_bits, desync_energy)
```

**Success criterion:** `desync_energy` scales linearly with `log2(drift * time / resolution)`.
**Figure:** Fig. 2 — scatter plot of drift vs. desync energy with Landauer floor line.

### Experiment E3: Credit Convergence

**Goal:** Show that credit scores stabilize and reflect true peer quality.

```python
# Python/sim/credit_convergence.py
N_PEERS = 10
N_ROUNDS = 500
# Peer 0: atomic-class (0.1 ppb), Peers 1-8: quartz (10 ppb), Peer 9: bad (1000 ppb)

for round in range(N_ROUNDS):
    for agent in agents:
        best_peer = agent.select_best_peer()  # greedy credit
        bits = sync(agent, best_peer)
        transfer_credit(agent, best_peer, bits)
    record_all_credits(round)
```

**Success criterion:**
- Peer 0 (atomic) converges to highest credit
- Peer 9 (bad) converges to lowest credit
- Total credit is conserved (zero-sum within ε)

**Figure:** Fig. 3 — line plot of credit per peer over rounds.

### Experiment E4: Byzantine Isolation

**Goal:** Show that a lying peer is isolated within O(log n) rounds.

```python
# Python/sim/byzantine_test.py
N_PEERS = 20
N_BYZANTINE = 1  # then 2, 3, 5

byzantine_peer = peers[0]
byzantine_peer.lie_amplitude = 1e-3  # 1ms false phase offset

for round in range(200):
    # Normal sync protocol
    for agent in honest_agents:
        sync_and_update_credit(agent)
    # Byzantine peer sends false timestamps
    byzantine_peer.send_false_sync()

    record_byzantine_credit(round)
    record_honest_accuracy(round)
```

**Success criterion:**
- Byzantine peer's credit < 0 within 10 rounds
- No honest peer selects Byzantine after round 15
- Honest peers' phase accuracy unaffected after isolation

**Falsification:** If Byzantine peer maintains positive credit, the detection
mechanism fails and needs redesign.

**Figure:** Fig. 4 — credit trajectory of Byzantine vs. honest peers.

### Experiment E5: Epoch Overflow Simulation

**Goal:** Validate that epoch boundaries are zero-cost reindexing.

```python
# Python/sim/epoch_overflow.py
epoch_boundaries = {
    'unix_2038': 2**31,          # seconds since 1970-01-01
    'gps_week_rollover': 1024,   # weeks (mod 1024)
    'gregorian_400yr': 146097,   # days in 400-year cycle
}

for name, overflow_value in epoch_boundaries.items():
    clock = SimClock(phase=overflow_value - 10)
    for tick in range(20):
        clock.advance(1)
        if clock.phase >= overflow_value:
            cost = clock.apply_epoch_patch(overflow_value)
            assert cost == 0.0, f"Epoch patch should be zero-cost, got {cost}"
        record(name, tick, clock.phase, clock.credit)
```

**Success criterion:** All epoch patches have exactly zero Landauer cost.

---

## 4. Haskell QuickCheck Properties

```haskell
-- Haskell/Test/CreditProperties.hs

-- Credit is conserved across all sync events
prop_creditConservation :: [SyncEvent] -> Bool
prop_creditConservation events =
    abs (totalCreditAfter events - totalCreditBefore events) < 1e-10

-- Landauer floor is never violated
prop_secondLaw :: SyncEvent -> Bool
prop_secondLaw event =
    measuredEnergy event >= landauerCost (bitsResolved event) (temperature event)

-- Credit of honest peer never decreases (in expectation)
prop_honestCreditMono :: HonestPeer -> [Round] -> Bool
prop_honestCreditMono peer rounds =
    let credits = map (creditOf peer) rounds
    in isMonotoneIncreasing (movingAverage 10 credits)

-- Byzantine peer's credit diverges to -∞
prop_byzantineDecay :: ByzantinePeer -> [Round] -> Bool
prop_byzantineDecay peer rounds =
    let credits = map (creditOf peer) rounds
    in last credits < head credits

-- Greedy selection always picks lowest-cost peer
prop_greedyMinCost :: CreditLedger -> Bool
prop_greedyMinCost ledger =
    let best = bestPeer ledger
        costs = map (syncCost ledger) (allPeers ledger)
    in syncCost ledger best == minimum costs
```

---

## 5. CI / GitHub Actions

### `lean.yml`
```yaml
name: Lean
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: leanprover/lean4-action@v1
      - run: cd Lean && lake build
```

### `rust.yml`
```yaml
name: Rust
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cd Rust && cargo test
      - run: cd Rust && cargo clippy -- -D warnings
```

### `python.yml`
```yaml
name: Python
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with: { python-version: '3.12' }
      - run: pip install numpy matplotlib pytest
      - run: cd Python && python -m pytest tests/
```

---

## 6. Paper Completion Checklist

### Content

- [x] Abstract
- [x] Introduction (contribution bullets)
- [x] Background (thermodynamic gate, Landauer, QMI)
- [x] Tensor Landauer Accounting (Theorems 1-3)
- [x] Clock Synchronization as Measurement Channel
- [x] Thermodynamic Credit System (Theorem 4 + Byzantine)
- [x] Coalgebraic Time-Keeper
- [x] Multi-Scale Feedback
- [x] Implementation (Rust table)
- [x] Formal Verification (stack table)
- [x] Implications for Safe Multi-Agent Intelligence
- [x] Conclusion + Future Work
- [x] References

### Figures (to generate)

- [ ] Fig. 1: Network topology Landauer cost comparison (E1)
- [ ] Fig. 2: Drift vs. desync energy scatter + Landauer floor (E2)
- [ ] Fig. 3: Credit convergence over rounds (E3)
- [ ] Fig. 4: Byzantine isolation trajectory (E4)
- [ ] Fig. 5: Architecture diagram (TikZ, from README)

### Tables (done in LaTeX)

- [x] Table 1: Multi-scale sync architecture
- [x] Table 2: Rust module inventory
- [x] Table 3: Verification stack

### Before Submission

- [ ] Compile PDF with pdflatex (2 passes for refs)
- [ ] Check all overfull hbox warnings < 10pt
- [ ] Zenodo upload as FCP-VI
- [ ] Update UMST Research Dashboard (v3.3)
- [ ] Cross-reference from umst-ucrs README

---

## 7. Timeline

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| **W1** | Lean proofs L1-L4 | `TensorLandauer.lean`, `CoordinationCost.lean` |
| **W1** | Python E1-E2 | Topology sweep + drift Monte Carlo |
| **W2** | Rust P2P networking | `p2p.rs` with 3-peer localhost demo |
| **W2** | Python E3-E4 | Credit convergence + Byzantine isolation |
| **W3** | RAPL integration | Real energy measurement on Linux |
| **W3** | Lean proofs L5-L7 | `DesyncEnergy.lean`, `EpochPatch.lean`, `ClockCoalgebra.lean` |
| **W4** | Paper figures | All 5 figures generated from experiment data |
| **W4** | Haskell QuickCheck | 5 property tests passing |
| **W5** | Paper polish | Final LaTeX pass, compile PDF |
| **W5** | Zenodo + Dashboard | FCP-VI published, dashboard updated to v3.3 |

---

## 8. Success Criteria and Falsification

### Success = all of these hold:

1. **Lean:** All P0 theorems (L1-L4) compile with 0 sorry
2. **Rust:** 40+ tests passing, including multi-peer integration
3. **Experiment E1:** Greedy protocol < random protocol cost (all topologies)
4. **Experiment E3:** Credit scores converge and reflect true peer quality
5. **Experiment E4:** Byzantine peer isolated within 15 rounds
6. **RAPL:** Measured energy > Landauer floor for every sync event

### Falsification conditions:

- If `greedy_cost ≥ random_cost` for any topology → credit optimality theorem is wrong
- If Byzantine peer maintains positive credit after 50 rounds → detection mechanism fails
- If `E_measured < E_Landauer` for any sync → either RAPL reading is wrong or Second Law is violated (the former is more likely)
- If `coordinationCost < 0` in simulation → mutual information calculation has a bug

---

## 9. Dependencies and Resources

### Hardware
- **Linux machine with Intel RAPL** (Sandy Bridge+ or AMD Zen+) for energy telemetry
- macOS dev machines for Lean + Rust development (RAPL falls back to mock)
- Optional: Raspberry Pi cluster for real multi-node P2P testing

### Software
- Lean 4 + Mathlib (same version as umst-formal-double-slit: v4.14.0)
- Rust stable 1.75+
- Python 3.10+ with NumPy, matplotlib, pytest
- Haskell Stack (for QuickCheck)
- pdflatex (TeX Live 2026)

### Data
- No external datasets required
- All experiments are synthetic (simulated clocks, simulated drift)
- RAPL provides real hardware measurements
- Optional: GDELT event data for validating "observation spike" correlations (stretch goal, not in paper scope)

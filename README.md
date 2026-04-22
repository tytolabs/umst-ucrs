# UMST-UCRS: Peer-to-Peer Thermodynamic Clock Synchronization

> **Towards Unified Material-State Tensors VI:**
> Compositional Thermodynamic Accounting for Multi-Agent Constitutional Systems
> with Decentralized Coalgebraic Time Synchronization

[![Lean](https://img.shields.io/badge/Lean_4-Mathlib-blue)](Lean/)
[![Rust](https://img.shields.io/badge/Rust-Tokio%20P2P-orange)](Rust/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**Changelog:** [`CHANGELOG.md`](CHANGELOG.md).

---

## The Core Idea

Every clock synchronization message between peers is a **measurement**.
Every measurement has a **Landauer cost**: at least `k_B T ln(2)` joules per bit
of phase uncertainty resolved. In a decentralized network, the total
synchronization cost is bounded by the **sum of pairwise quantum mutual
informations** — and minimized when agents preferentially sync with
high-accuracy peers.

This repo formalizes and implements that idea:

| Layer | Tool | What it proves / measures |
|-------|------|--------------------------|
| **Formal proofs** | Lean 4 + Mathlib | Tensor Landauer theorem, coordination cost identity, credit optimality |
| **Working system** | Rust (Tokio, libp2p) | Real P2P clock daemon with RAPL energy telemetry |
| **Property tests** | Haskell QuickCheck | Thermodynamic invariant fuzzing |
| **Simulations** | Python (NumPy) | Network topology sweeps, drift Monte Carlo |

## Quick Stats

| Metric | Value |
|--------|-------|
| Lean (inherits [FCP-DS][ds]) | **537** `theorem` + **34** `lemma` in **59** lake roots (**571** line-start decls); **581** incl. all `Lean/*.lean` — upstream `scripts/lean_declaration_stats.py` |
| Lean (meso [FCP-I][mf]) | **221** `theorem` + **17** `lemma` in **45** roots — [umst-formal][mf] (gates, Landauer bridge, economics track) |
| Rust daemon | P2P clock sync with Landauer metering |
| Credit system | Thermodynamic economy — accuracy = credit |
| Axioms | **This repo:** 0 new Lean axioms planned. **FCP-DS:** 1 documented project `axiom` (`physicalSecondLaw`); see [PROOF-STATUS][ds-proof]. |

[ds]: https://github.com/tytolabs/umst-formal-double-slit
[ds-proof]: https://github.com/tytolabs/umst-formal-double-slit/blob/main/PROOF-STATUS.md
[mf]: https://github.com/tytolabs/umst-formal

---

## Relationship to Prior Work

```
FCP-I   (Physics-Gated AI)          ─── single agent, single gate
FCP-II  (Epistemic Sensing)         ─── single agent, MI-guided measurement
FCP-III (Functorial Mediation)      ─── multi-agent hierarchy (theory)
FCP-IV  (LandauerMark)              ─── macro→micro energy bridge + RAPL
FCP-V   (Culture as Scaling Layer)  ─── collective colimit
FCP-DS  (Thermodynamic Cost)        ─── quantum measurement Landauer cost
                                         537 th + 34 lem (59 roots), 0 sorry,
                                         1 documented project axiom
    │
    ▼
FCP-VI  (THIS REPO)                 ─── multi-agent Landauer accounting
                                         P2P clock sync as measurement protocol
                                         Rust daemon with real energy telemetry
```

**This repo does NOT fork or duplicate umst-formal-double-slit.**
It imports the Lean Mathlib dependency chain and references proven results
(tensor product, partial trace, mutual information, Galois connection)
as foundations for new theorems about multi-agent composition.

---

## Architecture

### The Thermodynamic Credit System

```
  Agent A (low drift)              Agent B (high drift)
  ┌─────────────────┐              ┌─────────────────┐
  │ phase: 0.001 rad│  sync msg    │ phase: 0.847 rad│
  │ credit: 94      │─────────────>│ credit: 12      │
  │ drift: 2 ppb    │              │ drift: 150 ppb  │
  └─────────────────┘              └─────────────────┘
         │                                  │
         ▼                                  ▼
  Landauer cost:                    Landauer cost:
  k_B T ln(2) · H(B|A)             k_B T ln(2) · H(A|B)
  = 0.003 aJ (cheap)               = 0.28 aJ (expensive)
         │                                  │
         └────── DUMSTO gate ───────────────┘
                      │
              Net credit transfer:
              B pays A proportional to
              information asymmetry I(A→B)
```

**Key properties (to be formally proved):**
1. Total network sync cost = `k_B T ln(2) · ∑_{edges} I(A:B)`
2. Credit-optimal topology minimizes total Landauer expenditure
3. Byzantine peers (lying about phase) are detectable: their credit drops
4. Epoch boundaries (Y2038, GPS rollover) are zero-cost reindexing

### P2P Daemon Architecture (Rust)

```
┌──────────────────────────────────────────────────┐
│                 umst-ucrs daemon                  │
│                                                   │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐ │
│  │ Local     │  │ P2P Sync  │  │ RAPL Energy  │ │
│  │ Oscillator│  │ (libp2p)  │  │ Telemetry    │ │
│  │ Module    │  │           │  │              │ │
│  └─────┬─────┘  └─────┬─────┘  └──────┬───────┘ │
│        │              │               │          │
│        ▼              ▼               ▼          │
│  ┌─────────────────────────────────────────────┐ │
│  │         DUMSTO Admissibility Gate            │ │
│  │  desyncEnergy(c, T) ≤ budget?               │ │
│  │  yes → apply correction, pay Landauer cost  │ │
│  │  no  → free-run, accumulate drift           │ │
│  └─────────────────────────────────────────────┘ │
│        │              │               │          │
│        ▼              ▼               ▼          │
│  ┌─────────────────────────────────────────────┐ │
│  │         Credit Ledger                        │ │
│  │  Per-peer accuracy score + cost accounting   │ │
│  └─────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

---

## Repository Layout

```
umst-ucrs/
├── Lean/                          # Formal proofs (Lean 4 + Mathlib)
│   ├── lakefile.lean              # Build config (imports Mathlib)
│   ├── TensorLandauer.lean        # Tensor product cost decomposition
│   ├── CoordinationCost.lean      # Mutual info discount theorem
│   ├── ClockCoalgebra.lean        # Final coalgebra on Kleisli monad
│   ├── CreditOptimality.lean      # Credit system minimizes total cost
│   ├── EpochPatch.lean            # Zero-cost epoch reindexing
│   └── DesyncEnergy.lean          # Desync as Landauer-bounded quantity
│
├── Rust/                          # Working P2P daemon
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs                # Daemon entry point
│   │   ├── clock.rs               # Local oscillator + drift model
│   │   ├── p2p.rs                 # libp2p sync protocol
│   │   ├── gate.rs                # DUMSTO admissibility check
│   │   ├── credit.rs              # Thermodynamic credit ledger
│   │   ├── rapl.rs                # Intel RAPL energy measurement
│   │   ├── landauer.rs            # Landauer cost computation
│   │   └── telemetry.rs           # Metrics export (Prometheus)
│   ├── benches/
│   │   └── sync_cost_bench.rs     # Measure real sync energy
│   └── tests/
│       ├── integration_test.rs    # Multi-peer network test
│       └── credit_test.rs         # Credit convergence test
│
├── Haskell/                       # Property-based testing
│   └── Test/
│       └── CreditProperties.hs    # QuickCheck: credit invariants
│
├── Python/                        # Simulations & analysis
│   ├── sim/
│   │   ├── network_topology.py    # Sweep star/mesh/ring topologies
│   │   ├── drift_monte_carlo.py   # 10k-run drift accumulation
│   │   └── credit_convergence.py  # Credit equilibrium simulation
│   └── tests/
│       └── test_landauer_floor.py # Validate Rust against theory
│
├── Docs/
│   ├── Preprint/                  # FCP-VI LaTeX paper
│   ├── Media/                     # Figures, diagrams
│   └── DESIGN.md                  # Architecture decisions
│
├── scripts/
│   ├── run_benchmarks.sh          # Full benchmark suite
│   └── generate_figures.py        # Paper figure generation
│
├── .github/workflows/
│   ├── lean.yml                   # Lean 4 CI (lake build)
│   ├── rust.yml                   # Rust CI (cargo test + clippy)
│   └── python.yml                 # Python CI (pytest)
│
├── LICENSE                        # MIT
├── FOUNDATION.md                  # Relationship to prior FCP work
├── CREDIT-SYSTEM.md               # Thermodynamic credit deep-dive
└── README.md                      # This file
```

---

## The Credit System — Ensuring Least Thermodynamic Cost

The credit system is not just accounting — it provably minimizes total
network synchronization cost. Here's why:

### Definition

Each agent `i` maintains a **credit score** `C_i`:

```
C_i = accuracy_i / cost_i
    = (1 / drift_i) / (k_B T ln(2) · H(phase_i))
```

### Protocol

1. Agent B wants to sync. It queries the peer with highest `C` in range.
2. The sync message resolves `H(phase_B | peer)` bits of uncertainty.
3. Agent B's energy cost: `E_sync = k_B T ln(2) · H(phase_B | peer)`.
4. Credit transfer: `ΔC = E_sync / (k_B T ln(2))` from B to peer.
5. DUMSTO gate checks: `E_sync ≤ budget_B`. If not, B free-runs.

### Optimality Theorem (to be proved in Lean)

**Claim:** The greedy credit protocol (always sync with highest-credit
peer in range) minimizes total network Landauer expenditure among all
sync protocols that achieve target accuracy `ε`.

**Proof sketch:** Highest-credit peer has lowest conditional entropy
`H(phase_B | peer)`, hence lowest Landauer cost per sync. Greedy
selection on a submodular function (mutual information) achieves
`(1 - 1/e)` approximation to optimal; exact optimality follows from
the matroid structure of spanning sync trees.

### Why This Matters for Multi-Agent AI

- **No free consensus:** Every shared reference frame costs energy.
  The credit system makes this cost explicit and minimizes it.
- **Byzantine detection:** A lying peer's corrections increase
  recipients' drift → their credit collapses → network isolates them.
  No explicit Byzantine protocol needed — thermodynamics does the work.
- **Frugality-first:** Agents sync only when `desyncEnergy > threshold`,
  minimizing total cost. Eager sync wastes energy; lazy sync accumulates
  drift. The DUMSTO gate finds the optimal balance.

---

## Getting Started

### Prerequisites

- [Lean 4](https://leanprover.github.io/lean4/doc/setup.html) + Mathlib
- [Rust](https://rustup.rs/) (stable, 1.75+)
- Python 3.10+ with NumPy, matplotlib
- (Optional) Haskell Stack for QuickCheck tests

### Build & Test

```bash
# Lean proofs
cd Lean && lake build

# Rust daemon
cd Rust && cargo test
cargo build --release

# Python simulations
cd Python && python -m pytest tests/

# Full benchmark
./scripts/run_benchmarks.sh
```

### Run the P2P Daemon

```bash
# Start 3 local peers for testing
cargo run -- --peer-id 1 --port 9001 --bootstrap localhost:9002,localhost:9003
cargo run -- --peer-id 2 --port 9002 --bootstrap localhost:9001,localhost:9003
cargo run -- --peer-id 3 --port 9003 --bootstrap localhost:9001,localhost:9002
```

---

## Citation

If you use this work, please cite:

```bibtex
@techreport{shyamsundar2026ucrs,
  title   = {Towards Unified Material-State Tensors {VI}:
             Compositional Thermodynamic Accounting for Multi-Agent
             Constitutional Systems with Decentralized Coalgebraic
             Time Synchronization},
  author  = {Shyamsundar, Santhosh and Shenbagamoorthy, Santosh Prabhu},
  year    = {2026},
  institution = {Studio TYTO},
  note    = {Preprint. Source: \url{https://github.com/tytolabs/umst-ucrs}}
}
```

### Related FCP Papers

| # | Title | DOI |
|---|-------|-----|
| FCP-I | Towards UMST: Physics-Gated AI | [10.5281/zenodo.18768547](https://doi.org/10.5281/zenodo.18768547) |
| FCP-II | Towards UMST: Epistemic Sensing | [10.5281/zenodo.18894710](https://doi.org/10.5281/zenodo.18894710) |
| FCP-DS | The Thermodynamic Cost of Knowing | [10.5281/zenodo.19159660](https://doi.org/10.5281/zenodo.19159660) |
| Dashboard | UMST Research Dashboard v3.2 | [10.5281/zenodo.18940933](https://doi.org/10.5281/zenodo.18940933) |

---

## License

MIT License. Copyright (c) 2026 Santhosh Shyamsundar,
Santosh Prabhu Shenbagamoorthy — Studio TYTO.

# UMST-UCRS — Universal Calendar Resolution Spine

> _This ecosystem is dedicated to the thousands of unnamed contributors who wrote formal proofs, maintained open-source compilers, and built mathematical libraries for years — often without evidence that any of it would be used beyond pure theory. They chose to make their work free, because they understood that knowledge about physical reality cannot be owned. Whatever this system achieves is yours._

**A frugality-first "constitutional time" layer for multi-agent systems** — a shared, physics-grounded *now* where every agent spends only the minimum thermodynamic energy needed to stay in sync with reality, enforced by the same thermodynamic gate as [`umst-manifold`](https://github.com/tytolabs/umst-manifold).

<!-- readme:status -->
[![CI — Rust](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml)
[![CI — Lean](https://github.com/tytolabs/umst-ucrs/actions/workflows/lean.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/lean.yml)
[![CI — Haskell](https://github.com/tytolabs/umst-ucrs/actions/workflows/haskell.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/haskell.yml)
[![CI — Python](https://github.com/tytolabs/umst-ucrs/actions/workflows/python.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/python.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

```bash
cd Rust && cargo test && cargo build --release   # the library + tests
```

---

## In one minute

Atomic clocks give the most precise physical *tick*. UCRS sits **above** them and does what they cannot: it continuously measures the **temporal drift** between calendars, machines, and AI agents, and forces each one to pay only the **Landauer-floor** cost — `k_B T ln(2)` joules per bit of phase uncertainty resolved — to correct it. If a sync path would cost more than it's worth, the **thermodynamic gate rejects it.**

Every sync message is a **measurement**, and every measurement has a thermodynamic price. UCRS makes that price explicit, routes coordination toward accurate peers, and keeps the whole mesh honest about time without a separate consensus protocol.

> **The simple version:** a shared, intelligent clock that keeps systems honest about *now* — they spend energy only when it actually improves their understanding of the present.

Clock-sync admissibility reuses the manifold's thermodynamic gate predicate (`gateCheck`, [`umst-formal` `Gate.lean`](https://github.com/tytolabs/umst-formal)), specialized here to desync-energy budgets — UCRS is the **time** layer, not a fifth gate conjunct.

---

## How it works

| Mechanism | What it does |
|-----------|--------------|
| **Desync energy** `D(t)` | The thermodynamic cost incurred because clocks have drifted; bounded by the gate. |
| **Landauer floor** | Each bit of resolved phase uncertainty costs `≥ k_B T ln(2)` J — the irreducible price of a measurement. |
| **Thermodynamic gate** | Sync fires only when it **reduces** total desync energy within budget; wasteful paths are rejected. |
| **Thermodynamic credit** | Accuracy is accounted per peer; high-drift peers pay more; lying about phase collapses credit — Byzantine resistance with no separate BFT layer. |
| **Observation stamps** | `UcrsObservedAt` — `phase_entropy_bits`, `ucrs_seq`, `credit_head_bits`, `stamp_tier` — written onto durable logs so accepts carry *thermodynamic* time, not wall clock alone. |
| **Offline spine** | A local oscillator plus a precomputed spine of calendar structures for long offline operation. |
| **Epoch safety** | Unix 2038 / GPS rollover become zero-cost typed reindexing instead of crashes. |

**Multi-agent coordination, in three rules:** no free consensus (every shared reference frame costs energy, and credit makes that explicit); Byzantine resistance is emergent (lying raises recipients' drift, so credit collapses and the network isolates the liar); frugality-first (sync only when `desyncEnergy` exceeds threshold — eager wastes energy, lazy accumulates drift, the gate finds the balance).

---

## Use it: the MCP session clock

Cartridge MCP agents bind durable accepts to thermodynamic time via [`TemporalWitness`](Rust/src/observation.rs) / [`witness_for_agent`](Rust/src/lib.rs):

```rust
use umst_ucrs::{witness_for_agent, AgentConfig};

let config = AgentConfig::default();
let mut witness = witness_for_agent(&config);
let stamp = witness.stamp(); // UcrsTier2: ucrs_seq, phase_entropy_bits_q, credit_head_bits_q
```

| API / env | Role |
|-----------|------|
| `witness_for_agent(&AgentConfig)` | Construct a live witness from peer id, drift, temperature. |
| `TemporalWitness::stamp()` | Advance uncertainty and emit a monotonic `UcrsObservedAt`. |
| `UMST_UCRS_WITNESS=live` | Real `stamp()` on cartridge ingest → `UcrsTier2`. |
| `UMST_UCRS_WITNESS=synthetic` | Deterministic monotonic stamps for CI (default). |

The library keeps `default = []` features so consumers (e.g. [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge)'s `ucrs-provenance`) never pull in libp2p — enable `p2p` only for mesh daemons. Logging policy: [`Docs/LOGGING_POLICY.md`](Docs/LOGGING_POLICY.md) · HLC sidecar (never overwrites `ucrs_seq`): [`Docs/HLC_SIDECAR.md`](Docs/HLC_SIDECAR.md).

---

## Scope — what UCRS is and isn't

UCRS owns **constitutional time, P2P sync economics, and observation stamps.** It does **not** store mix recipes, hydration outcomes, calibration quality, or agent contribution content — material knowledge lives in [`umst-concrete-cartridge` `contribution.v1`](https://github.com/tytolabs/umst-concrete-cartridge/blob/main/schemas/contribution.v1.json) research memory.

`tytolabs/umst-ucrs` is the **system of record for UCRS** — not an appendix of another repo.

| This repo owns | Lives elsewhere |
|----------------|-----------------|
| Thermodynamic P2P clock sync | Material gate → [`umst-manifold`](https://github.com/tytolabs/umst-manifold) |
| Credit ledger + sync gate | Cartridge physics → [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) |
| `UcrsObservedAt` observation stamps | Lean proof catalogs → [`umst-formal`](https://github.com/tytolabs/umst-formal), [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) |
| Rust library (`umst_ucrs`) for agents | Public site → [`studiotyto-website`](https://github.com/tytolabs/studiotyto-website) |

UCRS **imports** thermodynamic and gate results from the formal repos; it does not fork or duplicate them.

---

## What this repo implements

| Layer | Status | Location |
|-------|--------|----------|
| **Rust library** | Working | `Rust/src/` — clock, gate, credit, Landauer, RAPL telemetry |
| **P2P daemon** | In progress | `Rust/src/p2p.rs`, `Rust/src/bin/p2p.rs` — libp2p sync path (`p2p` feature) |
| **Lean proofs** | Scaffold | `Lean/` — tensor Landauer axioms (see `PROOF-STATUS.md`) |
| **Simulations** | Foundation | `Python/sim/` — topology + drift studies |
| **Property tests** | Scaffold | `Haskell/test/Spec.hs` — QuickCheck stubs |

```text
        umst-formal ── gates, Landauer bridge, Kleisli composition
umst-formal-double-slit ── quantum mutual information, tensor Landauer identities
                    │
                    ▼
            umst-ucrs (THIS REPO) ── multi-agent Landauer accounting, P2P clock as measurement
```

### Thermodynamic credit (summary)

```text
  Agent A (low drift)              Agent B (high drift)
  phase: 0.001 rad  ──sync──►      phase: 0.847 rad
  credit: 94                       credit: 12
         │                                  │
         └────── thermodynamic gate ─────────┘
              Landauer cost ∝ H(phase | peer)
              net credit transfer ∝ information asymmetry
```

**Target theorems** (status in `PROOF-STATUS.md`): (1) total network sync cost `= k_B T ln(2) · Σ_edges I(A:B)`; (2) greedy credit routing minimizes total Landauer spend at target accuracy; (3) Byzantine peers are detectable via credit collapse; (4) epoch patches are admissible at zero thermodynamic cost.

Protocol detail: [`CREDIT-SYSTEM.md`](CREDIT-SYSTEM.md) · formal foundations: [`FOUNDATION.md`](FOUNDATION.md).

---

## Repository layout

```text
umst-ucrs/
├── Rust/       # umst_ucrs library + daemon (primary deliverable)
├── Lean/       # Formal proofs (scaffold)
├── Python/sim/ # Topology + drift simulations
├── Haskell/    # QuickCheck properties
├── scripts/    # systemd unit, benchmarks
├── Docs/       # Design notes, logging + HLC policy
└── PROOF-STATUS.md · FOUNDATION.md · CREDIT-SYSTEM.md
```

Publish check: `cd Rust && cargo publish --dry-run`. Full daemon: `cargo build --release --features daemon`.

---

## Citation

```bibtex
@software{umst_ucrs2026,
  title     = {{UMST-UCRS}: Universal Calendar Resolution Spine},
  author    = {Shyamsundar, Santhosh and Shenbagamoorthy, Santosh Prabhu},
  year      = {2026},
  publisher = {Studio TYTO},
  url       = {https://github.com/tytolabs/umst-ucrs},
  license   = {MIT}
}
```

| Related repo | Focus |
|------|--------|
| [`umst-manifold`](https://github.com/tytolabs/umst-manifold) | Thermodynamic gate host |
| [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) | Concrete agent + MCP surface |
| [`umst-formal`](https://github.com/tytolabs/umst-formal) | Meso-scale formal proofs |
| [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) | Quantum-scale Landauer bridge |

---

## License

MIT License. Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.

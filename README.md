# UMST-UCRS

**Universal Calendar Resolution Spine** — frugality-first constitutional time for multi-agent systems.

[![Rust](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml)
[![layer: constitutional-time](https://img.shields.io/badge/layer-constitutional_time-2d3436)](FOUNDATION.md)
[![witness: library+live](https://img.shields.io/badge/witness-library%20%2B%20live-C9A27A)](Rust/src/observation.rs)
[![p2p: deferred](https://img.shields.io/badge/p2p-deferred-888888)](EXPERIMENTS_AND_ROADMAP.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

**Changelog:** [`CHANGELOG.md`](CHANGELOG.md) · **Credit system:** [`CREDIT-SYSTEM.md`](CREDIT-SYSTEM.md) · **Formal foundations:** [`FOUNDATION.md`](FOUNDATION.md)

---

## Independent repository

**`tytolabs/umst-ucrs` is the system of record for UCRS** — not a chapter appendix of another paper repo.

| This repo owns | Other repos own |
|----------------|-----------------|
| Thermodynamic P2P clock sync | Material gates → [`umst-manifold`](https://github.com/tytolabs/umst-manifold) |
| Credit ledger + thermodynamic sync gate | Cartridge physics → [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) |
| `UcrsObservedAt` observation stamps | Lean proof catalogs → [`umst-formal`](https://github.com/tytolabs/umst-formal), [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) |
| Rust library (`umst_ucrs`) consumed by agents | Public site → [`studiotyto-website`](https://github.com/tytolabs/studiotyto-website) |

Cartridges bind UCRS via an optional **`ucrs-provenance`** feature: durable logs (memory rows, gate rejects, promotion records) carry explicit **`stamp_tier`** observation stamps — thermodynamic time, not wall clock alone.

> **Scope box — UCRS is NOT material memory**  
> UCRS provides **constitutional time**, P2P sync economics, and `UcrsObservedAt` stamps on durable logs. It does **not** store mix recipes, hydration outcomes, calibration quality, or agent contribution content. Material knowledge lives in [`umst-concrete-cartridge` `contribution.v1`](https://github.com/tytolabs/umst-concrete-cartridge/blob/main/schemas/contribution.v1.json) research memory — see [`docs/AGENT_MCP.md`](https://github.com/tytolabs/umst-concrete-cartridge/blob/main/docs/AGENT_MCP.md).

---

## What is UCRS?

**UCRS** = **U**niversal **C**alendar **R**esolution **S**pine.

UCRS is the **frugality-first constitutional time layer** on top of the UMST stack. It gives every AI system (and any human system that wants it) a shared, physics-grounded **now**, while ensuring they never waste more energy than necessary to stay in sync with reality. Clock sync admissibility uses the same **thermodynamic gate** predicate as [`umst-manifold`](https://github.com/tytolabs/umst-manifold) (`gateCheck` in [`umst-formal`](https://github.com/tytolabs/umst-formal) `Gate.lean`), specialized here to desync-energy budgets.

### Simple reminder

> UCRS is a shared, intelligent clock that keeps systems honest about time — they only spend energy when it actually improves their understanding of **now**.

### Core idea in plain terms

- **Atomic clocks** give the most precise physical tick of a second.
- **UCRS** sits above them and does something atomic clocks cannot: it continuously measures how much **temporal noise** or **drift** exists between different calendars, computers, and AI agents.
- It then forces every agent to pay the **minimum thermodynamic cost** (Landauer cost) to correct that drift.
- If the cost is too high, the **thermodynamic gate** rejects the wasteful path.

Every sync message is a **measurement**. Every measurement has a Landauer floor: at least `k_B T ln(2)` joules per bit of phase uncertainty resolved.

### Key technical properties

| Property | Role |
|----------|------|
| **Final coalgebra** | Rigorous model of an ongoing, self-consistent sync process (`ClockCoalgebra.lean`, planned) |
| **Desync energy** `D(t)` | Thermodynamic cost paid because clocks have drifted; bounded by the thermodynamic gate |
| **Offline spine** | Local oscillator + pre-computed spine of known calendar structures for long offline operation |
| **P2P mesh** | Agents gossip timing only when the exchange **reduces** total desync energy (gate-enforced frugality) |
| **Thermodynamic credit** | Accuracy is accounted; high-drift peers pay more; Byzantine behavior collapses credit |
| **Epoch boundaries** | Unix 2038, GPS rollover → safe, zero-cost typed reindexing instead of crashes |
| **Observation stamps** | `UcrsObservedAt`: `phase_entropy_bits`, `ucrs_seq`, `credit_head_bits`, `stamp_tier` on durable logs |

### Where UCRS sits in the constitutional stack

```text
Time layer          → UCRS (frugality-first shared “now”)
Material layer      → UMST manifold (thermodynamic gate)
Geometry layer      → SDF / FREP (MaOS)
Gravity extension   → Volumetric potential gradient (manifold roadmap)
Multi-agent mesh    → Thermodynamic credit system + P2P gossip
```

Everything in this stack is designed so agents pay only the **real physical cost of knowing and acting** — nothing more.

### Multi-agent coordination

- **No free consensus** — every shared reference frame costs energy; credit makes that cost explicit and routes sync toward accurate peers.
- **Byzantine resistance** — lying about phase increases recipients' drift; credit collapses; the network isolates bad actors without a separate BFT protocol.
- **Frugality-first** — sync only when `desyncEnergy` exceeds threshold; eager sync wastes energy; lazy sync accumulates drift; the gate finds the balance.

---

## What this repo implements

| Layer | Status | Location |
|-------|--------|----------|
| **Rust library** | Working | `Rust/src/` — clock, gate, credit, Landauer, RAPL hooks |
| **P2P daemon** | In progress | `Rust/src/` — libp2p sync path |
| **Lean proofs** | Planned | `Lean/` — tensor Landauer, coordination cost, credit optimality |
| **Simulations** | Foundation | `Python/sim/` |
| **Property tests** | Planned | `Haskell/Test/` |

```bash
cd Rust && cargo test && cargo build --release
cd Python && python -m pytest tests/
./scripts/run_benchmarks.sh   # when configured
```

### P2P daemon (sketch)

```text
┌──────────────────────────────────────────────────┐
│                 umst-ucrs daemon                  │
│  Local oscillator → P2P sync → RAPL telemetry     │
│         ↓              ↓              ↓            │
│         thermodynamic gate (desyncEnergy ≤ budget?) │
│         Credit ledger (per-peer accuracy)          │
└──────────────────────────────────────────────────┘
```

See [`CREDIT-SYSTEM.md`](CREDIT-SYSTEM.md) for the credit protocol and optimality sketch.

---

## Relationship to UMST formal work

UCRS **imports** thermodynamic and gate results from sibling formal repos; it does **not** fork or duplicate them.

```text
umst-formal          ── gates, Landauer bridge, Kleisli composition
umst-formal-double-slit ── quantum mutual information, tensor Landauer identities
        │
        ▼
   umst-ucrs (THIS REPO) ── multi-agent Landauer accounting, P2P clock as measurement
```

Details: [`FOUNDATION.md`](FOUNDATION.md).

---

## Architecture — thermodynamic credit (summary)

```text
  Agent A (low drift)              Agent B (high drift)
  phase: 0.001 rad  ──sync──►      phase: 0.847 rad
  credit: 94                       credit: 12
         │                                  │
         └────── thermodynamic gate ─────────┘
              Landauer cost ∝ H(phase | peer)
              Net credit transfer ∝ information asymmetry
```

**Properties (target theorems):**

1. Total network sync cost = `k_B T ln(2) · ∑_{edges} I(A:B)`
2. Greedy credit routing minimizes total Landauer spend at target accuracy
3. Byzantine peers are detectable via credit collapse
4. Epoch patches are admissible at zero thermodynamic cost

---

## Repository layout

```text
umst-ucrs/
├── Rust/              # umst_ucrs library + daemon (primary deliverable)
├── Lean/              # Formal proofs (planned)
├── Python/sim/        # Topology + drift simulations
├── Haskell/Test/      # QuickCheck properties (planned)
├── Docs/              # Design notes, media
├── FOUNDATION.md
├── CREDIT-SYSTEM.md
└── README.md
```

---

## Citation

If you use UCRS or this implementation, please cite the repository:

```bibtex
@software{umst_ucrs2026,
  title   = {{UMST-UCRS}: Universal Calendar Resolution Spine},
  author  = {Shyamsundar, Santhosh and Shenbagamoorthy, Santosh Prabhu},
  year    = {2026},
  publisher = {Studio TYTO},
  url     = {https://github.com/tytolabs/umst-ucrs},
  license = {MIT}
}
```

### Related UMST repositories

| Repo | Focus |
|------|--------|
| [`umst-manifold`](https://github.com/tytolabs/umst-manifold) | Thermodynamic gate host |
| [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) | Concrete agent + MCP surface |
| [`umst-formal`](https://github.com/tytolabs/umst-formal) | Meso-scale formal proofs |
| [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) | Quantum-scale Landauer bridge |

---

## License

MIT License. Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.

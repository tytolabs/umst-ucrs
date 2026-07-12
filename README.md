<!--
SPDX-License-Identifier: MIT
Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO
-->
<!-- markdownlint-disable-file MD013 MD040 MD001 MD026 — hero README is intentionally dense; other docs stay strict via shared config. -->

# Universal Calendar Resolution Spine

### `umst-ucrs` — temporal witness / stamp spine

> _This ecosystem is dedicated to the thousands of unnamed contributors who wrote formal proofs, maintained open-source compilers, and built mathematical libraries for years — often without evidence that any of it would be used beyond pure theory. They chose to make their work free, because they understood that knowledge about physical reality cannot be owned. Whatever this system achieves is yours._

**Gloss.** “Calendar” here means systems of time-representation (Y2038-class epoch/clock drift), not appointments; UCRS is the constitutional-time and temporal-provenance spine.

**What it is.** The **time** organ of the shared thermodynamic admissibility gate: a Rust library that makes time itself gate-checked and Landauer-frugal — multi-agent sync economics **and** a generic temporal-witness / stamp spine for design steps across the stack.

**The gate / time idea.** Every sync is a typed measurement that resolves phase uncertainty at the Landauer floor (`k_B T ln 2` J/bit). Sync fires only when `gate_check` admits it against desync-energy budget and Clausius–Duhem on ψ; wasteful paths are rejected.

**Honest is / isn't.** **Is:** Rust clock / gate / credit / Landauer / observation stamps / frame–sheaf–decision infra (post infra-purity @ `5a3df25`); optional `p2p` daemon path; Lean + Haskell + Python scaffolds with status in [`PROOF-STATUS.md`](PROOF-STATUS.md). **Isn't:** a hot-arena physics kernel, an MCP host, or domain steering (TNA / vault logic lives in consumer crates). Do **not** blend “Rust tests green”, “Lean L5–L8 proved”, and “mesh daemon production-ready” into one completion %.

<!-- readme:status -->
[![CI — Rust](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/rust.yml)
[![CI — Lean](https://github.com/tytolabs/umst-ucrs/actions/workflows/lean.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/lean.yml)
[![CI — Haskell](https://github.com/tytolabs/umst-ucrs/actions/workflows/haskell.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/haskell.yml)
[![CI — Python](https://github.com/tytolabs/umst-ucrs/actions/workflows/python.yml/badge.svg)](https://github.com/tytolabs/umst-ucrs/actions/workflows/python.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-black.svg)](LICENSE)

### Shared stack (matter · knowing · acting · time)

These public repos share **one** thermodynamic admissibility gate, applied across domains:

| Domain | Public repo | Role |
|:---|:---|:---|
| **Matter** | [`umst-manifold`](https://github.com/tytolabs/umst-manifold) + [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) | DEC carrier + cementitious constitutive law |
| **Knowing** | [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) | Observation / measurement-cost formal fiber |
| **Acting** | [`umst-formal`](https://github.com/tytolabs/umst-formal) | Economic-admissibility formal fiber |
| **Time** | **this repo** ([`umst-ucrs`](https://github.com/tytolabs/umst-ucrs)) **← you are here** | Temporal witness / stamp spine |

Sibling links only — no paper-series arc naming in this README. Already-public per-repo DOI badges stay where they exist; this repo does not invent new ones here.

**Compositional cohesion.** Matter / knowing / acting each run under the shared gate. UCRS supplies the **when + provenance** morphism: any design step (manifold solve, cartridge accept, formal Kleisli step, double-slit observation cost) can carry a `UcrsObservedAt` stamp. The spine (`Frame` → ordered `Vertebra` → `DesignSheafOverSpine`) is the time-axis under that stamp — not a second product.

---

## In one minute

Atomic clocks give a precise physical *tick*. UCRS sits **above** them and does what they cannot: it measures **temporal drift** between clocks and agents, and forces each sync to pay only the **Landauer-floor** cost to resolve phase uncertainty. If a sync path would cost more than it returns, the thermodynamic gate **rejects** it.

Two faces, one substance:

1. **Constitutional time-sync** — `LocalClock` / `CreditLedger` / `landauer_cost` / optional P2P / RAPL telemetry. Accuracy is tradeable as credit; Byzantine peers lose credit without a separate BFT protocol ([`CREDIT-SYSTEM.md`](CREDIT-SYSTEM.md)).
2. **Generic witness / stamp spine** — `frame_spine` / `design_sheaf` / `decision_tree` expose witness, stamp, and steer-routing primitives. Domain steering (e.g. TNA) moved to consumer crates in the infra-purity refactor (PR #9 @ `5a3df25`).

> **The simple version:** a shared, gate-checked *now* — agents spend energy only when it improves their understanding of the present, and every durable accept can carry that thermodynamic time.

Clock-sync admissibility reuses the manifold / formal gate family (`clausius_duhem_admissible` via `umst-math`; formal `gateCheck` in [`umst-formal`](https://github.com/tytolabs/umst-formal)), specialized here to desync-energy budgets — UCRS is the **time** layer, not a fifth gate conjunct.

---

## Real objects (categorical — not “the system”)

| Symbol | Role (objects / morphisms) | Defined at |
|:---|:---|:---|
| `ClockThermState` | Object: desync energy, budget, T, monotone sync spend | [`Rust/src/gate.rs:15`](Rust/src/gate.rs) |
| `GateVerdict` / `gate_check` | Morphism: Admit \| Reject on Landauer cost + CD on ψ | [`Rust/src/gate.rs:28`](Rust/src/gate.rs), [`:44`](Rust/src/gate.rs) |
| `gated_sync` | Partial arrow: Admit → updated state; Reject → `None` | [`Rust/src/gate.rs:71`](Rust/src/gate.rs) |
| `landauer_cost` | Cost of resolving `bits` at temperature | [`Rust/src/landauer.rs:32`](Rust/src/landauer.rs) |
| `CreditLedger` | Object: per-peer accuracy credit; Byzantine collapse | [`Rust/src/credit.rs:59`](Rust/src/credit.rs) |
| `StampTier` | Enum: `UcrsTier2` \| `WallOnly` \| `Absent` \| `Synthetic` | [`Rust/src/observation.rs:21`](Rust/src/observation.rs) |
| `UcrsObservedAt` | Stamp object: `stamp_tier`, `ucrs_seq`, phase/credit fixed-point fields | [`Rust/src/observation.rs:42`](Rust/src/observation.rs) |
| `TemporalWitness` | Stateful producer: `stamp() → UcrsObservedAt` (Tier-2) | [`Rust/src/observation.rs:164`](Rust/src/observation.rs) |
| `Frame` / `SpineTime` / `Vertebra` / `Spine` | Frame + ordered vertebrae; each vertebra carries a stamp | [`Rust/src/frame_spine.rs:70`](Rust/src/frame_spine.rs), [`:87`](Rust/src/frame_spine.rs), [`:206`](Rust/src/frame_spine.rs), [`:218`](Rust/src/frame_spine.rs) |
| `SheafSection` / `DesignSheafOverSpine<M>` | Sections of admissibility along the spine time-axis | [`Rust/src/design_sheaf.rs:19`](Rust/src/design_sheaf.rs), [`:114`](Rust/src/design_sheaf.rs) |
| `SteerKnobs<L>` / `SteerDecision<W>` / `SteerPolicy<L,W>` | Generic steer routing (consumer supplies `L`, `W`) | [`Rust/src/decision_tree.rs:13`](Rust/src/decision_tree.rs), [`:22`](Rust/src/decision_tree.rs), [`:41`](Rust/src/decision_tree.rs) |
| `witness_for_agent` | Construct `TemporalWitness` from `AgentConfig` | [`Rust/src/lib.rs:87`](Rust/src/lib.rs) |

Wire schema for ticks: [`Rust/src/wire.rs`](Rust/src/wire.rs) (`phase_entropy_bits`, Landauer cost fields). Logging / HLC policy (HLC never overwrites `ucrs_seq`): [`Docs/LOGGING_POLICY.md`](Docs/LOGGING_POLICY.md) · [`Docs/HLC_SIDECAR.md`](Docs/HLC_SIDECAR.md).

---

## Performance honesty

UCRS is **infrastructure**, not a tensor hot-arena kernel.

| Path | What | Location |
|:---|:---|:---|
| **Witness / stamp (library)** | Pure-ish Rust: clock, gate, credit, stamps, spine/sheaf/decision types | `Rust/src/` — default features `[]` (no libp2p) |
| **Cold-edge coordination** | Optional P2P gossip daemon | `Rust/src/p2p.rs`, `Rust/src/bin/p2p.rs` — requires `--features p2p` |
| **Not here** | DEC cochains, Burn solvers, MCP tools | [`umst-manifold`](https://github.com/tytolabs/umst-manifold), [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) |

Consumers (e.g. cartridge `ucrs-provenance`) depend on the library only. Do not imply UCRS sits on the manifold arena hot path.

---

## Honesty ledger (status @ `5a3df25`)

**One status pointer:** [`PROOF-STATUS.md`](PROOF-STATUS.md). Protocol detail: [`CREDIT-SYSTEM.md`](CREDIT-SYSTEM.md). Formal foundations notes: [`FOUNDATION.md`](FOUNDATION.md).

| Layer | Status | Evidence |
|:---|:---|:---|
| **Rust library** | Working | `cd Rust && cargo test` @ `0921552` → **59** passed, 0 failed (paste below) |
| **P2P daemon** | Optional / in progress | Feature-gated; not required for stamps |
| **Lean** | Mixed | L1–L2 proved; L3 partial; L4 axiom; L5–L8 **sorry stubs** — see [`PROOF-STATUS.md`](PROOF-STATUS.md). Treat L5–L8 as **Proposed (not yet built)** as proofs. |
| **Haskell QuickCheck** | Scaffold | 5 properties in `Haskell/test/Spec.hs` ([`PROOF-STATUS.md`](PROOF-STATUS.md)) |
| **Python sims** | Foundation | `Python/sim/` topology + drift studies |
| **Material evolution between vertebrae** | **Proposed (not yet built)** | `MaterialEvolutionFrontier.built = false` ([`design_sheaf.rs:89–107`](Rust/src/design_sheaf.rs)) |
| **Cohomology / memory H¹** | Seam only | `SheafCohomologySeam.built = false` ([`design_sheaf.rs:72–86`](Rust/src/design_sheaf.rs)) |

**Strengthen — do not soften:** UCRS does **not** store mix recipes, hydration outcomes, or contribution content — those live in cartridge research memory ([`contribution.v1`](https://github.com/tytolabs/umst-concrete-cartridge/blob/main/schemas/contribution.v1.json)). Infra-purity purged TNA / vault domain types from this crate; consumers own domain steering.

### Rust test paste (`origin/master` @ `0921552`)

```bash
git checkout 0921552   # or origin/master
cd Rust && cargo test
```

Summary (full run 2026-07-12, SHA `0921552b93939f49041e53d8e1ac2070d94cbd82`):

```text
lib unit tests:                 39 passed
credit_test:                     5 passed
gate_cd_drift_fixture:           2 passed
gate_full_conjunct_drift_fixture: 1 passed
integration_test:                4 passed
s0_crypto_ucrs_parity:           6 passed
wire_v2_fixture:                 2 passed
doc-tests:                       0
-------------------------------------------
TOTAL:                          59 passed; 0 failed
```

---

## Agent surface (guarantees)

| Guarantee | Contract | Remediation |
|:---|:---|:---|
| Sync admissibility | `gate_check` → `Admit` only if Landauer cost ≤ budget **and** CD on desync ψ | On `Reject`: free-run / wait; do not force sync ([`gate.rs:44–61`](Rust/src/gate.rs)) |
| Stamp monotonicity | `TemporalWitness::stamp` saturating-increments `ucrs_seq` | Use `Synthetic` / `WallOnly` tiers only when Tier-2 unavailable ([`observation.rs:192–213`](Rust/src/observation.rs)) |
| Promotion credit floor | `MIN_PROMOTION_CREDIT_BITS` on contributor credit head | Raise peer credit via honest sync; do not forge stamps ([`observation.rs:77–78`](Rust/src/observation.rs)) |
| Default features | `default = []` — no libp2p in library consumers | Enable `--features p2p` only for mesh daemons |
| Env witness mode | `UMST_UCRS_WITNESS=live` \| `synthetic` (cartridge ingest) | `synthetic` for CI; `live` for real Tier-2 |

```rust
use umst_ucrs::{witness_for_agent, AgentConfig};

let config = AgentConfig::default();
let mut witness = witness_for_agent(&config);
let stamp = witness.stamp(); // UcrsTier2: ucrs_seq, phase_entropy_bits_q, credit_head_bits_q
```

---

## Scope — what this repo owns

| This repo owns | Lives elsewhere |
|:---|:---|
| Thermodynamic P2P clock sync + credit ledger | Material gate / DEC → [`umst-manifold`](https://github.com/tytolabs/umst-manifold) |
| `UcrsObservedAt` / spine / sheaf / generic decision infra | Cartridge physics + MCP → [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) |
| Rust library `umst_ucrs` for agents | Acting fiber → [`umst-formal`](https://github.com/tytolabs/umst-formal) |
| Lean / Haskell / Python scaffolds (status SSOT above) | Knowing fiber → [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) |

`tytolabs/umst-ucrs` is the **system of record for UCRS** — not an appendix of another repo.

---

## Repository layout

```text
umst-ucrs/
├── Rust/              # umst_ucrs library + optional p2p daemon (primary deliverable)
├── Lean/              # Formal track (see PROOF-STATUS.md — mixed proved / stub)
├── Python/sim/        # Topology + drift simulations
├── Haskell/           # QuickCheck properties (scaffold)
├── Docs/              # Logging + HLC policy
└── PROOF-STATUS.md · FOUNDATION.md · CREDIT-SYSTEM.md
```

```bash
cd Rust && cargo test && cargo build --release
# optional mesh daemon:
cargo build --release --features daemon
# publish check:
cargo publish --dry-run
```

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
|:---|:---|
| [`umst-manifold`](https://github.com/tytolabs/umst-manifold) | DEC carrier + thermodynamic gate host |
| [`umst-concrete-cartridge`](https://github.com/tytolabs/umst-concrete-cartridge) | Cementitious law + MCP surface |
| [`umst-formal`](https://github.com/tytolabs/umst-formal) | Economic-admissibility formal fiber |
| [`umst-formal-double-slit`](https://github.com/tytolabs/umst-formal-double-slit) | Observation / measurement-cost formal fiber |

---

## License

MIT License. Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO.

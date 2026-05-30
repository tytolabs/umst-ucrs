# Foundation: Relationship to Prior FCP Work

This document records which results from prior FCP papers and repos
are **used as foundations** by UMST-UCRS, and which are **extended**
with new theorems and implementations.

---

## Inherited Results (NOT duplicated — referenced)

### From [umst-formal-double-slit](https://github.com/tytolabs/umst-formal-double-slit) (FCP-DS)

| Result | File | Used by |
|--------|------|---------|
| `tensorDensity ρA ρB : DensityMatrix` | `TensorPartialTrace.lean` | TensorLandauer.lean |
| `partialTraceRightFin`, `partialTraceLeftFin` | `TensorPartialTrace.lean` | CoordinationCost.lean |
| `quantumMutualInfo (ρAB) = S(A) + S(B) - S(AB)` | `QuantumMutualInfo.lean` | CoordinationCost.lean |
| `quantumMutualInfo_product_eq_zero` | `QuantumMutualInfo.lean` | TensorLandauer.lean |
| `vonNeumannEntropy_tensorDensity` (axiom) | `QuantumMutualInfo.lean` | TensorLandauer.lean |
| `landauerBitEnergy T = k_B * T * log 2` | `UMSTCore.lean` | DesyncEnergy.lean |
| `landauerCostDiagonal_n` | `LandauerBound.lean` | TensorLandauer.lean |
| `requiredEnergy T I ≤ E ↔ I ≤ acquirableInfo T E` | `EpistemicGalois.lean` | CreditOptimality.lean |
| `KrausChannel`, `whichPathChannel` | `MeasurementChannel.lean` | ClockCoalgebra.lean |
| `principle_of_maximal_information_collapse` | `LandauerBound.lean` | CreditOptimality.lean |

### From [umst-formal](https://github.com/tytolabs/umst-formal) (FCP-I formal)

| Result | File | Used by |
|--------|------|---------|
| `Admissible old new : Prop` | `Gate.lean` | ClockCoalgebra.lean |
| `gateCheck` (sound + complete) | `Gate.lean` | Rust `gate.rs` |
| `KleisliArrow`, `kleisliCompose` | `Constitutional.lean` | ClockCoalgebra.lean |
| `WellTypedN`, `kleisliFoldWellTypedN` | `Constitutional.lean` | ClockCoalgebra.lean |
| `AdmissibleN_compose` (graded) | `Gate.lean` | EpochPatch.lean |
| `landauerBound` (Second Law) | `LandauerLaw.lean` | DesyncEnergy.lean |
| `landauer_galois_connection` | `EpistemicGalois` (both repos) | CreditOptimality.lean |

---

## New Results (THIS REPO)

### Lean Theorems (planned)

| Theorem | Description | Depends on |
|---------|-------------|------------|
| `landauerCost_tensor_product` | Joint cost of product states = sum of marginals | tensorDensity + landauerCostDiagonal_n |
| `landauerCost_joint_le_sum_marginals` | Subadditivity: joint ≤ sum | quantumMutualInfo + subadditivity |
| `coordinationCost_eq_mutualInfo_energy` | Gap = k_B T ln(2) · I(A:B) | Pure algebra on above |
| `coordinationCost_nonneg` | Coordination cost ≥ 0 | mutual info ≥ 0 |
| `desyncEnergy_bounded` | Desync cost ≤ budget iff admissible | EpistemicGalois + Gate |
| `epochPatch_wellTyped` | Epoch reindex is admissible (zero cost) | Gate.lean |
| `credit_greedy_optimal` | Greedy credit minimizes total cost | Galois + submodularity |
| `multiScaleCost_bounded` | Multi-scale sync ≤ sum of drift costs | kleisliFoldWellTypedN |

### Rust Implementation (planned)

| Component | Description | Validates |
|-----------|-------------|-----------|
| `clock.rs` | Local oscillator with drift model | Clock state structure |
| `p2p.rs` | libp2p sync protocol | Sync message as measurement |
| `gate.rs` | DUMSTO admissibility check | gateCheck (Lean-verified predicate) |
| `credit.rs` | Credit ledger + transfer protocol | Credit optimality theorem |
| `rapl.rs` | Intel RAPL energy telemetry | Real Landauer cost measurement |
| `landauer.rs` | Theoretical cost computation | Compare RAPL actual vs. Landauer floor |
| `telemetry.rs` | Prometheus metrics export | Observability |

---

## What This Repo Does NOT Do

1. **Does not re-prove** existing FCP-DS material (**540** theorems + **34** lemmas in **59** lake roots in [umst-formal-double-slit](https://github.com/tytolabs/umst-formal-double-slit); **584** line-start decls incl. all `Lean/*.lean` — counts from upstream `scripts/lean_declaration_stats.py`)
2. **Does not fork** Mathlib or any upstream dependency
3. **Does not introduce new axioms** — all new Lean theorems chain from existing infrastructure
4. **Does not claim** blockchain or cryptocurrency features — the credit system is
   a thermodynamic accounting tool, not a financial instrument
5. **Does not replace** NTP/PTP — it provides a formal framework for understanding
   their thermodynamic cost and for building Landauer-optimal sync protocols

---

## Migration Notes

### Safe practices for repo independence

1. **Lean dependency**: This repo's `lakefile.lean` imports Mathlib directly
   (same version as umst-formal-double-slit). It does NOT import
   umst-formal-double-slit as a Lake dependency — that would create a
   brittle cross-repo build. Instead, foundational results are
   referenced by citation and re-used by type-compatible definitions.

2. **Rust crate**: Independent `Cargo.toml` with no path dependencies
   on umst-prototype. Shared types (ThermodynamicState, Admissible)
   are re-implemented in Rust to match the Lean-verified specification.

3. **Git history**: Clean start. No `git subtree` or `git submodule`
   from prior repos. The relationship is documented here, not encoded
   in build tooling.

4. **CI independence**: Own GitHub Actions workflows. Lean CI builds
   only this repo's modules. Rust CI runs cargo test + clippy.
   No cross-repo CI triggers.

5. **Versioning**: Semantic versioning starting at 0.1.0. The first
   release (1.0.0) will coincide with paper submission.

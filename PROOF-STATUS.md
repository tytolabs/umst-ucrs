SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
SPDX-License-Identifier: MIT
# UCRS proof status (generated)

**Repo:** [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs)  
**Updated:** 2026-10-01  
**Lean toolchain:** Mathlib4 v4.14.0 (see `Lean/lakefile.lean`)

## Summary

The runtime's laws are proved once, in [`umst-formal`](https://github.com/tytolabs/umst-formal) `CoordinationContract`,
in Lean, Coq and Agda, and run there as Haskell properties. This repository holds the runtime and binds it to them:

| Binding | Location | What it checks |
|---------|----------|----------------|
| Contract interface | `Lean/RuntimeContract.lean` | each law the runtime relies on, restated and closed by the umst-formal theorem at the pinned commit |
| Refinement tests | `Rust/tests/coordination_contract.rs` | the same laws as properties of the runtime's functions; `gate_check` admits exactly the states the contract admits |
| Rust unit tests | `Rust/src/`, `Rust/tests/` | the runtime |

| Law | umst-formal (Lean · Coq · Agda · Haskell) |
|-----|-------------------------------------------|
| Landauer cost nonnegative and additive | `landauerCostJoules_nonneg`, `landauerCostJoules_add` |
| Admission bounds the cost by budget and desync (Clausius–Duhem) | `admitted_cost_bounded` |
| An admitted sync never raises desync nor lowers total cost | `gatedSync_second_law` |
| Clock drift monotone along any nonnegative trace | `clockRun_monotone` |
| A faulty cohort leaves honest credit unchanged | `honestCredits_append_faulty` |
| Wire sequence advances by exactly the step count | `wireNext_iterate` |

The former local modules (Lean `Ucrs/L1`–`L8`, Haskell properties) are preserved at tag
`archive/formal-move-2026-10-01`; their content is subsumed by the laws above.

## Formal import status

`Lean/lakefile.lean` requires `umst-formal` at a pinned commit and Mathlib4 v4.14.0.

## Manifold catalog (Track F)

UCRS Lean roots are listed in [`umst-manifold/artifacts/ucrs-catalog.json`](https://github.com/tytolabs/umst-manifold/blob/main/artifacts/ucrs-catalog.json) as a tertiary fiber preview pending unified merge.

## CI

| Workflow | Path |
|----------|------|
| Rust | `.github/workflows/rust.yml` |
| Lean | `.github/workflows/lean.yml` |

## Regenerate

```bash
cd Lean && lake build
cd ../Haskell && cabal test all
cd ../Rust && cargo test
```

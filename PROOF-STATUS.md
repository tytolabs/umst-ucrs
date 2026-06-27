# UCRS proof status (generated)

**Repo:** [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs)  
**Updated:** 2026-06-27  
**Lean toolchain:** Mathlib4 v4.14.0 (see `Lean/lakefile.lean`)

## Summary

| Track | Location | Status | `sorry` count |
|-------|----------|--------|---------------|
| L1 Landauer nonneg | `Lean/Ucrs/L1_LandauerNonneg.lean` | **Proved** (from `umst-formal` `LandauerEinsteinBridge`) | 0 |
| L2 Tensor additivity | `Lean/Ucrs/L2_TensorLandauer.lean` | **Proved** (`ring`) | 0 |
| L3 Credit greedy | `Lean/Ucrs/L3_CreditGreedy.lean` | **Partial theorem** (Landauer bound; full greedy pending) | 0 |
| L4 Gate admit | `Lean/Ucrs/L4_GateAdmit.lean` | **Axiom** | 0 |
| L5–L8 | `Lean/Ucrs/L5_*.lean` … `L8_*.lean` | **Sorry stub** (`theorem … : True := by sorry`) | 1 each |
| Legacy scaffold | `Lean/TensorLandauer.lean` | Axioms | 0 |
| Haskell QuickCheck | `Haskell/test/Spec.hs` | 5 properties | — |
| Rust unit tests | `Rust/src/`, `Rust/tests/` | Active | — |

## Formal import status (U4)

UCRS Lean `lakefile.lean` requires **`umst-formal`** (`LandauerEinsteinBridge`) + Mathlib4 v4.14.0.

## Manifold catalog (Track F)

UCRS Lean roots are listed in [`umst-manifold/artifacts/ucrs-catalog.json`](https://github.com/tytolabs/umst-manifold/blob/main/artifacts/ucrs-catalog.json) as a tertiary fiber preview pending unified merge.

## CI

| Workflow | Path |
|----------|------|
| Rust | `.github/workflows/rust.yml` |
| Haskell | `.github/workflows/haskell.yml` |
| Lean | `.github/workflows/lean.yml` |

## Regenerate

```bash
cd Lean && lake build
cd ../Haskell && cabal test all
cd ../Rust && cargo test
```

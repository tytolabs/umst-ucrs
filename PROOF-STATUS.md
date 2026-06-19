# UCRS proof status (generated template)

**Repo:** [`umst-ucrs`](https://github.com/tytolabs/umst-ucrs)  
**Updated:** 2026-06-19  
**Lean toolchain:** Mathlib4 v4.16.0 (see `Lean/lakefile.lean`)

## Summary

| Track | Location | Status | `sorry` count |
|-------|----------|--------|---------------|
| Tensor Landauer | `Lean/TensorLandauer.lean` | Scaffold — axioms only | 0 |
| Credit optimality | `Lean/` (planned) | Not started | — |
| Clock coalgebra | `Lean/` (planned) | Not started | — |
| Haskell QuickCheck | `Haskell/test/Spec.hs` | 5 property stubs | — |
| Rust unit tests | `Rust/src/`, `Rust/tests/` | Active | — |

## Lean axioms (explicit until proved)

| Axiom | File | Intent |
|-------|------|--------|
| `landauer_nonneg` | `TensorLandauer.lean` | Second-law floor at T > 0 |
| `tensor_landauer_add` | `TensorLandauer.lean` | QMI / tensor product bridge |
| `credit_greedy_optimal` | `TensorLandauer.lean` | Greedy peer selection = min Landauer |

## Cross-repo citations (no Lake dependency)

- `umst-formal` — `Gate.lean` `gateCheck`
- `umst-formal-double-slit` — tensor Landauer identities
- `umst-manifold` — `ThermodynamicGate` runtime host

## CI

| Workflow | Path |
|----------|------|
| Rust | `.github/workflows/rust.yml` |
| Haskell | `.github/workflows/haskell.yml` |
| Lean | `.github/workflows/lean.yml` |

## Regenerate

```bash
# After Lean build:
cd Lean && lake build && lake env lean --version
# Update this file's sorry/axiom table manually until auto-export lands.
```

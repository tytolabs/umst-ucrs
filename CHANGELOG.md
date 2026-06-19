# Changelog

All notable changes to **UMST-UCRS** (`tytolabs/umst-ucrs`) are documented here.

**Identity:** Independent public repo — Universal Calendar Resolution Spine (UCRS). Not coupled to FCP-VI/Zenodo as repo identity; see [`README.md`](README.md) and [`FOUNDATION.md`](FOUNDATION.md).

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [1.0.0] - 2026-06-19

> **Release note:** CHANGELOG entry prepared for 1.0.0. **Do not create a git tag** without explicit maintainer approval.

### Added

- **P2P runtime** (`p2p` feature): libp2p noise/yamux/mDNS/GossipSub daemon (`umst-ucrs-p2p`) with gate-guarded outbound sync.
- **Observability**: Prometheus HTTP `:9090/metrics`, `ucrs_sync_overhead_ratio` histogram, Grafana dashboard (`Docs/grafana-ucrs-dashboard.json`).
- **RAPL**: Linux sysfs path; macOS `powermetrics` fallback (`#[cfg(ucrs_skip_powermetrics)]` on CI).
- **Proof mirrors**: Haskell QuickCheck (5 properties), Lean L1–L8 scaffold (`Lean/Ucrs/`), shared `fixtures/wire_v2_observed_at.json`.
- **Ship**: Multi-stage `Dockerfile`, `scripts/umst-ucrs.service` systemd unit.

### Changed

- `TemporalWitness` + `observed_at.v2` integer wire parity with `umst-concrete-cartridge`.
- Byzantine detections export `ucrs_byzantine_detections_total`.

[1.0.0]: https://github.com/tytolabs/umst-ucrs/compare/v0.1.0...v1.0.0


- **FCP-DS / FCP-I headline counts** synced to sibling `lean_declaration_stats.py`: double-slit **540+34** / **574** roots-only (**584** all-Lean); meso **226+17** / **243** in **47** roots.

### Changed (2026-04-22)

- **`Rust/src/**`** — `rustfmt` so `cargo fmt -- --check` passes in clean checkouts (CI prep).

### Documentation (2026-04-22)

- **`README.md`** — CI status badge (GitHub Actions YAML follows in a commit pushed after `workflow` OAuth scope is granted). **`FOUNDATION.md`**, **`EXPERIMENTS_AND_ROADMAP.md`** — aligned FCP-DS / FCP-I Lean headline counts with sibling repos (`python3 scripts/lean_declaration_stats.py` in `umst-formal` / `umst-formal-double-slit`): **59** roots, **537** `theorem` + **34** `lemma` (**571** roots-only; **581** all `Lean/*.lean` for double-slit); **45** roots, **221** + **17** for meso layer; **0** `sorry`; **1** documented upstream project axiom (`physicalSecondLaw`). Supersedes legacy “515 theorems / six axioms” phrasing.

### Added (2026-04-22)

- **This file** — root changelog for consumer-facing documentation and API changes.

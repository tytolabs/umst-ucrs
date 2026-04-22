# Changelog

All notable changes to **UMST-UCRS** (`tytolabs/umst-ucrs`) are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed (2026-04-22)

- **`Rust/src/**`** — `rustfmt` so `cargo fmt -- --check` passes in clean checkouts (CI prep).

### Documentation (2026-04-22)

- **`README.md`** — CI status badge (GitHub Actions YAML follows in a commit pushed after `workflow` OAuth scope is granted). **`FOUNDATION.md`**, **`EXPERIMENTS_AND_ROADMAP.md`** — aligned FCP-DS / FCP-I Lean headline counts with sibling repos (`python3 scripts/lean_declaration_stats.py` in `umst-formal` / `umst-formal-double-slit`): **59** roots, **537** `theorem` + **34** `lemma` (**571** roots-only; **581** all `Lean/*.lean` for double-slit); **45** roots, **221** + **17** for meso layer; **0** `sorry`; **1** documented upstream project axiom (`physicalSecondLaw`). Supersedes legacy “515 theorems / six axioms” phrasing.

### Added (2026-04-22)

- **This file** — root changelog for consumer-facing documentation and API changes.

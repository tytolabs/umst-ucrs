# Tier-1 HLC sidecar pattern

**Status:** Design note (2026-06-19)  
**Audience:** Cartridge MCP operators, `umst-ucrs` embedders  
**Companion:** [`LOGGING_POLICY.md`](LOGGING_POLICY.md) · [`contribution-stack-ucrs-timing-research.md`](https://github.com/tytolabs/MaOS-Workspace/blob/main/outputs/.plans/contribution-stack-ucrs-timing-research.md)

---

## Rule (non-negotiable)

**Never overwrite `ucrs_seq` with HLC, Lamport, or vector-clock values.**

Happens-before clocks (HLC, `uhlc-rs`, `hlc-gen`) are **Tier-1 merge helpers** for foreign events — MCP sidecar logs, OTLP spans, transport ordering. UCRS **`ucrs_seq`** remains the **authoritative monotonic key** for durable memory, credit ledgers, and promotion bundles.

```text
ingest foreign event
  → merge HLC into sidecar field (wall_anchor / hlc_logical)
  → gate_check + credit merge
  → TemporalWitness::stamp() advances ucrs_seq (Tier-2)
```

---

## Sidecar envelope

Store both layers explicitly:

| Field | Tier | Role |
|-------|------|------|
| `ucrs_seq` | T2 | Monotonic constitutional sequence — **never** derived from HLC |
| `phase_entropy_bits_q` | T2 | Landauer-accounted phase uncertainty |
| `credit_head_bits_q` | T2 | Influence ceiling witness |
| `wall_ms` | T1 | Diagnostic wall anchor only |
| `hlc_logical` (optional) | T1 | Foreign causal merge helper on ingest |
| `stamp_tier` | — | `UcrsTier2` \| `WallOnly` \| `Synthetic` \| `Absent` |

**Uplift path:** When re-stamping a Tier-1 sidecar log for promotion, merge HLC timestamps for ordering diagnostics, then emit a fresh `TemporalWitness::stamp()` — the new stamp gets a **new** `ucrs_seq`, not an HLC transplant.

---

## Consumer behavior

| Scenario | Behavior |
|----------|----------|
| HLC says A before B but `ucrs_seq_B < ucrs_seq_A` | Trust **`ucrs_seq`** for merge authority; log HLC disagreement |
| Laptop clock jumps backward | Freeze `ucrs_seq` advance until witness resync; Tier-1 `wall_ms` tagged suspect |
| Feature off (`UMST_UCRS_BIND=0`) | Explicit `stamp_tier: WallOnly` or `Absent` — never silent T2 |
| P2P gossip (future) | `ClockTick` may carry optional HLC; `ucrs_seq` still allocated by local witness |

---

## Rust sketch

```rust
// Ingest only — never mutate existing ucrs_seq
struct HlcSidecar {
    hlc_logical: u64,
    wall_anchor_ms: u64,
}

// After gate admits the event:
let stamp = witness.stamp(); // ucrs_seq += 1, Tier-2 fields populated
```

Libraries: [`uhlc-rs`](https://github.com/atolab/uhlc-rs), [`hlc-gen`](https://docs.rs/hlc-gen) for Tier-1 only. PTP (`statime`) is optional lab infrastructure — do not block MCP tool latency on PTP lock.

---

## Related

- [`witness_for_agent`](../Rust/src/lib.rs) — embedder entry for live `TemporalWitness`
- [`UMST_UCRS_WITNESS=live|synthetic`](https://github.com/tytolabs/umst-concrete-cartridge/blob/main/docs/AGENT_MCP.md) — cartridge session boundary
- RFC 3161 / Sigstore on **promotion bundles** — deferred; not per MCP call

© 2026 Studio TYTO — MIT where applicable.

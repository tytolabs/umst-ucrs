# UCRS logging policy — immutable observation stamps

**Status:** Draft policy (2026-06-18)  
**Audience:** Contributors to `umst-ucrs`, `umst-manifold`, and any open cartridge that emits audit logs.  
**Scope:** Public, operator-agnostic logging — no private runtime names in schemas or published docs.

---

## 1. Goal

Every **durable log record** (memory promotion, credit contribution, gate verdict export, MSDF layer frame, MCP receipt) should carry a **meaningful UCRS observation stamp**: an immutable value that binds the payload to thermodynamic time, not merely wall clock.

Pure FP reading: an observation is a **value** in the log preimage; record identity is `content_hash(preimage)` where the preimage **includes** the stamp. Mutating or omitting the stamp changes the hash — there is no silent retroactive edit.

---

## 2. Core type: `UcrsObservedAt`

Canonical fields (Tier-2 bind; aligns with `umst-ucrs` clock + credit ledger):

| Field | Type (wire) | Role |
|-------|-------------|------|
| `phase_entropy_bits` | `u64` fixed-point (see §5) or rational pair | Shannon entropy of phase uncertainty after drift update; Landauer-accounted |
| `ucrs_seq` | `u64` | Monotonic sync / accept sequence (primary ordering key) |
| `credit_head_bits` | `u64` fixed-point | Max peer credit at observation (influence ceiling witness) |
| `wall_ms` | `u64` | Tier-1 auxiliary: Unix epoch ms (never sole authority) |

**Ordering law (monotonic bind):**  
`obs₂` is valid after `obs₁` iff  
`ucrs_seq₂ > ucrs_seq₁` OR (`ucrs_seq₂ == ucrs_seq₁` AND `phase_entropy_bits₂ ≥ phase_entropy_bits₁`).

**Wire key for memory tiers:** `wire_at_key = ucrs_seq` (content-addressed slots use seq in preimage).

**Sampling:** Prefer `sample_from_agent(agent)` after a Landauer-accounted tick. Tests and offline replay use `synthetic_monotonic(seq, phase_entropy_bits)` with explicit `stamp_tier: Synthetic`.

---

## 3. Mandatory stamps — recommendation

| Verdict | Policy |
|---------|--------|
| **Production logs with UCRS feature on** | **Required** Tier-2 `UcrsObservedAt` on every `memory_record.v1` and `contribution.v1` |
| **Gate / thermo exports** | Required `observed_at` block; reject writes missing stamp when `ucrs_bind=required` |
| **Ephemeral debug** | May use Tier-1 (`wall_ms` only) only if `stamp_tier: WallOnly` is set explicitly in envelope |
| **Tests** | `stamp_tier: Synthetic` allowed; must not be mixed into production merge paths |

**Soundness:** Mandatory UCRS stamps are **sound** for audit and credit ledgers where ordering must survive clock skew and replay. They are **not** a substitute for cryptographic signatures on gossip wire (`ClockTick.sig` remains separate).

**Not required everywhere:** Raw Prometheus scrape buffers, stderr tracing, and one-shot CLI stdout may omit stamps if they are not persisted or merged into tiered memory.

---

## 4. Schema hooks

### `memory_record.v1`

```json
{
  "schema": "memory_record.v1",
  "content_id": "<sha3-256 of canonical preimage>",
  "observed_at": {
    "stamp_tier": "UcrsTier2",
    "phase_entropy_bits_q": 123456789,
    "phase_entropy_bits_scale": 1000000,
    "ucrs_seq": 42,
    "credit_head_bits_q": 0,
    "credit_head_bits_scale": 1000000,
    "wall_ms": 1718745600123
  },
  "payload": { }
}
```

Required when `ucrs_bind=required`: `observed_at` with `stamp_tier ∈ {UcrsTier2, Synthetic}` (Synthetic only in test profiles).

### `contribution.v1`

```json
{
  "schema": "contribution.v1",
  "observed_at": { },
  "peer_id": 1,
  "delta_credit_bits_q": 500000,
  "delta_credit_bits_scale": 1000000,
  "theorem_id": "…",
  "content_id": "…"
}
```

Credit accumulation hashes **must** include `observed_at` in preimage (greedy optimal mass invariant).

---

## 5. Degradation tiers (never silent)

| Tier | `stamp_tier` | When | Consumer behavior |
|------|--------------|------|-------------------|
| **T2** | `UcrsTier2` | Default when `UMST_UCRS_BIND=1` or feature `ucrs-bind` | Full monotonic merge, Hilbert layout keyed on `ucrs_seq` |
| **T1** | `WallOnly` | Feature off but logging enabled | Accept only into **non-authoritative** buffers; **no** tier-3 promotion |
| **T0** | `Absent` | Explicit `UMST_UCRS_BIND=0` + `UMST_UCRS_ALLOW_ABSENT=1` | Receipts tagged `observation_ungrounded: true`; merge to tier-2+ **rejected** |
| **Synth** | `Synthetic` | Tests / fixtures | Isolated from production merge graphs |

**Rule:** If UCRS is disabled, logs **must** declare `stamp_tier` explicitly — never omit the field and imply T2.

Environment flags (public names only):

- `UMST_UCRS_BIND=1` — require Tier-2 on persist paths  
- `UMST_UCRS_ALLOW_WALL_ONLY=1` — permit Tier-1 with visible tag  
- `UMST_UCRS_ALLOW_ABSENT=1` — permit Tier-0 (debug only)

---

## 6. Pure FP logging principles

1. **Immutable preimage** — Serialize canonical JSON (sorted keys, integer rationals for floats) before hash.  
2. **No float authority** — `phase_entropy_bits` and `credit_head_bits` on wire as `(q, scale)` integers; `f64` only at simulation boundary inside `umst-ucrs`.  
3. **Observation as value** — `UcrsObservedAt` is `Copy`/`Clone` data, not a handle to mutable clock state.  
4. **Content identity** — `content_id = H(canonical_bytes(preimage))`; preimage includes full `observed_at`.  
5. **Reject, don’t repair** — Merge functions return `RejectedUngrounded` if stamp tier insufficient — no backfill from `wall_ms`.  
6. **Open deps only** — Public crates depend on `umst-ucrs` + `umst-math`; no private operator crate in the persist path.

---

## 7. Pitfalls

| Pitfall | Mitigation |
|---------|------------|
| Wall clock as primary order | Use `ucrs_seq`; treat `wall_ms` as diagnostic only |
| Float drift in hashes | Integer rationals on wire; ε-bisim tests between `umst_ucrs` and `umst_math` |
| Silent feature-off | Explicit `stamp_tier`; CI fixture asserting `Absent` paths never promote |
| Over-stamping hot paths | Exempt non-persisted metrics; document in crate README |
| Synthetic in production | CI gate: `stamp_tier == Synthetic` ⇒ `merge_policy: test_only` |
| Gossip without seq | `ClockTick` should gain optional `ucrs_seq` field in `wire.v2` (future) |

---

## 8. Relation to existing crates

- **`umst-ucrs`** — Source of `LocalClock::phase_entropy_bits()`, `CreditLedger`, `ClockTick` wire (today float-heavy; migrate to §5 rationals in `wire.v2`).  
- **`umst-manifold`** — Gate verdict exports should embed `observed_at` when cartridges emit trace v2.  
- **Private operator runtimes** — May implement `sample_from_agent`; public policy does not depend on them.

---

## 9. Further opportunities

See parent audit summary § "Further opportunities" in the workspace handoff; highest priority:

1. Publish `UcrsObservedAt` in `umst-ucrs` (move from private impl pattern).  
2. Add `wire.v2` integer canonical `ClockTick`.  
3. JSON Schema publish for `memory_record.v1` / `contribution.v1` under `umst-ucrs/schemas/`.  
4. MCP tool receipts in concrete cartridge: require `observed_at` when `ucrs-bind` feature enabled.  
5. CI test: merge rejects `stamp_tier: Absent` when `UMST_UCRS_BIND=1`.

---

© 2026 Studio TYTO — MIT where applicable.

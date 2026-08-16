// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Shared fixture roundtrip: Rust `observed_at.v2` ↔ cartridge `wire_v2.rs`.

use umst_ucrs::observation::{ObservedAtV2Wire, UcrsObservedAt, WIRE_SCALE};

#[test]
fn wire_v2_fixture_json_roundtrip() {
    let raw = include_str!("../../fixtures/wire_v2_observed_at.json");
    let wire: ObservedAtV2Wire = serde_json::from_str(raw).expect(
        "shared wire_v2_observed_at.json fixture must parse ObservedAtV2Wire (FP §6 Track T.2)",
    );
    assert_eq!(wire.schema_version, "observed_at.v2");
    assert_eq!(wire.phase_entropy_bits_scale, Some(WIRE_SCALE));

    let obs = UcrsObservedAt::from_v2_wire(&wire);
    let back = obs.to_v2_wire();
    let json = serde_json::to_string(&back)
        .expect("UcrsObservedAt v2 roundtrip must serialize to JSON (FP §6 Track T.2)");
    let reparsed: ObservedAtV2Wire = serde_json::from_str(&json).expect(
        "UcrsObservedAt v2 roundtrip JSON must deserialize ObservedAtV2Wire (FP §6 Track T.2)",
    );
    assert_eq!(back, reparsed);
    assert_eq!(obs.ucrs_seq, wire.ucrs_seq);
}

#[test]
fn synthetic_maps_to_v2_integers() {
    let obs = UcrsObservedAt::synthetic(7, 2.5);
    let v2 = obs.to_v2_wire();
    assert_eq!(v2.ucrs_seq, Some(7));
    assert_eq!(v2.phase_entropy_bits_q, Some(2_500_000));
    assert_eq!(v2.schema_version, "observed_at.v2");
}

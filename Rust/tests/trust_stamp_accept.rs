//! Integration: `TrustAttested` warrant ↔ `UcrsObservedAt` durable accept binding (SEC-UCRS-STAMP).

use umst_ucrs::{
    DurableAccept, DurableAcceptWire, TemporalWitness, TrustAttestedWarrant, TrustCipherSuite,
    TrustStampReject, DURABLE_ACCEPT_SCHEMA_VERSION,
};

fn fixture_warrant(root: &str) -> TrustAttestedWarrant {
    TrustAttestedWarrant::from_trust_attested(
        "fleet-sec-test",
        "Federated",
        root,
        Some(4_102_444_800_000),
        TrustCipherSuite::nist_pqc_balanced_3(),
    )
}

#[test]
fn integration_trust_attested_binds_to_ucrs_stamp() {
    let mut witness = TemporalWitness::new(2033);
    let root = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let accept = witness
        .stamp_durable_accept(fixture_warrant(root))
        .expect("bind trust warrant to Tier-2 stamp");

    assert!(accept.observed_at.ucrs_seq.is_some());
    assert_eq!(accept.trust.attestation_chain_root, root);
    assert_eq!(accept.trust.scope, "Federated");
    assert_ne!(
        accept.trust.cipher_suite,
        TrustCipherSuite::classical_only()
    );
}

#[test]
fn integration_wire_json_roundtrip_preserves_trust_and_stamp() {
    let mut witness = TemporalWitness::new(42);
    let accept = witness
        .stamp_durable_accept(fixture_warrant(
            "feedfacefeedfacefeedfacefeedfacefeedfacefeedfacefeedfacefeedface",
        ))
        .unwrap();

    let wire = accept.to_wire();
    let json = serde_json::to_string(&wire).expect("wire serializes");
    let reparsed: DurableAcceptWire = serde_json::from_str(&json).expect("wire deserializes");
    assert_eq!(reparsed.schema_version, DURABLE_ACCEPT_SCHEMA_VERSION);

    let back = DurableAccept::from_wire(&reparsed).unwrap();
    assert_eq!(back.observed_at.ucrs_seq, accept.observed_at.ucrs_seq);
    assert_eq!(
        back.trust.attestation_chain_root,
        accept.trust.attestation_chain_root
    );
}

#[test]
fn integration_monotonic_seq_across_three_durable_accepts() {
    let mut witness = TemporalWitness::new(1);
    let root = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mut prior: Option<DurableAccept> = None;
    for _ in 0..3 {
        let next = witness.stamp_durable_accept(fixture_warrant(root)).unwrap();
        if let Some(ref p) = prior {
            assert!(next.is_monotonic_after(p));
        }
        prior = Some(next);
    }
}

#[test]
fn integration_suite_mismatch_warrant_still_roundtrips_on_wire() {
    let mut witness = TemporalWitness::new(7);
    let mut warrant =
        fixture_warrant("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    warrant.cipher_suite = TrustCipherSuite::classical_only();
    let accept = witness.stamp_durable_accept(warrant).unwrap();
    let wire = accept.to_wire();
    let back = DurableAccept::from_wire(&wire).unwrap();
    assert_eq!(back.trust.cipher_suite.sig, "ed25519");
}

#[test]
fn integration_rejects_schema_version_mismatch() {
    let mut witness = TemporalWitness::new(1);
    let accept = witness
        .stamp_durable_accept(fixture_warrant(
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        ))
        .unwrap();
    let mut wire = accept.to_wire();
    wire.schema_version = "durable_accept.v99".into();
    assert_eq!(
        DurableAccept::from_wire(&wire),
        Err(TrustStampReject::SchemaVersionMismatch)
    );
}

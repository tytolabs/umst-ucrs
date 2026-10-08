// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Durable accept stamps — bind [`UcrsObservedAt`] to `TrustAttested` warrants (S-Q4).
//!
//! UCRS owns **when + provenance** (`ucrs_seq`, phase/credit fields). Trust ledger SSOT
//! remains in consumer crates (`egoff::trust::ledger`); this module carries only the
//! foreign-key warrant (`attestation_chain_root`) and typed [`TrustCipherSuite`] wire
//! projection from [`umst_trust`] (SEC-TRUST-EXTRACT; no duplicate cipher identifiers).

use serde::{Deserialize, Serialize};

use crate::trust_wire_public::{CipherSuite, CipherSuiteWire, Trust, TrustWarrantWire};

use crate::observation::{ObservedAtV2Wire, TemporalWitness, UcrsObservedAt};

/// Wire schema id for durable gate accepts that carry trust + UCRS legs.
pub const DURABLE_ACCEPT_SCHEMA_VERSION: &str = "durable_accept.v0";

/// NIST PQC cipher-suite identifiers — wire projection of core [`CipherSuite`] (S-Q4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustCipherSuite {
    pub kem: String,
    pub sig: String,
    pub hash: String,
}

impl From<&CipherSuite> for TrustCipherSuite {
    fn from(s: &CipherSuite) -> Self {
        let wire: CipherSuiteWire = s.into();
        Self {
            kem: wire.kem,
            sig: wire.sig,
            hash: wire.hash,
        }
    }
}

impl From<CipherSuite> for TrustCipherSuite {
    fn from(s: CipherSuite) -> Self {
        (&s).into()
    }
}

impl From<CipherSuiteWire> for TrustCipherSuite {
    fn from(w: CipherSuiteWire) -> Self {
        Self {
            kem: w.kem,
            sig: w.sig,
            hash: w.hash,
        }
    }
}

impl TrustCipherSuite {
    /// Operator default: ML-KEM-768 + ML-DSA-65 + SHA3-256 (delegates to core ADT).
    #[must_use]
    pub fn nist_pqc_balanced_3() -> Self {
        CipherSuite::nist_pqc_balanced_3().into()
    }

    /// Classical opt-in posture (delegates to core ADT).
    #[must_use]
    pub fn classical_only() -> Self {
        CipherSuite::classical_only().into()
    }
}

/// Lightweight `MemoryProvenance::TrustAttested` warrant — FK into trust ledger, not the ledger.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustAttestedWarrant {
    pub authority_id: String,
    pub scope: String,
    /// Merkle / chain root hex — replay via `:trust history <root>` in egoff (S-Q4).
    pub attestation_chain_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_ms: Option<u64>,
    pub cipher_suite: TrustCipherSuite,
}

impl TrustAttestedWarrant {
    /// Build warrant from core [`Trust`] ADT (SEC-TRUST-EXTRACT SSOT).
    #[must_use]
    pub fn from_core_trust(trust: &Trust) -> Self {
        let wire = TrustWarrantWire::from(trust);
        Self {
            authority_id: wire.authority_id,
            scope: wire.scope,
            attestation_chain_root: wire.attestation_chain_root,
            expires_at_unix_ms: wire.expires_at_unix_ms,
            cipher_suite: wire.cipher_suite.into(),
        }
    }

    /// Build warrant from S-Q4 `TrustAttested` fields + explicit cipher suite.
    #[must_use]
    pub fn from_trust_attested(
        authority_id: impl Into<String>,
        scope: impl Into<String>,
        attestation_chain_root: impl Into<String>,
        expires_at_unix_ms: Option<u64>,
        cipher_suite: TrustCipherSuite,
    ) -> Self {
        Self {
            authority_id: authority_id.into(),
            scope: scope.into(),
            attestation_chain_root: attestation_chain_root.into(),
            expires_at_unix_ms,
            cipher_suite,
        }
    }

    /// Structural validation — does not consult trust ledger (consumer responsibility).
    pub fn validate(&self) -> Result<(), TrustStampReject> {
        if self.authority_id.trim().is_empty() {
            return Err(TrustStampReject::EmptyAuthority);
        }
        if self.scope.trim().is_empty() {
            return Err(TrustStampReject::EmptyScope);
        }
        if self.attestation_chain_root.trim().is_empty() {
            return Err(TrustStampReject::EmptyAttestationRoot);
        }
        if self.cipher_suite.kem.trim().is_empty()
            || self.cipher_suite.sig.trim().is_empty()
            || self.cipher_suite.hash.trim().is_empty()
        {
            return Err(TrustStampReject::IncompleteCipherSuite);
        }
        Ok(())
    }
}

/// Gate-admitted durable accept — UCRS stamp + trust warrant (no TrustLedger payload).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableAccept {
    pub observed_at: UcrsObservedAt,
    pub trust: TrustAttestedWarrant,
}

/// Cold-edge wire for durable accepts (`durable_accept.v0`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableAcceptWire {
    pub schema_version: String,
    pub observed_at: ObservedAtV2Wire,
    pub trust: TrustAttestedWarrant,
}

/// Reject reasons when binding trust warrants to UCRS stamps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustStampReject {
    EmptyAuthority,
    EmptyScope,
    EmptyAttestationRoot,
    IncompleteCipherSuite,
    MissingUcrsSeq,
    SchemaVersionMismatch,
    NonMonotonicSeq { prior: u64, new: u64 },
}

impl DurableAccept {
    /// Bind a validated trust warrant to an existing UCRS stamp.
    pub fn bind(
        observed_at: UcrsObservedAt,
        trust: TrustAttestedWarrant,
    ) -> Result<Self, TrustStampReject> {
        trust.validate()?;
        if observed_at.ucrs_seq.is_none() {
            return Err(TrustStampReject::MissingUcrsSeq);
        }
        Ok(Self { observed_at, trust })
    }

    /// Monotonicity: `other` is a valid successor accept after `self`.
    #[must_use]
    pub fn is_monotonic_after(&self, prior: &Self) -> bool {
        let Some(prev_seq) = prior.observed_at.ucrs_seq else {
            return false;
        };
        let Some(new_seq) = self.observed_at.ucrs_seq else {
            return false;
        };
        new_seq > prev_seq
    }

    /// Serialize to `durable_accept.v0` wire.
    #[must_use]
    pub fn to_wire(&self) -> DurableAcceptWire {
        DurableAcceptWire {
            schema_version: DURABLE_ACCEPT_SCHEMA_VERSION.into(),
            observed_at: self.observed_at.to_v2_wire(),
            trust: self.trust.clone(),
        }
    }

    /// Parse and validate inbound `durable_accept.v0` wire.
    pub fn from_wire(wire: &DurableAcceptWire) -> Result<Self, TrustStampReject> {
        if wire.schema_version != DURABLE_ACCEPT_SCHEMA_VERSION {
            return Err(TrustStampReject::SchemaVersionMismatch);
        }
        let observed_at = UcrsObservedAt::from_v2_wire(&wire.observed_at);
        Self::bind(observed_at, wire.trust.clone())
    }
}

impl TemporalWitness {
    /// Emit Tier-2 UCRS stamp and bind a `TrustAttested` warrant (durable accept path).
    pub fn stamp_durable_accept(
        &mut self,
        trust: TrustAttestedWarrant,
    ) -> Result<DurableAccept, TrustStampReject> {
        let observed_at = self.stamp();
        DurableAccept::bind(observed_at, trust)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::StampTier;

    fn sample_warrant(root: &str) -> TrustAttestedWarrant {
        TrustAttestedWarrant::from_trust_attested(
            "operator-alpha",
            "Device",
            root,
            None,
            TrustCipherSuite::nist_pqc_balanced_3(),
        )
    }

    #[test]
    fn stamp_durable_accept_emits_seq_and_trust_root() {
        let mut w = TemporalWitness::new(1);
        let accept = w
            .stamp_durable_accept(sample_warrant(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ))
            .expect("valid warrant");
        assert_eq!(accept.observed_at.stamp_tier, StampTier::UcrsTier2);
        assert_eq!(accept.observed_at.ucrs_seq, Some(1));
        assert_eq!(
            accept.trust.attestation_chain_root,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(accept.trust.cipher_suite.kem, "ml-kem-768");
    }

    #[test]
    fn durable_accept_monotonic_across_stamps() {
        let mut w = TemporalWitness::new(3);
        let root = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let a = w.stamp_durable_accept(sample_warrant(root)).unwrap();
        let b = w.stamp_durable_accept(sample_warrant(root)).unwrap();
        assert!(b.is_monotonic_after(&a));
    }

    #[test]
    fn empty_attestation_root_rejects() {
        let mut w = TemporalWitness::new(1);
        let err = w
            .stamp_durable_accept(sample_warrant(""))
            .expect_err("empty root");
        assert_eq!(err, TrustStampReject::EmptyAttestationRoot);
    }

    #[test]
    fn durable_accept_wire_roundtrip() {
        let mut w = TemporalWitness::new(9);
        let accept = w
            .stamp_durable_accept(sample_warrant(
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            ))
            .unwrap();
        let wire = accept.to_wire();
        let back = DurableAccept::from_wire(&wire).unwrap();
        assert_eq!(back, accept);
        assert_eq!(wire.schema_version, DURABLE_ACCEPT_SCHEMA_VERSION);
    }

    #[test]
    fn from_core_trust_matches_manual_warrant() {
        let trust = Trust::bootstrap_unknown();
        let from_core = TrustAttestedWarrant::from_core_trust(&trust);
        let manual = TrustAttestedWarrant::from_trust_attested(
            "bootstrap",
            "Ephemeral",
            trust.chain.chain_root_hex.clone(),
            None,
            TrustCipherSuite::nist_pqc_balanced_3(),
        );
        assert_eq!(from_core, manual);
    }

}

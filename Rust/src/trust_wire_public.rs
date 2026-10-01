// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! W-63 public CI trust wire facade — not SSOT.
//!
//! Enable feature `foundations-ssot` for [`umst_trust`] wire projection from
//! `umst_algebra::crypto::trust`.

/// NIST PQC cipher-suite identifiers (public-ladder subset for durable accept).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CipherSuite {
    pub kem: String,
    pub sig: String,
    pub hash: String,
}

impl CipherSuite {
    #[must_use]
    pub fn nist_pqc_balanced_3() -> Self {
        Self {
            kem: "ml-kem-768".into(),
            sig: "ml-dsa-65".into(),
            hash: "sha3-256".into(),
        }
    }

    #[must_use]
    pub fn classical_only() -> Self {
        Self {
            kem: "x25519".into(),
            sig: "ed25519".into(),
            hash: "sha3-256".into(),
        }
    }
}

/// UCRS wire cipher suite — kem/sig/hash legs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CipherSuiteWire {
    pub kem: String,
    pub sig: String,
    pub hash: String,
}

impl From<&CipherSuite> for CipherSuiteWire {
    fn from(s: &CipherSuite) -> Self {
        Self {
            kem: s.kem.clone(),
            sig: s.sig.clone(),
            hash: s.hash.clone(),
        }
    }
}

impl From<CipherSuite> for CipherSuiteWire {
    fn from(s: CipherSuite) -> Self {
        (&s).into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustScope {
    Ephemeral,
    Device,
    Federated,
    HighAssurance,
    HardwareRooted,
}

fn trust_scope_label(scope: TrustScope) -> &'static str {
    match scope {
        TrustScope::Ephemeral => "Ephemeral",
        TrustScope::Device => "Device",
        TrustScope::Federated => "Federated",
        TrustScope::HighAssurance => "HighAssurance",
        TrustScope::HardwareRooted => "HardwareRooted",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttestationChain {
    pub chain_root_hex: String,
    pub expires_at_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trust {
    pub authority: String,
    pub scope: TrustScope,
    pub suite: CipherSuite,
    pub chain: AttestationChain,
}

impl Trust {
    #[must_use]
    pub fn bootstrap_unknown() -> Self {
        Self {
            authority: "bootstrap".into(),
            scope: TrustScope::Ephemeral,
            suite: CipherSuite::nist_pqc_balanced_3(),
            chain: AttestationChain {
                chain_root_hex: "0000000000000000000000000000000000000000000000000000000000000000"
                    .into(),
                expires_at_unix_ms: None,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustWarrantWire {
    pub authority_id: String,
    pub scope: String,
    pub attestation_chain_root: String,
    pub expires_at_unix_ms: Option<u64>,
    pub cipher_suite: CipherSuiteWire,
}

impl From<&Trust> for TrustWarrantWire {
    fn from(t: &Trust) -> Self {
        Self {
            authority_id: t.authority.clone(),
            scope: trust_scope_label(t.scope).into(),
            attestation_chain_root: t.chain.chain_root_hex.clone(),
            expires_at_unix_ms: t.chain.expires_at_unix_ms,
            cipher_suite: (&t.suite).into(),
        }
    }
}

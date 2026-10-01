// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
#![cfg(feature = "foundations-ssot")]
//! §14bis.f-S-1 — `Trust` ADT ↔ UCRS wire parity (SEC-TRUST-EXTRACT).
//!
//! Relocated from `umst-algebra/tests/` (OP5A-PARITY-RELOC; SSOT: `docs/OP5_EXCEPTION_UMST_ALGEBRA.md` §3.2.1).

use umst_algebra::crypto::{CipherSuite, Trust};
use umst_ucrs::{TrustAttestedWarrant, TrustCipherSuite};

#[test]
fn ucrs_cipher_suite_delegates_to_algebra_default() {
    let algebra = CipherSuite::nist_pqc_balanced_3();
    let ucrs = TrustCipherSuite::nist_pqc_balanced_3();
    assert_eq!(ucrs.kem, algebra.kem);
    assert_eq!(ucrs.sig, algebra.sig);
    assert_eq!(ucrs.hash, algebra.hash);
}

#[test]
fn ucrs_cipher_suite_delegates_to_algebra_classical() {
    let algebra = CipherSuite::classical_only();
    let ucrs = TrustCipherSuite::classical_only();
    assert_eq!(ucrs.kem, algebra.kem);
    assert_eq!(ucrs.sig, algebra.sig);
    assert_eq!(ucrs.hash, algebra.hash);
}

#[test]
fn from_core_trust_warrant_matches_algebra_suite() {
    let trust = Trust::bootstrap_unknown();
    let warrant = TrustAttestedWarrant::from_core_trust(&trust);
    assert_eq!(warrant.cipher_suite.kem, trust.suite.kem);
    assert_eq!(warrant.cipher_suite.sig, trust.suite.sig);
    assert_eq!(warrant.cipher_suite.hash, trust.suite.hash);
    assert_eq!(warrant.scope, "Ephemeral");
}

#[test]
fn trust_compose_error_stays_algebra_local() {
    let a = Trust::bootstrap_unknown();
    let mut b = Trust::bootstrap_unknown();
    b.suite = CipherSuite::classical_only();
    assert!(a.compose(&b).is_err());
}

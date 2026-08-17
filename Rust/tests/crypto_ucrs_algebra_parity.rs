// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! S-0 — ε-bisim parity: `umst_algebra::crypto` ↔ `umst_ucrs::crypto` (R-3.9.x deepen).
//!
//! Relocated from `umst-algebra/tests/` (OP5A-PARITY-RELOC; SSOT: `docs/OP5_EXCEPTION_UMST_ALGEBRA.md` §3.2.1).
//! Deepens the historical `umst-ucrs` ↔ `umst-math` battery with explicit algebra↔ucrs
//! cross-verify for KEM / SIG / HASH. Does not claim new GREEN slices — evidence only.

mod common;

use common::TestResult;
use pqcrypto_dilithium::dilithium3;
use pqcrypto_sphincsplus::sphincssha2128ssimple;
use umst_algebra::crypto::hash::sha3_256::digest as algebra_digest;
use umst_algebra::crypto::kem::ml_kem_768::{
    decapsulate as a_decap, encapsulate as a_encap, KemError as AlgebraKemError,
    ML_KEM_768_CIPHERTEXT_BYTES, ML_KEM_768_PUBLIC_KEY_BYTES, ML_KEM_768_SECRET_KEY_BYTES,
    ML_KEM_768_SHARED_SECRET_BYTES,
};
use umst_algebra::crypto::sig::ml_dsa_65::{
    sign as a_sign, verify as a_verify, SigError as AlgebraSigError,
};
use umst_algebra::crypto::sig::slh_dsa_128s;
use umst_ucrs::crypto::hash::sha3_256::digest as ucrs_digest;
use umst_ucrs::crypto::kem::ml_kem_768::{
    decapsulate as u_decap, encapsulate as u_encap, KemError as UcrsKemError,
    ML_KEM_768_CIPHERTEXT_BYTES as U_CT, ML_KEM_768_PUBLIC_KEY_BYTES as U_PK,
    ML_KEM_768_SECRET_KEY_BYTES as U_SK, ML_KEM_768_SHARED_SECRET_BYTES as U_SS,
};
use umst_ucrs::crypto::sig::ml_dsa_65::{
    sign as u_sign, verify as u_verify, SigError as UcrsSigError,
};

/// NIST SHA3-256("") — empty message (FIPS 202).
const EMPTY_SHA3_256: [u8; 32] = [
    0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62,
    0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a,
];

/// NIST SHA3-256("abc") short-message KAT (FIPS 202).
const ABC_SHA3_256: [u8; 32] = [
    0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd,
    0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32,
];

/// SHA3-256(`b"umst-s0-parity-kat-long-message-v1" || 0..=255`) — multi-block KAT witness.
const LONG_SHA3_256: [u8; 32] = [
    0x65, 0x34, 0xa5, 0x09, 0x11, 0xa7, 0x95, 0x69, 0xee, 0x3c, 0x5a, 0xe0, 0xd1, 0x72, 0x48, 0x9f,
    0xe8, 0x3f, 0x95, 0xb2, 0x0d, 0xea, 0x5a, 0x41, 0xec, 0x75, 0xa9, 0x0a, 0x77, 0xfc, 0x4f, 0x14,
];

fn long_kat_preimage() -> Vec<u8> {
    let mut msg = b"umst-s0-parity-kat-long-message-v1".to_vec();
    msg.extend(0u8..=255);
    msg
}

#[test]
fn r394_algebra_ucrs_kem_byte_width_parity() {
    assert_eq!(ML_KEM_768_PUBLIC_KEY_BYTES, U_PK);
    assert_eq!(ML_KEM_768_SECRET_KEY_BYTES, U_SK);
    assert_eq!(ML_KEM_768_CIPHERTEXT_BYTES, U_CT);
    assert_eq!(ML_KEM_768_SHARED_SECRET_BYTES, U_SS);
    assert_eq!(ML_KEM_768_SHARED_SECRET_BYTES, 32);
}

#[test]
fn r395_hash_empty_and_abc_kat_algebra_ucrs() -> TestResult {
    assert_eq!(algebra_digest(&[])?, ucrs_digest(&[])?);
    assert_eq!(algebra_digest(&[])?, EMPTY_SHA3_256);
    let abc = b"abc";
    assert_eq!(algebra_digest(abc)?, ucrs_digest(abc)?);
    assert_eq!(algebra_digest(abc)?, ABC_SHA3_256);
    Ok(())
}

#[test]
fn r396_kem_algebra_encap_ucrs_decap() -> TestResult {
    let (pk, sk) = umst_algebra::crypto::kem::ml_kem_768::keypair_bytes()?;
    let (ss_a, ct) = a_encap(&pk, &[])?;
    let ss_u = u_decap(&sk, &ct)?;
    assert_eq!(ss_a, ss_u);
    Ok(())
}

#[test]
fn r397_kem_ucrs_encap_algebra_decap() -> TestResult {
    let (pk, sk) = umst_ucrs::crypto::kem::ml_kem_768::keypair_bytes()?;
    let (ss_u, ct) = u_encap(&pk, &[])?;
    let ss_a = a_decap(&sk, &ct)?;
    assert_eq!(ss_u, ss_a);
    Ok(())
}

#[test]
fn r398_ml_dsa_algebra_sign_ucrs_verify() -> TestResult {
    let msg = b"algebra->ucrs ML-DSA cross-verify";
    let (pk, sk) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
    let sig = a_sign(msg, &sk, &pk)?;
    u_verify(&sig, msg, &pk)?;
    Ok(())
}

#[test]
fn r399_ml_dsa_ucrs_sign_algebra_verify() -> TestResult {
    let msg = b"ucrs->algebra ML-DSA cross-verify";
    let (pk, sk) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
    let sig = u_sign(msg, &sk, &pk)?;
    a_verify(&sig, msg, &pk)?;
    Ok(())
}

#[test]
fn r39a_slh_dsa_cross_verify_single_shot() -> TestResult {
    let msg = b"S-0 SLH-DSA algebra<->ucrs single-shot";
    let (pk_a, sk_a) = slh_dsa_128s::keypair_bytes();
    let sig_a = slh_dsa_128s::sign(msg, &sk_a, &pk_a)?;
    umst_ucrs::crypto::sig::slh_dsa_128s::verify(&sig_a, msg, &pk_a)?;

    let (pk_u, sk_u) = umst_ucrs::crypto::sig::slh_dsa_128s::keypair_bytes();
    let sig_u = umst_ucrs::crypto::sig::slh_dsa_128s::sign(msg, &sk_u, &pk_u)?;
    slh_dsa_128s::verify(&sig_u, msg, &pk_u)?;
    Ok(())
}

#[test]
fn r39b_sig_byte_width_parity_via_pqclean() {
    let (pk_a, sk_a) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
    let (pk_u, sk_u) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
    assert_eq!(pk_a.len(), dilithium3::public_key_bytes());
    assert_eq!(sk_a.len(), dilithium3::secret_key_bytes());
    assert_eq!(pk_u.len(), pk_a.len());
    assert_eq!(sk_u.len(), sk_a.len());

    let (spk_a, ssk_a) = slh_dsa_128s::keypair_bytes();
    let (spk_u, ssk_u) = umst_ucrs::crypto::sig::slh_dsa_128s::keypair_bytes();
    assert_eq!(spk_a.len(), sphincssha2128ssimple::public_key_bytes());
    assert_eq!(ssk_a.len(), sphincssha2128ssimple::secret_key_bytes());
    assert_eq!(spk_u.len(), spk_a.len());
    assert_eq!(ssk_u.len(), ssk_a.len());
}

#[test]
fn r39c_hash_long_message_kat_algebra_ucrs() -> TestResult {
    let msg = long_kat_preimage();
    assert_eq!(algebra_digest(&msg)?, ucrs_digest(&msg)?);
    assert_eq!(algebra_digest(&msg)?, LONG_SHA3_256);
    Ok(())
}

#[test]
fn r39d_kem_same_crate_roundtrip_shared_secret_width() -> TestResult {
    let (pk_a, sk_a) = umst_algebra::crypto::kem::ml_kem_768::keypair_bytes()?;
    let (ss_a, ct_a) = a_encap(&pk_a, &[])?;
    assert_eq!(ss_a.len(), ML_KEM_768_SHARED_SECRET_BYTES);
    assert_eq!(a_decap(&sk_a, &ct_a)?, ss_a);

    let (pk_u, sk_u) = umst_ucrs::crypto::kem::ml_kem_768::keypair_bytes()?;
    let (ss_u, ct_u) = u_encap(&pk_u, &[])?;
    assert_eq!(ss_u.len(), U_SS);
    assert_eq!(u_decap(&sk_u, &ct_u)?, ss_u);
    Ok(())
}

#[test]
fn r39e_malformed_kem_error_parity() {
    assert!(matches!(
        a_encap(&[], &[]),
        Err(AlgebraKemError::MalformedInput)
    ));
    assert!(matches!(
        u_encap(&[], &[]),
        Err(UcrsKemError::MalformedInput)
    ));
    assert!(matches!(
        a_decap(&[], &[]),
        Err(AlgebraKemError::MalformedInput)
    ));
    assert!(matches!(
        u_decap(&[], &[]),
        Err(UcrsKemError::MalformedInput)
    ));
}

#[test]
fn r39f_malformed_sig_error_parity() {
    assert!(matches!(
        a_verify(&[], &[], &[]),
        Err(AlgebraSigError::MalformedInput)
    ));
    assert!(matches!(
        u_verify(&[], &[], &[]),
        Err(UcrsSigError::MalformedInput)
    ));
    assert!(matches!(
        slh_dsa_128s::verify(&[], &[], &[]),
        Err(umst_algebra::crypto::sig::slh_dsa_128s::SigError::MalformedInput)
    ));
    assert!(matches!(
        umst_ucrs::crypto::sig::slh_dsa_128s::verify(&[], &[], &[]),
        Err(umst_ucrs::crypto::sig::slh_dsa_128s::SigError::MalformedInput)
    ));
}

#[test]
fn r39g_kem_cross_parity_stress_32() -> TestResult {
    for i in 0..32 {
        if i % 2 == 0 {
            let (pk, sk) = umst_algebra::crypto::kem::ml_kem_768::keypair_bytes()?;
            let (ss_a, ct) = a_encap(&pk, &[])?;
            assert_eq!(u_decap(&sk, &ct)?, ss_a);
        } else {
            let (pk, sk) = umst_ucrs::crypto::kem::ml_kem_768::keypair_bytes()?;
            let (ss_u, ct) = u_encap(&pk, &[])?;
            assert_eq!(a_decap(&sk, &ct)?, ss_u);
        }
    }
    Ok(())
}

#[test]
fn r39h_ml_dsa_sig_cross_parity_stress_16() -> TestResult {
    let msg = b"S-0 ML-DSA algebra<->ucrs stress";
    for i in 0..16 {
        if i % 2 == 0 {
            let (pk, sk) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
            let sig = a_sign(msg, &sk, &pk)?;
            u_verify(&sig, msg, &pk)?;
        } else {
            let (pk, sk) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
            let sig = u_sign(msg, &sk, &pk)?;
            a_verify(&sig, msg, &pk)?;
        }
    }
    Ok(())
}

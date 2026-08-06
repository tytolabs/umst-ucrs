//! S-0 — ε-bisim parity: `umst_ucrs::crypto` vs `umst_math::crypto` + `umst_algebra::crypto` (R-3.9.x deepen).

use umst_algebra::crypto::hash::sha3_256::digest as algebra_digest;
use umst_algebra::crypto::kem::ml_kem_768::{
    KemError as AlgebraKemError, ML_KEM_768_SHARED_SECRET_BYTES as A_SS,
};
use umst_algebra::crypto::sig::ml_dsa_65::SigError as AlgebraSigError;
use umst_algebra::crypto::sig::slh_dsa_128s;
use umst_math::crypto::hash::sha3_256::digest as math_digest;
use umst_ucrs::crypto::hash::sha3_256::digest as ucrs_digest;
use umst_ucrs::crypto::kem::ml_kem_768::{
    decapsulate as u_decap, encapsulate as u_encap, KemError as UcrsKemError,
    ML_KEM_768_SHARED_SECRET_BYTES,
};
use umst_ucrs::crypto::sig::ml_dsa_65::{sign as u_sign, verify as u_verify, SigError as UcrsSigError};

/// NIST SHA3-256("abc") short-message KAT (FIPS 202).
const ABC_SHA3_256: [u8; 32] = [
    0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd,
    0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32,
];

fn long_kat_preimage() -> Vec<u8> {
    let mut msg = b"umst-s0-parity-kat-long-message-v1".to_vec();
    msg.extend(0u8..=255);
    msg
}

#[test]
fn r391_ucrs_kem_surface() {
    let _ = u_encap;
    let _ = u_decap;
}

#[test]
fn r392_ucrs_sig_surface() {
    let _ = u_sign;
    let _ = u_verify;
}

#[test]
fn r393_ucrs_hash_surface() {
    let _ = ucrs_digest(&[]);
}

#[test]
fn parity_sha3_256_empty_digest_math_ucrs() {
    assert_eq!(math_digest(&[]).unwrap(), ucrs_digest(&[]).unwrap());
}

#[test]
fn parity_sha3_256_abc_digest_math_ucrs_algebra() {
    let abc = b"abc";
    assert_eq!(math_digest(abc).unwrap(), ucrs_digest(abc).unwrap());
    assert_eq!(algebra_digest(abc).unwrap(), ucrs_digest(abc).unwrap());
    assert_eq!(ucrs_digest(abc).unwrap(), ABC_SHA3_256);
}

#[test]
fn parity_sha3_256_long_digest_tri_crate() {
    let msg = long_kat_preimage();
    assert_eq!(math_digest(&msg).unwrap(), ucrs_digest(&msg).unwrap());
    assert_eq!(algebra_digest(&msg).unwrap(), ucrs_digest(&msg).unwrap());
}

#[test]
fn parity_ml_kem_decaps_matches_math_encaps() {
    let (pk, sk) = umst_math::crypto::kem::ml_kem_768::keypair_bytes().expect("kp");
    let (ss, ct) = umst_math::crypto::kem::ml_kem_768::encapsulate(&pk, &[]).expect("enc");
    let ss_u = u_decap(&sk, &ct).expect("ucrs decap");
    assert_eq!(ss, ss_u);
}

#[test]
fn parity_ml_dsa_verify_across_crates() {
    let msg = b"ucrs/math ML-DSA cross-verify";
    let (pk, sk) = umst_math::crypto::sig::ml_dsa_65::keypair_bytes();
    let sig = umst_math::crypto::sig::ml_dsa_65::sign(msg, &sk, &pk).expect("sign");
    u_verify(&sig, msg, &pk).expect("ucrs verify");
}

#[test]
fn parity_sha3_256_empty_digest_algebra_ucrs() {
    assert_eq!(algebra_digest(&[]).unwrap(), ucrs_digest(&[]).unwrap());
}

#[test]
fn parity_ml_kem_decaps_matches_algebra_encaps() {
    let (pk, sk) = umst_algebra::crypto::kem::ml_kem_768::keypair_bytes().expect("kp");
    let (ss, ct) = umst_algebra::crypto::kem::ml_kem_768::encapsulate(&pk, &[]).expect("enc");
    let ss_u = u_decap(&sk, &ct).expect("ucrs decap");
    assert_eq!(ss, ss_u);
}

#[test]
fn parity_ml_dsa_verify_algebra_to_ucrs() {
    let msg = b"algebra/ucrs ML-DSA cross-verify";
    let (pk, sk) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
    let sig = umst_algebra::crypto::sig::ml_dsa_65::sign(msg, &sk, &pk).expect("sign");
    u_verify(&sig, msg, &pk).expect("ucrs verify");
}

#[test]
fn parity_ml_dsa_verify_ucrs_to_algebra() {
    let msg = b"ucrs/algebra ML-DSA cross-verify";
    let (pk, sk) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
    let sig = u_sign(msg, &sk, &pk).expect("sign");
    umst_algebra::crypto::sig::ml_dsa_65::verify(&sig, msg, &pk).expect("algebra verify");
}

#[test]
fn parity_slh_dsa_cross_verify_algebra_ucrs() {
    let msg = b"S-0 SLH-DSA ucrs<->algebra single-shot";
    let (pk_a, sk_a) = slh_dsa_128s::keypair_bytes();
    let sig_a = slh_dsa_128s::sign(msg, &sk_a, &pk_a).expect("algebra sign");
    umst_ucrs::crypto::sig::slh_dsa_128s::verify(&sig_a, msg, &pk_a).expect("ucrs verify");

    let (pk_u, sk_u) = umst_ucrs::crypto::sig::slh_dsa_128s::keypair_bytes();
    let sig_u = umst_ucrs::crypto::sig::slh_dsa_128s::sign(msg, &sk_u, &pk_u).expect("ucrs sign");
    slh_dsa_128s::verify(&sig_u, msg, &pk_u).expect("algebra verify");
}

#[test]
fn parity_kem_shared_secret_width_tri_crate() {
    assert_eq!(ML_KEM_768_SHARED_SECRET_BYTES, A_SS);
    assert_eq!(ML_KEM_768_SHARED_SECRET_BYTES, 32);
}

#[test]
fn parity_malformed_kem_errors_ucrs_algebra() {
    assert!(matches!(
        u_encap(&[], &[]),
        Err(UcrsKemError::MalformedInput)
    ));
    assert!(matches!(
        umst_algebra::crypto::kem::ml_kem_768::encapsulate(&[], &[]),
        Err(AlgebraKemError::MalformedInput)
    ));
}

#[test]
fn parity_malformed_sig_errors_ucrs_algebra() {
    assert!(matches!(
        u_verify(&[], &[], &[]),
        Err(UcrsSigError::MalformedInput)
    ));
    assert!(matches!(
        umst_algebra::crypto::sig::ml_dsa_65::verify(&[], &[], &[]),
        Err(AlgebraSigError::MalformedInput)
    ));
}

/// NIST SHA3-256("") — empty message (FIPS 202).
const EMPTY_SHA3_256: [u8; 32] = [
    0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62,
    0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a,
];

#[test]
fn parity_sha3_256_empty_digest_matches_fips_vector() {
    assert_eq!(ucrs_digest(&[]).unwrap(), EMPTY_SHA3_256);
    assert_eq!(math_digest(&[]).unwrap(), EMPTY_SHA3_256);
    assert_eq!(algebra_digest(&[]).unwrap(), EMPTY_SHA3_256);
}

#[test]
fn parity_kem_byte_width_algebra_ucrs() {
    use umst_algebra::crypto::kem::ml_kem_768::{
        ML_KEM_768_CIPHERTEXT_BYTES as A_CT, ML_KEM_768_PUBLIC_KEY_BYTES as A_PK,
        ML_KEM_768_SECRET_KEY_BYTES as A_SK,
    };
    use umst_ucrs::crypto::kem::ml_kem_768::{
        ML_KEM_768_CIPHERTEXT_BYTES as U_CT, ML_KEM_768_PUBLIC_KEY_BYTES as U_PK,
        ML_KEM_768_SECRET_KEY_BYTES as U_SK,
    };
    assert_eq!(U_PK, A_PK);
    assert_eq!(U_SK, A_SK);
    assert_eq!(U_CT, A_CT);
}

#[test]
fn parity_kem_cross_stress_32_algebra_ucrs() {
    use umst_algebra::crypto::kem::ml_kem_768::{decapsulate as a_decap, encapsulate as a_encap};
    for i in 0..32 {
        if i % 2 == 0 {
            let (pk, sk) = umst_algebra::crypto::kem::ml_kem_768::keypair_bytes().expect("kp");
            let (ss_a, ct) = a_encap(&pk, &[]).expect("enc");
            let ss_u = u_decap(&sk, &ct).expect("dec");
            assert_eq!(ss_a, ss_u);
        } else {
            let (pk, sk) = umst_ucrs::crypto::kem::ml_kem_768::keypair_bytes().expect("kp");
            let (ss_u, ct) = u_encap(&pk, &[]).expect("enc");
            let ss_a = a_decap(&sk, &ct).expect("dec");
            assert_eq!(ss_u, ss_a);
        }
    }
}

#[test]
fn parity_ml_dsa_cross_stress_16_algebra_ucrs() {
    use umst_algebra::crypto::sig::ml_dsa_65::{sign as a_sign, verify as a_verify};
    let msg = b"S-0 ML-DSA ucrs<->algebra stress";
    for i in 0..16 {
        if i % 2 == 0 {
            let (pk, sk) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
            let sig = a_sign(msg, &sk, &pk).expect("sign");
            u_verify(&sig, msg, &pk).expect("verify");
        } else {
            let (pk, sk) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
            let sig = u_sign(msg, &sk, &pk).expect("sign");
            a_verify(&sig, msg, &pk).expect("verify");
        }
    }
}

#[test]
fn parity_slh_dsa_cross_stress_8_algebra_ucrs() {
    let msg = b"S-0 SLH-DSA ucrs<->algebra stress";
    for i in 0..8 {
        if i % 2 == 0 {
            let (pk, sk) = slh_dsa_128s::keypair_bytes();
            let sig = slh_dsa_128s::sign(msg, &sk, &pk).expect("sign");
            umst_ucrs::crypto::sig::slh_dsa_128s::verify(&sig, msg, &pk).expect("verify");
        } else {
            let (pk, sk) = umst_ucrs::crypto::sig::slh_dsa_128s::keypair_bytes();
            let sig = umst_ucrs::crypto::sig::slh_dsa_128s::sign(msg, &sk, &pk).expect("sign");
            slh_dsa_128s::verify(&sig, msg, &pk).expect("verify");
        }
    }
}

//! GREEN §14bis.f-S-0 — ε-bisim parity: `umst_ucrs::crypto` vs `umst_math::crypto` (R-3.9.x).

use umst_math::crypto::hash::sha3_256::digest as math_digest;
use umst_ucrs::crypto::hash::sha3_256::digest as ucrs_digest;
use umst_ucrs::crypto::kem::ml_kem_768::{decapsulate as u_decap, encapsulate as u_encap};
use umst_ucrs::crypto::sig::ml_dsa_65::{sign as u_sign, verify as u_verify};

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

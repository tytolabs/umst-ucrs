// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! U1 crypto facade parity — `shared_types::crypto` consumer contract (CELL_UCRS_READY_U1_CRYPTO).

use umst_ucrs::shared_types::crypto::{
    error::CryptoError,
    hash::sha3_256::digest,
    kem::ml_kem_768::{decapsulate, encapsulate, ML_KEM_768_SHARED_SECRET_BYTES},
    sig::ml_dsa_65::{keypair_bytes, sign, verify},
};

/// NIST SHA3-256("abc") short-message KAT (FIPS 202).
const ABC_SHA3_256: [u8; 32] = [
    0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd,
    0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32,
];

#[test]
fn shared_types_crypto_hash_morphism_roundtrip() {
    let empty = digest(&[]).expect("empty digest");
    assert_eq!(empty.len(), 32);

    let abc = digest(b"abc").expect("abc digest");
    assert_eq!(abc, ABC_SHA3_256);
}

#[test]
fn shared_types_crypto_kem_sig_surface() {
    let _ = CryptoError::HashMismatch;
    assert_eq!(ML_KEM_768_SHARED_SECRET_BYTES, 32);

    let (pk, sk) = umst_math::crypto::kem::ml_kem_768::keypair_bytes().expect("kp");
    let (ss, ct) = encapsulate(&pk, &[]).expect("encap");
    assert_eq!(ss.len(), ML_KEM_768_SHARED_SECRET_BYTES);
    let ss_back = decapsulate(&sk, &ct).expect("decap");
    assert_eq!(ss, ss_back);

    let (pk, sk) = keypair_bytes();
    let msg = b"u1-crypto-facade-sign";
    let sig = sign(msg, &sk, &pk).expect("sign");
    verify(&sig, msg, &pk).expect("verify");
}

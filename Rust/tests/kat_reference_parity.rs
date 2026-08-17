// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! S-0 — committed KAT reference fixture drives ucrs↔algebra parity (SEC-UCRS-PARITY deepen).

mod common;

use common::{hex_to_bytes32, load_s0_kat_reference, long_v1_preimage, TestResult};
use pqcrypto_dilithium::dilithium3;
use pqcrypto_kyber::kyber768;
use pqcrypto_sphincsplus::sphincssha2128ssimple;
use umst_algebra::crypto::hash::sha3_256::digest as algebra_digest;
use umst_ucrs::crypto::hash::sha3_256::digest as ucrs_digest;
use umst_ucrs::crypto::kem::ml_kem_768::{
    ML_KEM_768_CIPHERTEXT_BYTES, ML_KEM_768_PUBLIC_KEY_BYTES, ML_KEM_768_SECRET_KEY_BYTES,
    ML_KEM_768_SHARED_SECRET_BYTES,
};

#[test]
fn r3a0_fixture_schema_version_locked() -> TestResult {
    let kat = load_s0_kat_reference()?;
    assert_eq!(kat.schema_version, "s0_pqc_kat_reference.v0");
    assert!(kat.authority.contains("SEC-UCRS-PARITY"));
    Ok(())
}

#[test]
fn r3a1_sha3_empty_abc_long_kat_ucrs_algebra() -> TestResult {
    let kat = load_s0_kat_reference()?;
    let empty = hex_to_bytes32(&kat.sha3_256.empty_hex)?;
    let abc = hex_to_bytes32(&kat.sha3_256.abc_hex)?;
    let long = hex_to_bytes32(&kat.sha3_256.long_v1.digest_hex)?;

    assert_eq!(ucrs_digest(&[])?, empty);
    assert_eq!(algebra_digest(&[])?, empty);

    assert_eq!(ucrs_digest(b"abc")?, abc);
    assert_eq!(algebra_digest(b"abc")?, abc);

    let msg = long_v1_preimage(&kat.sha3_256.long_v1);
    assert_eq!(kat.sha3_256.long_v1.preimage_suffix, "0..=255");
    assert_eq!(ucrs_digest(&msg)?, long);
    assert_eq!(algebra_digest(&msg)?, long);
    Ok(())
}

#[test]
fn r3a2_ml_kem_byte_width_fixture_matches_pqclean_and_crates() -> TestResult {
    let kat = load_s0_kat_reference()?;
    assert_eq!(
        kat.ml_kem_768.public_key_bytes,
        kyber768::public_key_bytes()
    );
    assert_eq!(
        kat.ml_kem_768.secret_key_bytes,
        kyber768::secret_key_bytes()
    );
    assert_eq!(
        kat.ml_kem_768.ciphertext_bytes,
        kyber768::ciphertext_bytes()
    );
    assert_eq!(
        kat.ml_kem_768.shared_secret_bytes,
        kyber768::shared_secret_bytes()
    );

    assert_eq!(ML_KEM_768_PUBLIC_KEY_BYTES, kat.ml_kem_768.public_key_bytes);
    assert_eq!(ML_KEM_768_SECRET_KEY_BYTES, kat.ml_kem_768.secret_key_bytes);
    assert_eq!(ML_KEM_768_CIPHERTEXT_BYTES, kat.ml_kem_768.ciphertext_bytes);
    assert_eq!(
        ML_KEM_768_SHARED_SECRET_BYTES,
        kat.ml_kem_768.shared_secret_bytes
    );

    assert_eq!(
        umst_algebra::crypto::kem::ml_kem_768::ML_KEM_768_PUBLIC_KEY_BYTES,
        kat.ml_kem_768.public_key_bytes
    );
    Ok(())
}

#[test]
fn r3a3_sig_byte_width_fixture_matches_pqclean_and_crates() -> TestResult {
    let kat = load_s0_kat_reference()?;
    assert_eq!(
        kat.ml_dsa_65.public_key_bytes,
        dilithium3::public_key_bytes()
    );
    assert_eq!(
        kat.ml_dsa_65.secret_key_bytes,
        dilithium3::secret_key_bytes()
    );
    assert_eq!(
        kat.slh_dsa_128s.public_key_bytes,
        sphincssha2128ssimple::public_key_bytes()
    );
    assert_eq!(
        kat.slh_dsa_128s.secret_key_bytes,
        sphincssha2128ssimple::secret_key_bytes()
    );

    let (pk_u, sk_u) = umst_ucrs::crypto::sig::ml_dsa_65::keypair_bytes();
    assert_eq!(pk_u.len(), kat.ml_dsa_65.public_key_bytes);
    assert_eq!(sk_u.len(), kat.ml_dsa_65.secret_key_bytes);

    let (pk_a, sk_a) = umst_algebra::crypto::sig::ml_dsa_65::keypair_bytes();
    assert_eq!(pk_a.len(), pk_u.len());
    assert_eq!(sk_a.len(), sk_u.len());
    Ok(())
}

#[test]
fn r3a4_sha3_digest_width_fixture_locked() -> TestResult {
    let kat = load_s0_kat_reference()?;
    assert_eq!(kat.sha3_digest_bytes, 32);
    assert_eq!(ucrs_digest(&[])?.len(), kat.sha3_digest_bytes);
    Ok(())
}

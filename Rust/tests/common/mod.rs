//! Shared helpers for `umst-ucrs` integration tests.
#![allow(dead_code)]

use serde::Deserialize;
use std::path::PathBuf;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Committed S-0 PQC KAT reference (`tests/fixtures/s0_pqc_kat_reference.json`).
pub const S0_PQC_KAT_REFERENCE_JSON: &str = include_str!("../fixtures/s0_pqc_kat_reference.json");

#[derive(Debug, Deserialize)]
pub struct S0PqcKatReference {
    pub schema_version: String,
    pub authority: String,
    pub sha3_256: Sha3Kat,
    pub ml_kem_768: ByteWidthRow,
    pub ml_dsa_65: ByteWidthRow,
    pub slh_dsa_128s: ByteWidthRow,
    pub sha3_digest_bytes: usize,
}

#[derive(Debug, Deserialize)]
pub struct Sha3Kat {
    pub empty_hex: String,
    pub abc_hex: String,
    pub long_v1: LongSha3Kat,
}

#[derive(Debug, Deserialize)]
pub struct LongSha3Kat {
    pub preimage_prefix: String,
    pub preimage_suffix: String,
    pub digest_hex: String,
}

#[derive(Debug, Deserialize)]
pub struct ByteWidthRow {
    pub public_key_bytes: usize,
    pub secret_key_bytes: usize,
    #[serde(default)]
    pub ciphertext_bytes: usize,
    #[serde(default)]
    pub shared_secret_bytes: usize,
}

pub fn load_s0_kat_reference() -> Result<S0PqcKatReference, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(S0_PQC_KAT_REFERENCE_JSON)?)
}

pub fn hex_to_bytes32(hex: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(format!("expected 32-byte hex digest, got {} nibbles", hex.len()).into());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk)?;
        out[i] = u8::from_str_radix(pair, 16)?;
    }
    Ok(out)
}

pub fn long_v1_preimage(ref_kat: &LongSha3Kat) -> Vec<u8> {
    let mut msg = ref_kat.preimage_prefix.as_bytes().to_vec();
    msg.extend(0u8..=255);
    msg
}

/// Resolve fixture path for out-of-crate consumers (algebra parity harness).
pub fn s0_kat_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/s0_pqc_kat_reference.json")
}

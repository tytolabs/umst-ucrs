// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! Consumer contract: `umst_ucrs::shared_types::crypto` (Wave 1 · CELL_UCRS_READY_U1_CRYPTO).
//! PQC morphisms reachable via `hash` · `kem` · `sig` submodules — **not** daemon · p2p · agent_tick.
//!
//! **Morphisms (consumer hot path):** `sha3_256::digest` · `ml_kem_768::{encapsulate,decapsulate}` ·
//! `ml_dsa_65::{sign,verify}` · `slh_dsa_128s` · `CryptoError`.
//!
//! S-0 — Post-quantum cryptographic primitives (Rust engineering mirrors of `umst-formal` L-S0..L-S5 statements).
//!
//! Concrete bindings: PQClean via `pqcrypto-*` (`kyber768`, `dilithium3`, `sphincssha2128ssimple`) + FIPS 202 SHA3-256.

pub mod error;
pub mod hash;
pub mod kem;
pub mod sig;

pub use error::CryptoError;

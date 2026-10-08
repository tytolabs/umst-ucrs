// SPDX-FileCopyrightText: 2026 Santosh Prabhu Shenbagamoorthy and Santhosh Shyamsundar
// SPDX-License-Identifier: MIT
//! UMST-UCRS library crate.
//!
//! Consumer contract: `umst_ucrs::shared_types` (Wave 0–1 · observation · accept · wire · crypto · cast spine).
//! Spine locals: `umst_ucrs::ucrs_keep` (clock · credit · agent_tick · landauer_global).
//!
//! The binary (`src/main.rs`) is a thin wrapper: constructs [`ucrs_keep::AgentConfig`], starts the
//! simulation / P2P loop, and exports Prometheus metrics.

/// Durable accept stamps — `TrustAttested` warrant + `UcrsObservedAt` (S-Q4; no TrustLedger).
pub mod accept;
pub mod clock;
pub mod credit;
/// S-0 crypto parity — PQC reference (`umst_math::crypto` mirror).
#[allow(missing_docs)]
pub mod crypto;
/// Cast spine decision tree — steerability branches for agent episodes.
pub mod decision_tree;
/// Design sheaf facets — spine as time-axis (section / gluing / restriction / cohomology).
pub mod design_sheaf;
/// Frame / spine contract — cast funicular as degenerate 2-vertebra trajectory.
pub mod frame_spine;
pub mod gate;
/// AC82 — UARCS-gossip mesh_wired false census (`mesh_wired` false unless WEB-034 measured).
pub mod gossip_mesh_census;
mod trust_wire_public;
/// AC82 gossip mesh census — root re-exports for quality_gates wire scan C.
pub use gossip_mesh_census::{
    H81AbsorbedPosture, HaskellUcrsGossipMeshAdoptPosture, HaskellUcrsGossipMeshPropertySlot,
    UarcsGossipMeshProbe, UarcsGossipMeshWireHop, UcrsGossipMeshC82Probe,
    UcrsUrgeGossipComposeProbe, UrgeHistoryMeshConsumer,
};
pub mod landauer;
/// LIB-ADOPT-A-LANDAUER — UCRS pattern SSOT adoption witness (P1542 B4).
pub mod landauer_adopt;
#[cfg(feature = "a7-4")]
/// A7-4 global multi-information Landauer cost (feature-gated).
pub mod landauer_global;
/// Immutable observation stamps for durable agent logs (`UcrsObservedAt`, `TemporalWitness`).
pub mod observation;
/// P2P gossip types + gate-guarded sync hook (no libp2p in default builds).
pub mod p2p;
pub mod rapl;
/// Consumer contract facade — observation, accept, wire, crypto, cast spine (Wave 0).
pub mod shared_types;
pub mod telemetry;
/// AC21 — UARCS-004 policy-present deepen (`present_wired` false unless measured).
pub mod uarcs_004_policy_present;
/// AC81 — UARCS-A7-4 policy wire deepen (`policy_wired` false unless measured).
pub mod uarcs_a7_4_policy_wire;
/// Spine locals — clock, credit, agent loop, landauer_global (Wave 1 · U1_LIB barrel split).
pub mod ucrs_keep;
/// Gossip wire format + signature glue (no libp2p — safe for default library-only builds).
pub mod wire;
/// Root re-exports of module types (clock, credit, crypto errors, energy, gossip, probes).
pub use clock::LocalClock;
pub use credit::{CreditLedger, PeerCredit, SyncDecision};
pub use crypto::hash::sha3_256::HashError;
pub use crypto::kem::ml_kem_768::KemError;
pub use crypto::sig::ml_dsa_65::SigError as MlDsa65SigError;
pub use crypto::sig::slh_dsa_128s::SigError as SlhDsa128sSigError;
pub use crypto::CryptoError;
pub use landauer_adopt::{
    LandauerAdoptSymbolsProbe, LandauerMiEntropyBridgeProbe, LandauerPairwiseCloseProbe,
    LandauerUcrsWireHop,
};
pub use p2p::{GatedSyncOutcome, PeerGossip};
pub use rapl::{EnergyReading, RaplError, SyncEnergyRecord};
pub use uarcs_004_policy_present::{Uarcs004PolicyPresentProbe, Uarcs004PolicyPresentWireHop};
pub use uarcs_a7_4_policy_wire::{UarcsA74PolicyWireHop, UarcsA74PolicyWireProbe};
/// Root re-export — quality_gates wire scan C (overlay compose).
pub use wire::{ClockTick, ClockTickMldsa, MergeOutcome};

// --- Wave 2 compat shims (deprecated root re-exports) ---

#[deprecated(note = "use umst_ucrs::shared_types::accept")]
pub use accept::{
    DurableAccept, DurableAcceptWire, TrustAttestedWarrant, TrustCipherSuite, TrustStampReject,
    DURABLE_ACCEPT_SCHEMA_VERSION,
};
#[deprecated(note = "use umst_ucrs::shared_types::decision_tree")]
pub use decision_tree::{SteerDecision, SteerDecisionTrace, SteerKnobs, SteerPolicy};
#[deprecated(note = "use umst_ucrs::shared_types::design_sheaf")]
pub use design_sheaf::{
    route_steerability, spine_admissible_under_gluing, DecisionPolicy, DesignSheafOverSpine,
    MaterialEvolutionFrontier, SheafCohomologySeam, SheafGluingWitness, SheafRestriction,
    SheafSection, SteerabilityBranch, SteerabilityDecision,
};
#[deprecated(note = "use umst_ucrs::shared_types::frame_spine")]
pub use frame_spine::{
    Frame, MaterialState, OriginEvent, Spine, SpineTime, UnitVec3, Vertebra, VertebraGateVerdict,
};
#[deprecated(note = "use umst_ucrs::shared_types::observation")]
pub use observation::{TemporalWitness, UcrsObservedAt};

/// Root compat — prefer [`ucrs_keep::AgentConfig`].
pub use ucrs_keep::{agent_tick, witness_for_agent, AgentConfig};

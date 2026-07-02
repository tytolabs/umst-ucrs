// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cast spine **decision tree** — steerability branches for paper/demo agents.
//!
//! Maps agent edits (thickness, load morph, objective lane, Block MinR/MaxR) to spine
//! transitions referencing TNA metrics at `v_tna`. Not site certification — bracket envelope only.

use serde::{Deserialize, Serialize};

/// Agent-editable steer knobs (mirrors `SteerableVaultParams` in umst-steerable-vault).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SteerKnobs {
    pub thickness_m: f64,
    pub live_x_offset_frac: f64,
    pub wind_fx_n: f64,
    pub objective_lane: SteerObjectiveLane,
}

/// Thrust-network objective lane (string-stable for JSON witnesses).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteerObjectiveLane {
    ThrustNetworkBlock,
    ThrustNetworkStrip,
    ComplianceMinArchived,
}

impl SteerObjectiveLane {
    #[must_use]
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::ThrustNetworkBlock => "ThrustNetworkBlock",
            Self::ThrustNetworkStrip => "ThrustNetworkStrip",
            Self::ComplianceMinArchived => "ComplianceMinArchived",
        }
    }
}

/// TNA witness at `v_tna` (Block LP metrics).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TnaStrikeWitness {
    pub block_r: f64,
    pub abutment_thrust_n: f64,
    pub eq_residual: f64,
    pub phase_gate_admissible: bool,
}

/// Decision-tree node outcome after evaluating steer input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SteerDecision {
    /// Advance to next vertebra on the cast spine.
    AdvanceVertebra { label: String, admissible: bool },
    /// Re-solve thrust network (thickness or load morph changed equilibrium).
    ReSolveThrust {
        reason: String,
        witness: TnaStrikeWitness,
    },
    /// Phase gate rejected — agent must back off steer edit.
    GateReject { verdict: String, margin: f64 },
    /// Sweep thickness bracket (MinR funicular envelope exploration).
    SweepThicknessBracket {
        thickness_min_m: f64,
        thickness_max_m: f64,
    },
    /// Morph live-load centroid along span.
    MorphLoadOffset { from_frac: f64, to_frac: f64 },
}

/// Evaluate the cast-spine decision tree for one agent edit step.
///
/// `prior` — previous TNA witness (if any); `delta` — measurable change after steer.
#[must_use]
pub fn evaluate_steer_branch(
    knobs: &SteerKnobs,
    witness: &TnaStrikeWitness,
    prior: Option<&TnaStrikeWitness>,
) -> SteerDecision {
    if !witness.phase_gate_admissible || witness.eq_residual >= 1e-5 {
        return SteerDecision::GateReject {
            verdict: format!(
                "v_tna block_lp eq_res={:.3e} (lane={})",
                witness.eq_residual,
                knobs.objective_lane.as_label()
            ),
            margin: witness.eq_residual,
        };
    }

    if let Some(p) = prior {
        let h_delta = (witness.abutment_thrust_n - p.abutment_thrust_n).abs();
        let r_delta = (witness.block_r - p.block_r).abs();
        if h_delta > 50.0 || r_delta > 1e-4 {
            return SteerDecision::ReSolveThrust {
                reason: format!(
                    "H_delta={h_delta:.1}N r_delta={r_delta:.6} (offset={:.2})",
                    knobs.live_x_offset_frac
                ),
                witness: *witness,
            };
        }
    }

    if knobs.live_x_offset_frac.abs() > 0.05 {
        return SteerDecision::MorphLoadOffset {
            from_frac: 0.0,
            to_frac: knobs.live_x_offset_frac,
        };
    }

    if knobs.thickness_m < 0.10 || knobs.thickness_m > 0.25 {
        return SteerDecision::SweepThicknessBracket {
            thickness_min_m: 0.06,
            thickness_max_m: 0.32,
        };
    }

    SteerDecision::AdvanceVertebra {
        label: "v_tna".into(),
        admissible: witness.phase_gate_admissible,
    }
}

/// JSON-serializable decision-tree trace for agent episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerDecisionTrace {
    pub knobs: SteerKnobs,
    pub witness: TnaStrikeWitness,
    pub decision: SteerDecision,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_witness(h: f64, r: f64) -> TnaStrikeWitness {
        TnaStrikeWitness {
            block_r: r,
            abutment_thrust_n: h,
            eq_residual: 1e-7,
            phase_gate_admissible: true,
        }
    }

    #[test]
    fn load_offset_triggers_resolve_thrust() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.6,
            wind_fx_n: 0.0,
            objective_lane: SteerObjectiveLane::ThrustNetworkBlock,
        };
        let prior = demo_witness(12_000.0, 0.42);
        let witness = demo_witness(12_800.0, 0.418);
        let d = evaluate_steer_branch(&knobs, &witness, Some(&prior));
        assert!(
            matches!(d, SteerDecision::ReSolveThrust { .. }),
            "expected ReSolveThrust, got {d:?}"
        );
    }

    #[test]
    fn symmetric_baseline_advances_v_tna() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.0,
            wind_fx_n: 0.0,
            objective_lane: SteerObjectiveLane::ThrustNetworkBlock,
        };
        let witness = demo_witness(12_000.0, 0.42);
        let d = evaluate_steer_branch(&knobs, &witness, None);
        assert!(matches!(d, SteerDecision::AdvanceVertebra { .. }));
    }

    #[test]
    fn inadmissible_eq_residual_rejects() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.0,
            wind_fx_n: 0.0,
            objective_lane: SteerObjectiveLane::ThrustNetworkBlock,
        };
        let witness = TnaStrikeWitness {
            block_r: 0.4,
            abutment_thrust_n: 10_000.0,
            eq_residual: 1e-2,
            phase_gate_admissible: false,
        };
        let d = evaluate_steer_branch(&knobs, &witness, None);
        assert!(matches!(d, SteerDecision::GateReject { .. }));
    }
}

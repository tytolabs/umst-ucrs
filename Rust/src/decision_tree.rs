// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Cast spine **decision tree** — generic steerability branches for agent episodes.
//!
//! Consumer crates supply concrete witness and lane types; this module provides only the
//! domain-neutral routing infrastructure.

use serde::{Deserialize, Serialize};

/// Agent-editable steer knobs — generic over objective-lane type `L`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SteerKnobs<L> {
    pub thickness_m: f64,
    pub live_x_offset_frac: f64,
    pub wind_fx_n: f64,
    pub objective_lane: L,
}

/// Decision-tree node outcome after evaluating steer input — generic over witness `W`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SteerDecision<W> {
    /// Advance to next vertebra on the cast spine.
    AdvanceVertebra { label: String, admissible: bool },
    /// Re-solve equilibrium (geometry or load changed).
    ReSolve {
        reason: String,
        witness: W,
    },
    /// Phase gate rejected — agent must back off steer edit.
    GateReject { verdict: String, margin: f64 },
    /// Sweep thickness bracket (envelope exploration).
    SweepThicknessBracket {
        thickness_min_m: f64,
        thickness_max_m: f64,
    },
    /// Morph live-load centroid along span.
    MorphLoadOffset { from_frac: f64, to_frac: f64 },
}

/// Consumer-implemented policy that evaluates one steer step.
///
/// `L` = objective-lane type, `W` = witness type.
pub trait SteerPolicy<L, W> {
    /// Evaluate the decision tree for one agent edit step.
    fn evaluate(
        &self,
        knobs: &SteerKnobs<L>,
        witness: &W,
        prior: Option<&W>,
    ) -> SteerDecision<W>;
}

/// JSON-serializable decision-tree trace for agent episodes — generic over `L` and `W`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerDecisionTrace<L, W> {
    pub knobs: SteerKnobs<L>,
    pub witness: W,
    pub decision: SteerDecision<W>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct DummyLane;

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    struct DummyWitness {
        value: f64,
        admissible: bool,
    }

    struct DummySteerPolicy;

    impl SteerPolicy<DummyLane, DummyWitness> for DummySteerPolicy {
        fn evaluate(
            &self,
            knobs: &SteerKnobs<DummyLane>,
            witness: &DummyWitness,
            prior: Option<&DummyWitness>,
        ) -> SteerDecision<DummyWitness> {
            if !witness.admissible {
                return SteerDecision::GateReject {
                    verdict: "inadmissible".into(),
                    margin: witness.value,
                };
            }
            if let Some(p) = prior {
                if (witness.value - p.value).abs() > 0.1 {
                    return SteerDecision::ReSolve {
                        reason: "delta exceeded".into(),
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
            SteerDecision::AdvanceVertebra {
                label: "v_next".into(),
                admissible: witness.admissible,
            }
        }
    }

    fn demo_witness(v: f64) -> DummyWitness {
        DummyWitness {
            value: v,
            admissible: true,
        }
    }

    #[test]
    fn load_offset_triggers_re_solve() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.6,
            wind_fx_n: 0.0,
            objective_lane: DummyLane,
        };
        let prior = demo_witness(0.42);
        let witness = demo_witness(0.80);
        let d = DummySteerPolicy.evaluate(&knobs, &witness, Some(&prior));
        assert!(
            matches!(d, SteerDecision::ReSolve { .. }),
            "expected ReSolve, got {d:?}"
        );
    }

    #[test]
    fn symmetric_baseline_advances() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.0,
            wind_fx_n: 0.0,
            objective_lane: DummyLane,
        };
        let witness = demo_witness(0.42);
        let d = DummySteerPolicy.evaluate(&knobs, &witness, None);
        assert!(matches!(d, SteerDecision::AdvanceVertebra { .. }));
    }

    #[test]
    fn inadmissible_rejects() {
        let knobs = SteerKnobs {
            thickness_m: 0.15,
            live_x_offset_frac: 0.0,
            wind_fx_n: 0.0,
            objective_lane: DummyLane,
        };
        let witness = DummyWitness {
            value: 0.4,
            admissible: false,
        };
        let d = DummySteerPolicy.evaluate(&knobs, &witness, None);
        assert!(matches!(d, SteerDecision::GateReject { .. }));
    }
}

// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! Design sheaf facets — the spine is the **time-axis** of the admissibility sheaf.
//!
//! | Facet | Role in cast lifecycle |
//! |-------|------------------------|
//! | **Section** | Per-vertebra gate = local section of admissibility |
//! | **Gluing** | DEC conservation (`d∘d=0`) glues local fluxes to global conservation |
//! | **Restriction** | `hex_coarsen_cell_field` coarse-grains across scale |
//! | **Cohomology** | Obstruction-to-gluing / memory / symmetry-break = H¹ (seam only) |

use serde::{Deserialize, Serialize};

use crate::frame_spine::{Spine, Vertebra};

/// SECTION — per-vertebra gate is a section of the admissibility sheaf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheafSection {
    pub vertebra_label: String,
    pub admissible: bool,
    pub margin: f64,
    pub verdict_label: String,
}

impl SheafSection {
    #[must_use]
    pub fn from_vertebra(v: &Vertebra) -> Self {
        Self {
            vertebra_label: v.t.label.clone(),
            admissible: v.gate.admissible,
            margin: v.gate.h_notension,
            verdict_label: v.gate.verdict_label.clone(),
        }
    }
}

/// GLUING — DEC conservation (`d∘d=0`) is the gluing axiom.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheafGluingWitness {
    pub conservation_axiom: String,
    pub sections_glue: bool,
}

impl SheafGluingWitness {
    #[must_use]
    pub fn dec_conservation() -> Self {
        Self {
            conservation_axiom: "d∘d=0".into(),
            sections_glue: true,
        }
    }
}

/// RESTRICTION — coarse-graining across scale (`hex_coarsen_cell_field` in manifold).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheafRestriction {
    pub map_name: String,
}

impl SheafRestriction {
    #[must_use]
    pub fn hex_coarsen_cell_field() -> Self {
        Self {
            map_name: "hex_coarsen_cell_field".into(),
        }
    }
}

/// COHOMOLOGY — obstruction seam (H¹ of this sheaf); memory-sheaf NOT built here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SheafCohomologySeam {
    pub degree: u8,
    pub role: String,
    pub built: bool,
}

impl SheafCohomologySeam {
    #[must_use]
    pub fn memory_h1_seam() -> Self {
        Self {
            degree: 1,
            role: "obstruction-to-gluing / symmetry-break memory".into(),
            built: false,
        }
    }
}

/// Continuous material evolution between vertebrae — FRONTIER, not built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialEvolutionFrontier {
    pub models: Vec<String>,
    pub built: bool,
}

impl MaterialEvolutionFrontier {
    #[must_use]
    pub fn cartridge_frontier() -> Self {
        Self {
            models: vec![
                "hydration α(t)".into(),
                "creep".into(),
                "drying shrinkage".into(),
                "thermal".into(),
                "as-cast→service camber".into(),
            ],
            built: false,
        }
    }
}

/// Spine as time-axis: collect sections + gluing witness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSheafOverSpine {
    pub time_axis: String,
    pub sections: Vec<SheafSection>,
    pub gluing: SheafGluingWitness,
    pub restriction: SheafRestriction,
    pub cohomology_seam: SheafCohomologySeam,
    pub material_frontier: MaterialEvolutionFrontier,
    /// Optional steerability routing from TNA metric shape (cast lifecycle).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steerability: Option<SteerabilityDecision>,
}

/// TNA metric shape fed from steerable-vault lifecycle / bracket solve.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TnaMetricShape {
    pub thrust_bracket_width: f64,
    pub phase_gate: String,
    pub objective_lane: String,
}

/// Steerability branch selected from TNA metrics + spine phase gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteerabilityBranch {
    HoldSymmetric,
    ExploreLoadOffset,
    WidenBracketSweep,
    RejectInadmissible,
}

/// Routed steerability decision for agent / UCRS spine export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SteerabilityDecision {
    pub branch: SteerabilityBranch,
    pub metric: TnaMetricShape,
}

/// Route steerability from TNA metric shape (minimal decision tree).
#[must_use]
pub fn route_steerability(metric: &TnaMetricShape) -> SteerabilityDecision {
    let branch = if metric.phase_gate.contains("fail")
        || metric.phase_gate.contains("inadmissible")
    {
        SteerabilityBranch::RejectInadmissible
    } else if metric.thrust_bracket_width < 0.05 {
        SteerabilityBranch::HoldSymmetric
    } else if metric.objective_lane == "block_lp" && metric.thrust_bracket_width > 0.1 {
        SteerabilityBranch::ExploreLoadOffset
    } else {
        SteerabilityBranch::WidenBracketSweep
    };
    SteerabilityDecision {
        branch,
        metric: metric.clone(),
    }
}

impl DesignSheafOverSpine {
    pub const TIME_AXIS_LABEL: &'static str = "spine_is_time_axis_of_design_sheaf";

    #[must_use]
    pub fn from_spine(spine: &Spine) -> Self {
        Self::from_spine_with_metric(spine, None)
    }

    #[must_use]
    pub fn from_spine_with_metric(
        spine: &Spine,
        metric: Option<TnaMetricShape>,
    ) -> Self {
        let sections: Vec<_> = spine
            .vertebrae
            .iter()
            .map(SheafSection::from_vertebra)
            .collect();
        let gluing = SheafGluingWitness {
            conservation_axiom: "d∘d=0".into(),
            sections_glue: spine_admissible_under_gluing(spine),
        };
        let steerability = metric.map(|m| route_steerability(&m));
        Self {
            time_axis: Self::TIME_AXIS_LABEL.into(),
            sections,
            gluing,
            restriction: SheafRestriction::hex_coarsen_cell_field(),
            cohomology_seam: SheafCohomologySeam::memory_h1_seam(),
            material_frontier: MaterialEvolutionFrontier::cartridge_frontier(),
            steerability,
        }
    }
}

/// A spine is admissible iff each vertebra's section margins glue under conservation.
#[must_use]
pub fn spine_admissible_under_gluing(spine: &Spine) -> bool {
    spine.vertebrae.iter().all(|v| {
        if v.t.label == "formwork_strike" {
            return true;
        }
        v.gate.admissible || v.gate.h_notension >= 0.0
    })
}

#[cfg(test)]
mod steerability_tests {
    use super::*;

    #[test]
    fn route_explores_load_offset_for_wide_block_bracket() {
        let metric = TnaMetricShape {
            thrust_bracket_width: 0.15,
            phase_gate: "compression_admissible_envelope".into(),
            objective_lane: "block_lp".into(),
        };
        let decision = route_steerability(&metric);
        assert_eq!(decision.branch, SteerabilityBranch::ExploreLoadOffset);
    }

    #[test]
    fn route_rejects_inadmissible_phase_gate() {
        let metric = TnaMetricShape {
            thrust_bracket_width: 0.2,
            phase_gate: "compression_inadmissible_envelope".into(),
            objective_lane: "block_lp".into(),
        };
        let decision = route_steerability(&metric);
        assert_eq!(decision.branch, SteerabilityBranch::RejectInadmissible);
    }
}

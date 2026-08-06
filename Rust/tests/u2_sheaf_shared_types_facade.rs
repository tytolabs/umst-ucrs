// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! U2 sheaf facade parity — `shared_types::design_sheaf` consumer contract (CELL_UCRS_READY_U2_SHEAF).

use umst_ucrs::shared_types::design_sheaf::{
    DesignSheafOverSpine, DecisionPolicy, SheafGluingWitness, SheafSection, SteerabilityBranch,
    route_steerability, spine_admissible_under_gluing,
};
use umst_ucrs::shared_types::frame_spine::{
    Frame, MaterialState, Spine, SpineTime, Vertebra, VertebraGateVerdict,
};
use umst_ucrs::shared_types::observation::UcrsObservedAt;

fn minimal_spine() -> Spine {
    let frame = Frame::default_negative_y();
    let stamp = UcrsObservedAt::wall_only();
    let vertebra = Vertebra {
        t: SpineTime::origin(),
        rho_ref: vec![0.0, 1.0],
        material: MaterialState::cured_service(),
        load: [0.0, -9.81, 0.0],
        gate: VertebraGateVerdict {
            admissible: true,
            h_notension: 0.0,
            verdict_label: "ok".into(),
        },
        stamp,
    };
    Spine {
        frame,
        vertebrae: vec![vertebra.clone(), vertebra],
    }
}

#[test]
fn shared_types_design_sheaf_morphisms_constructors() {
    let gluing = SheafGluingWitness::dec_conservation();
    assert_eq!(gluing.conservation_axiom, "d∘d=0");
    assert!(gluing.sections_glue);

    let spine = minimal_spine();
    let section = SheafSection::from_vertebra(&spine.vertebrae[0]);
    assert!(section.admissible);
    assert_eq!(section.verdict_label, "ok");

    let sheaf = DesignSheafOverSpine::<()>::from_spine(&spine);
    assert_eq!(
        sheaf.time_axis,
        DesignSheafOverSpine::<()>::TIME_AXIS_LABEL
    );
    assert_eq!(sheaf.sections.len(), 2);
    assert!(sheaf.gluing.sections_glue);
    assert!(spine_admissible_under_gluing(&spine));
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct DummyMetric {
    val: f64,
}

struct DummyPolicy;

impl DecisionPolicy<DummyMetric> for DummyPolicy {
    fn route_decision(&self, metric: &DummyMetric) -> SteerabilityBranch {
        if metric.val > 0.5 {
            SteerabilityBranch::ExploreOffset
        } else {
            SteerabilityBranch::Hold
        }
    }
}

#[test]
fn shared_types_design_sheaf_steerability_roundtrip() {
    let spine = minimal_spine();
    let metric = DummyMetric { val: 0.8 };
    let decision = route_steerability(&metric, &DummyPolicy);
    assert_eq!(decision.branch, SteerabilityBranch::ExploreOffset);

    let sheaf =
        DesignSheafOverSpine::from_spine_with_metric(&spine, Some(metric.clone()), &DummyPolicy);
    assert!(sheaf.steerability.is_some());
    assert_eq!(
        sheaf.steerability.as_ref().unwrap().branch,
        SteerabilityBranch::ExploreOffset
    );

    let json = serde_json::to_string(&sheaf).expect("DesignSheafOverSpine serializes");
    let back: DesignSheafOverSpine<DummyMetric> =
        serde_json::from_str(&json).expect("DesignSheafOverSpine deserializes");
    assert_eq!(back.sections.len(), sheaf.sections.len());
    assert_eq!(back.time_axis, DesignSheafOverSpine::<DummyMetric>::TIME_AXIS_LABEL);
}

// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! U2 frame facade parity — `shared_types::frame_spine` consumer contract (CELL_UCRS_READY_U2_FRAME).

use umst_ucrs::shared_types::frame_spine::{
    Frame, MaterialState, Spine, SpineTime, UnitVec3, Vertebra, VertebraGateVerdict,
};
use umst_ucrs::shared_types::observation::UcrsObservedAt;

#[test]
fn shared_types_frame_spine_morphisms_constructors() {
    let frame = Frame::default_negative_y();
    assert_eq!(frame.gravity_dir, UnitVec3::negative_y());
    assert_eq!(frame.time_origin.label, "formwork_strike");

    let mat = MaterialState::cured_service();
    assert_eq!(mat.hydration_alpha, 1.0);
    assert_eq!(mat.strength_mpa, 37.0);

    let origin = SpineTime::origin();
    assert_eq!(origin.offset_s, 0.0);
    let service = SpineTime::service();
    assert_eq!(service.label, "service");
}

#[test]
fn shared_types_frame_spine_cast_degenerate_roundtrip() {
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
    let spine = Spine {
        frame,
        vertebrae: vec![vertebra.clone(), vertebra],
    };
    assert!(spine.final_vertebra().is_some());
    assert_eq!(spine.cast_rho(), Some([0.0, 1.0].as_slice()));

    let json = serde_json::to_string(&spine).expect("Spine serializes");
    let back: Spine = serde_json::from_str(&json).expect("Spine deserializes");
    assert_eq!(spine.vertebrae.len(), back.vertebrae.len());
    assert_eq!(back.frame.gravity_dir, UnitVec3::negative_y());
}

// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Santhosh Shyamsundar, Santosh Prabhu Shenbagamoorthy — Studio TYTO

//! UCRS frame / spine contract — coupling root for geometry × material × load × time.
//!
//! Additive foundation: cast funicular is the 2-vertebra degenerate case; print / steer /
//! generative trajectories extend the same `Spine` without breaking existing observation stamps.

use serde::{Deserialize, Serialize};

use crate::observation::UcrsObservedAt;

/// Unit gravity direction ĝ (normalized).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UnitVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl UnitVec3 {
    /// Vault legacy self-weight axis (−Y).
    #[must_use]
    pub const fn negative_y() -> Self {
        Self {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        }
    }

    #[must_use]
    pub fn as_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    /// Renormalize; zero vector falls back to −Y.
    #[must_use]
    pub fn normalized(self) -> Self {
        let n = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if n > 1e-12 {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
            }
        } else {
            Self::negative_y()
        }
    }
}

/// Time origin t = 0 event (recorded in UCRS stamp context).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OriginEvent {
    pub label: String,
}

impl OriginEvent {
    #[must_use]
    pub fn formwork_strike() -> Self {
        Self {
            label: "formwork_strike".into(),
        }
    }
}

/// Frame — fixes ĝ and t = 0; steerable inputs for downstream trajectories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub gravity_dir: UnitVec3,
    pub time_origin: OriginEvent,
}

impl Frame {
    #[must_use]
    pub fn cast_vault_default() -> Self {
        Self {
            gravity_dir: UnitVec3::negative_y(),
            time_origin: OriginEvent::formwork_strike(),
        }
    }
}

/// Elapsed time from frame origin (seconds).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpineTime {
    pub label: String,
    pub offset_s: f64,
}

impl SpineTime {
    #[must_use]
    pub fn origin() -> Self {
        Self {
            label: "formwork_strike".into(),
            offset_s: 0.0,
        }
    }

    #[must_use]
    pub fn service() -> Self {
        Self {
            label: "service".into(),
            offset_s: 0.0,
        }
    }

    /// Early-age strip / strike check (t_days from formwork strike).
    #[must_use]
    pub fn strike_early_age(t_days: f64) -> Self {
        Self {
            label: "strike_early_age".into(),
            offset_s: t_days * 86_400.0,
        }
    }

    /// Fresh plastic phase (0–6 h from formwork strike).
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            label: "fresh".into(),
            offset_s: 3.0 * 3600.0,
        }
    }

    /// Setting phase (set → 24 h).
    #[must_use]
    pub fn setting() -> Self {
        Self {
            label: "setting".into(),
            offset_s: 12.0 * 3600.0,
        }
    }

    /// Strengthening phase (creep / shrinkage toward 28 d).
    #[must_use]
    pub fn strengthening() -> Self {
        Self {
            label: "strengthening".into(),
            offset_s: 14.0 * 86_400.0,
        }
    }

    /// Service load envelope (self-weight + live superimposed).
    #[must_use]
    pub fn service_envelope() -> Self {
        Self {
            label: "service_envelope".into(),
            offset_s: 0.0,
        }
    }
}

/// Hydration / strength snapshot at a vertebra.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialState {
    pub hydration_alpha: f64,
    pub strength_mpa: f64,
    /// Elastic modulus (MPa) when maturity-scaled; `None` = service default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e_mpa: Option<f64>,
    /// Age at vertebra (days) for EC2 maturity cite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_days: Option<f64>,
    /// EC2 strength-development exponent s (Table 3.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ec2_s: Option<f64>,
}

impl MaterialState {
    #[must_use]
    pub fn cured_service() -> Self {
        Self {
            hydration_alpha: 1.0,
            strength_mpa: 37.0,
            e_mpa: None,
            t_days: None,
            ec2_s: None,
        }
    }

    /// EC2 maturity snapshot at early-age strike.
    #[must_use]
    pub fn early_age_strike(t_days: f64, f_c_mpa: f64, e_mpa: f64, s: f64) -> Self {
        Self {
            hydration_alpha: (t_days / 28.0).clamp(0.0, 1.0),
            strength_mpa: f_c_mpa,
            e_mpa: Some(e_mpa),
            t_days: Some(t_days),
            ec2_s: Some(s),
        }
    }
}

/// Per-vertebra gate outcome (compression-only / admissibility face).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VertebraGateVerdict {
    pub admissible: bool,
    pub h_notension: f64,
    pub verdict_label: String,
}

/// One vertebra on an extensible spine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertebra {
    pub t: SpineTime,
    pub rho_ref: Vec<f64>,
    pub material: MaterialState,
    /// External traction [fx, fy, fz]; self-weight uses `Frame::gravity_dir`.
    pub load: [f64; 3],
    pub gate: VertebraGateVerdict,
    pub stamp: UcrsObservedAt,
}

/// Ordered, extensible spine (cast = 2 vertebrae; print / steer add more).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spine {
    pub frame: Frame,
    pub vertebrae: Vec<Vertebra>,
}

impl Spine {
    #[must_use]
    pub fn final_vertebra(&self) -> Option<&Vertebra> {
        self.vertebrae.last()
    }

    #[must_use]
    pub fn cast_rho(&self) -> Option<&[f64]> {
        self.final_vertebra().map(|v| v.rho_ref.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_frame_is_negative_y() {
        let f = Frame::cast_vault_default();
        assert_eq!(f.gravity_dir, UnitVec3::negative_y());
        assert_eq!(f.time_origin.label, "formwork_strike");
    }
}

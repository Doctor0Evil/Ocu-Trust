#![forbid(unsafe_code)]

use bioscaleupgradestore::{HostBudget, ThermodynamicEnvelope};
use nanoswarmhostmath::{HostCalibration, MlDutyEnvelope};
use oculustrust_corridor::{OculusCorridorState, analyze_oculus_corridor};

/// ALN-facing struct: oculus metrics attached to any visual upgrade.
/// No downgrade/reversal fields; this is strictly forward-evolution metadata.
#[derive(Clone, Debug)]
pub struct OcuTrustOculusEnvelope {
    /// Blink index in [0, 1] for this corridor under the planned load.
    pub oculus_blinkindex: f64,
    /// Average duty across visual + oculomotor regions.
    pub avg_duty: f64,
    /// Oculomotor-only duty component.
    pub oculomotor_duty: f64,
    /// Corridor-level safety (true => within envelopes).
    pub corridor_safe: bool,
    /// Duty history safety.
    pub duty_safe: bool,
}

/// Pure forward evaluation: compute oculus envelope for a proposed state.
/// This function never encodes or triggers any downgrade or rollback.
pub fn evaluate_oculus_envelope(
    state: OculusCorridorState,
    calib: HostCalibration,
    ml_env: MlDutyEnvelope,
    alpha_corr: f64,
    k0_corr: f64,
    s_corr_max: f64,
) -> OcuTrustOculusEnvelope {
    let analysis = analyze_oculus_corridor(
        state,
        calib,
        ml_env,
        alpha_corr,
        k0_corr,
        s_corr_max,
    );

    OcuTrustOculusEnvelope {
        oculus_blinkindex: analysis.oculus_blinkindex,
        avg_duty: analysis.avg_duty,
        oculomotor_duty: analysis.oculomotor_duty,
        corridor_safe: analysis.corridor_safe,
        duty_safe: analysis.duty_safe,
    }
}

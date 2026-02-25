#![forbid(unsafe_code)]

use std::time::Duration;
use nanoswarm_host_math::{
    HostRegionDerived, MlDutyEnvelope, aggregate_corridor, compute_theta_over_window,
};
use bioscale_upgrade_store::{HostBudget, ThermodynamicEnvelope};
use cyberswarm_neurostack::bci_host_snapshot::BciHostSnapshot;

/// Telemetry bundles: RET_ENERGY, SYM_STRESS, VIS_DUTY.
/// These are forward-only measurements, no rollback semantics.

#[derive(Clone, Debug)]
pub struct RetinalEnergyEpoch {
    pub e_in_joules: f64,
    pub e_out_joules: f64,
    pub qbio_ret: f64,
}

#[derive(Clone, Debug)]
pub struct SympatheticEpoch {
    /// HRV-based scalar, 0.0 = low stress, 1.0 = high stress.
    pub symp_scalar: f64,
}

#[derive(Clone, Debug)]
pub struct VisualDutyEpoch<R> {
    /// Full set of visual corridor regions (retina, LGN, V1, extrastriate).
    pub regions: Vec<R>,
    /// Duty samples (0.0–1.0) over T_blink for corridor.
    pub corridor_duty_samples: Vec<f64>,
    /// Blink interval window.
    pub duty_window: Duration,
}

/// Weights for blink index; must be non-negative and sum to 1.0.
#[derive(Clone, Debug)]
pub struct BlinkWeights {
    pub w_sbio: f64,
    pub w_theta: f64,
    pub w_symp: f64,
}

impl BlinkWeights {
    pub fn new(ws: f64, wt: f64, wp: f64) -> Self {
        let sum = (ws + wt + wp).max(f64::MIN_POSITIVE);
        Self {
            w_sbio: ws / sum,
            w_theta: wt / sum,
            w_symp: wp / sum,
        }
    }
}

fn clamp01(x: f64) -> f64 {
    if x <= 0.0 {
        0.0
    } else if x >= 1.0 {
        1.0
    } else {
        x
    }
}

/// Forward-only corridor metrics for a blink window.
#[derive(Clone, Debug)]
pub struct OculusBlinkMetrics {
    pub s_bio_corridor: f64,
    pub avg_duty: f64,
    pub symp_scalar: f64,
    pub oculus_blinkindex: f64,
}

/// Compute S_bio,C, C, symp, and B_oculus over a blink window.
/// Pure function; no downgrade/rollback behavior.
pub fn compute_oculus_blink_metrics(
    visual: VisualDutyEpoch<HostRegionDerived>,
    symp: SympatheticEpoch,
    alpha_c: f64,
    k0_c: f64,
    s_bio_c_safe: f64,
    c_safe: f64,
    weights: BlinkWeights,
) -> OculusBlinkMetrics {
    // 1. Aggregate corridor energy and K_bio.
    let (e_corr, k_corr) = aggregate_corridor(&visual.regions, alpha_c, k0_c);
    // Logistic corridor S_bio,C (same law as nanoswarm_host_math).
    let ratio = if k0_c > 0.0 { k_corr / k0_c } else { 0.0 };
    let s_corr = 1.0 - (-alpha_c * ratio * e_corr).exp();
    let s_corr = clamp01(s_corr);

    // 2. Corridor average duty C over T_blink.
    let c_corr = compute_theta_over_window(&visual.corridor_duty_samples);
    let c_corr = clamp01(c_corr);

    // 3. Sympathetic scalar.
    let symp_scalar = clamp01(symp.symp_scalar);

    // 4. Normalize against safe caps.
    let b_s = clamp01(s_corr / s_bio_c_safe.max(1e-6));
    let b_theta = clamp01(c_corr / c_safe.max(1e-6));
    let b_phi = symp_scalar;

    // 5. Weighted sum -> B_oculus in [0,1].
    let b_oculus = clamp01(
        weights.w_sbio * b_s +
        weights.w_theta * b_theta +
        weights.w_symp * b_phi,
    );

    OculusBlinkMetrics {
        s_bio_corridor: s_corr,
        avg_duty: c_corr,
        symp_scalar,
        oculus_blinkindex: b_oculus,
    }
}

/// Forward-only envelope attached to an evolution step.
/// No downgrade/rollback fields.
#[derive(Clone, Debug)]
pub struct OculusCorridorEnvelope {
    pub oculus_blinkindex: f64,
    pub oculus_avg_duty: f64,
    pub oculomotor_duty: f64,
    pub oculus_corridor_safe: bool,
    pub oculus_duty_safe: bool,
}

/// Compute the oculus envelope for a single blink-window,
/// using pre-calibrated safe caps and duty envelopes.
pub fn evaluate_oculus_envelope(
    metrics: OculusBlinkMetrics,
    oculomotor_duty: f64,
    s_bio_c_safe: f64,
    c_safe: f64,
    ml_duty_env: &MlDutyEnvelope,
) -> OculusCorridorEnvelope {
    let oculus_corridor_safe =
        metrics.s_bio_corridor <= s_bio_c_safe && metrics.avg_duty <= c_safe;

    let oculus_duty_safe =
        ml_duty_env.within_corridor_limits(metrics.avg_duty) &&
        ml_duty_env.within_oculomotor_limits(oculomotor_duty);

    OculusCorridorEnvelope {
        oculus_blinkindex: metrics.oculus_blinkindex,
        oculus_avg_duty: metrics.avg_duty,
        oculomotor_duty: clamp01(oculomotor_duty),
        oculus_corridor_safe,
        oculus_duty_safe,
    }
}

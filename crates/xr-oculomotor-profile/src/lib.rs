use crate::oculomotor_corridor::{OculomotorCorridorState, OculomotorStrainEnvelope};
use crate::visual_corridor_blink_analyzer::VisualBlinkMetrics;

/// Combined retinal–V1–oculomotor profile for XR safety gating.
#[derive(Clone, Debug)]
pub struct XROculomotorProfile {
    /// Visual corridor metrics (energy, BioKarma, Sbio,C, duty, symp, blink).
    pub visual: VisualBlinkMetrics,
    /// Oculomotor duty and strain envelope.
    pub oculomotor_env: OculomotorStrainEnvelope,
    /// Whether current XR oculomotor load is within envelope.
    pub oculomotor_safe: bool,
    /// Whether visual + oculomotor coupling is within safe duty cycle.
    pub coupled_safe: bool,
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

/// Evaluate XR oculomotor safety given corridor state and visual metrics.
pub fn evaluate_xr_oculomotor<R>(
    env: OculomotorStrainEnvelope,
    oculostate: &OculomotorCorridorState<R>,
    visual: &VisualBlinkMetrics,
) -> XROculomotorProfile {
    let duty = if oculostate.oculomotor_duty_samples.is_empty() {
        0.0
    } else {
        let sum: f64 = oculostate.oculomotor_duty_samples.iter().copied().sum();
        clamp01(sum / (oculostate.oculomotor_duty_samples.len() as f64))
    };

    // Sympathetic load is already encoded in BciHostSnapshot -> VisualBlinkMetrics.phi_symp.
    let symp = clamp01(visual.phi_symp);

    // Blink coupling: how far are we beyond a low‑load baseline (e.g. desktop control).
    let blink_delta = clamp01(visual.blink_index); // baseline assumed near 0

    let oculomotor_safe =
        duty <= env.max_oculomotor_duty && symp <= env.max_sympathetic_load;

    let coupled_safe =
        oculomotor_safe && blink_delta <= env.max_blink_delta;

    XROculomotorProfile {
        visual: visual.clone(),
        oculomotor_env: env,
        oculomotor_safe,
        coupled_safe,
    }
}

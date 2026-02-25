#![forbid(unsafe_code)]

use std::time::Duration;

use bioscale_upgrade_store::{
    EvolutionPointId,
    UpgradeDescriptorId,
    HostBudget,
    ThermodynamicEnvelope,
};
use cyberswarm_neurostack::hostenv::HostEnvContract;
use visual_corridor_blink::{
    VisualCorridorState,
    VisualCorridorBlinkAnalysis,
    analyze_visual_corridor,
};

/// Forward-only oculus metrics computed per evolution step.
/// These values are measurements / derived scalars, not controls.
#[derive(Clone, Debug)]
pub struct OculusBlinkEnvelope {
    /// Blink index B ∈ [0, 1] for the unified oculus corridor
    /// (retina, LGN, V1, extrastriate, oculomotor).
    pub oculus_blinkindex: f64,
    /// Average duty across the visual corridor.
    pub avg_duty: f64,
    /// Oculomotor-only duty (saccade/vergence/pursuit muscles).
    pub oculomotor_duty: f64,
    /// True if corridor-level bioimpact and energy are within envelopes.
    pub corridor_safe: bool,
    /// True if duty history for all regions is within ML duty envelopes.
    pub duty_safe: bool,
}

/// EvolutionPoint is extended with a forward-only oculus view.
/// This struct does not contain any rollback/downgrade fields.
#[derive(Clone, Debug)]
pub struct EvolutionPoint {
    pub id: EvolutionPointId,
    pub created_at: Duration,

    // Host envelopes at this point.
    pub host_budget: HostBudget,
    pub thermo_env: ThermodynamicEnvelope,

    // Existing bioscale metrics (not shown here) …

    // New oculus metrics.
    pub oculus_env: OculusBlinkEnvelope,
}

/// UpgradeDescriptor is extended to express *requirements* on oculus state,
/// but never carries instructions to reverse or downgrade prior evolution.
#[derive(Clone, Debug)]
pub struct UpgradeDescriptor {
    pub id: UpgradeDescriptorId,

    // Existing fields: energy/protein envelopes, neurorights tags, etc. …

    /// Optional upper bound on acceptable blink index for this upgrade.
    pub max_oculus_blinkindex: Option<f64>,
    /// Optional upper bound on avg visual duty for this upgrade.
    pub max_avg_duty: Option<f64>,
    /// Optional upper bound on oculomotor duty for this upgrade.
    pub max_oculomotor_duty: Option<f64>,

    /// If true, the upgrade requires the oculus corridor to be flagged safe.
    pub require_oculus_corridor_safe: bool,
    /// If true, the upgrade requires duty_safe to be true.
    pub require_oculus_duty_safe: bool,
}

/// HostEnvContract for the visual corridor.
/// This trait binds HostBudget + ThermodynamicEnvelope + raw corridor state
/// into a pure, forward-only oculus envelope for evolution routing.
pub trait OculusHostEnvContract: HostEnvContract {
    fn current_visual_corridor_state(&self) -> VisualCorridorState;

    fn host_calibration(&self) -> visual_corridor_blink::HostCalibration;
    fn ml_duty_envelope(&self) -> visual_corridor_blink::MlDutyEnvelope;

    /// Compute the oculus envelope from live host state, without side effects.
    fn compute_oculus_envelope(&self) -> OculusBlinkEnvelope {
        let state = self.current_visual_corridor_state();
        let calib = self.host_calibration();
        let ml_env = self.ml_duty_envelope();

        // Parameters for corridor BioKarma → Sbio,C mapping for visual corridor.
        let alpha_corr = self.visual_corridor_alpha();
        let k0_corr = self.visual_corridor_k0();
        let scorr_max = self.visual_corridor_scorr_max();

        let analysis: VisualCorridorBlinkAnalysis = analyze_visual_corridor(
            state,
            calib,
            ml_env,
            alpha_corr,
            k0_corr,
            scorr_max,
        );

        OculusBlinkEnvelope {
            oculus_blinkindex: analysis.blinkindex,
            avg_duty: analysis.avg_duty,
            oculomotor_duty: analysis.oculomotor_duty,
            corridor_safe: analysis.corridorsafe,
            duty_safe: analysis.dutysafe,
        }
    }

    /// Visual corridor calibration parameters (evidence-backed).
    fn visual_corridor_alpha(&self) -> f64;
    fn visual_corridor_k0(&self) -> f64;
    fn visual_corridor_scorr_max(&self) -> f64;
}

/// Forward-only routing predicate for Cyberswarm / bioscale router.
/// This function only *admits or denies* a proposed upgrade based on oculus
/// envelope; it does not encode any downgrade or rollback behavior.
pub fn oculus_allows_upgrade<E: OculusHostEnvContract>(
    env: &E,
    evo_point: &EvolutionPoint,
    desc: &UpgradeDescriptor,
) -> bool {
    let oc = &evo_point.oculus_env;

    if desc.require_oculus_corridor_safe && !oc.corridor_safe {
        return false;
    }
    if desc.require_oculus_duty_safe && !oc.duty_safe {
        return false;
    }

    if let Some(max_blink) = desc.max_oculus_blinkindex {
        if oc.oculus_blinkindex > max_blink {
            return false;
        }
    }
    if let Some(max_avg) = desc.max_avg_duty {
        if oc.avg_duty > max_avg {
            return false;
        }
    }
    if let Some(max_om) = desc.max_oculomotor_duty {
        if oc.oculomotor_duty > max_om {
            return false;
        }
    }

    true
}

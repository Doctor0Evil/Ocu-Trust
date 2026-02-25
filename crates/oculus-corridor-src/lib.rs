#![forbid(unsafe_code)]

//! Oculus corridor safety and blink-index analysis.
//!
//! This crate specializes your existing corridor math for the
//! visual + oculomotor system (retina, LGN, V1, extrastriate,
//! and extraocular muscles), producing an oculus_blinkindex in [0, 1]
//! plus hard corridor safety flags suitable for Cyberswarm routing.

use std::f64;

use bioscaleupgradestore::{HostBudget, ThermodynamicEnvelope};
use nanoswarmhostmath::{
    HostCalibration,
    HostRegionState,
    HostRegionDerived,
    MlDutyEnvelope,
    computeenergyandmass,
    computebiokarmaandscore,
    computewbio,
    updatedutycycle,
    aggregatecorridor,
    corridorscoreandgate,
    computethetaoverwindow,
    checkthetasafe,
};

/// Kinds of regions that participate in the oculus corridor.
#[derive(Clone, Debug)]
pub enum OculusRegionKind {
    RetinaFovea,
    RetinaPeriphery,
    Lgn,
    V1,
    Extrastriate,
    OcularMuscleHorizontal,
    OcularMuscleVertical,
}

/// Per-region state plus semantic tag.
#[derive(Clone, Debug)]
pub struct OculusRegionState {
    pub kind: OculusRegionKind,
    pub region: HostRegionState,
}

/// Corridor-level input: regions + history + global stress.
#[derive(Clone, Debug)]
pub struct OculusCorridorState {
    /// Visual and oculomotor regions.
    pub regions: Vec<OculusRegionState>,
    /// Duty history per region (for residence time checks).
    pub duty_history: Vec<Vec<f64>>,
    /// Global sympathetic stress (HRV/pain/etc.), 0–1.
    pub symp_global: f64,
    /// Host budget snapshot.
    pub host_budget: HostBudget,
    /// Thermodynamic envelope relevant to ocular tissues.
    pub thermo_env: ThermodynamicEnvelope,
}

/// Analysis output for the oculus corridor.
#[derive(Clone, Debug)]
pub struct OculusCorridorAnalysis {
    /// Per-region derived quantities (energy, BioKarma, duty).
    pub derived: Vec<HostRegionDerived>,
    /// Total corridor energy in joules.
    pub e_corr_joules: f64,
    /// Total corridor BioKarma.
    pub k_corr: f64,
    /// Corridor bioimpact score S_bio,C in [0, 1].
    pub s_corr: f64,
    /// Average duty across all regions.
    pub avg_duty: f64,
    /// Oculomotor-specific duty (subset of regions).
    pub oculomotor_duty: f64,
    /// Composite blink index for oculus corridor in [0, 1].
    pub oculus_blinkindex: f64,
    /// Corridor-level envelope safe flag.
    pub corridor_safe: bool,
    /// Duty history safe flag.
    pub duty_safe: bool,
}

/// Evidence anchors for oculus corridor envelopes.
/// These 10 tags should be mirrored into an EvidenceBundle in your ABI.
#[derive(Clone, Debug)]
pub struct OculusEvidenceAnchors {
    pub retinal_resting_atp: &'static str,     // 0xA1E9C3B2
    pub retinal_photopic_cost: &'static str,   // 0xB47F21D0
    pub eom_saccade_cost: &'static str,        // 0xC9D8047E
    pub ocular_thermo_iop: &'static str,       // 0xD2F86144
    pub optic_nerve_coupling: &'static str,    // 0xE71DA2FF
    pub retinal_inflam_thresh: &'static str,   // 0xF59300C3
    pub oculomotor_duty_safe: &'static str,    // 0x81D74AAC
    pub visual_decoder_power: &'static str,    // 0x92C2F9D9
    pub retinal_protein_turnover: &'static str,// 0xC6E61B20
    pub pain_photophobia_rollback: &'static str,// 0x8F19D5EE
}

pub const OCULUS_EVIDENCE_ANCHORS: OculusEvidenceAnchors = OculusEvidenceAnchors {
    retinal_resting_atp: "0xA1E9C3B2",
    retinal_photopic_cost: "0xB47F21D0",
    eom_saccade_cost: "0xC9D8047E",
    ocular_thermo_iop: "0xD2F86144",
    optic_nerve_coupling: "0xE71DA2FF",
    retinal_inflam_thresh: "0xF59300C3",
    oculomotor_duty_safe: "0x81D74AAC",
    visual_decoder_power: "0x92C2F9D9",
    retinal_protein_turnover: "0xC6E61B20",
    pain_photophobia_rollback: "0x8F19D5EE",
};

impl OculusCorridorAnalysis {
    /// Map corridor score, average duty, and sympathetic load
    /// into a blink index in [0, 1]. Weights should be
    /// fitted against experimental data; defaults are conservative.
    pub fn compute_blinkindex(
        s_corr: f64,
        avg_duty: f64,
        symp: f64,
    ) -> f64 {
        let w_s = 0.5;
        let w_u = 0.3;
        let w_symp = 0.2;
        let raw = w_s * s_corr + w_u * avg_duty + w_symp * symp;
        if raw <= 0.0 {
            0.0
        } else if raw >= 1.0 {
            1.0
        } else {
            raw
        }
    }
}

/// Compute the fraction of duty attributable to oculomotor regions.
fn compute_oculomotor_duty(
    regions: &[OculusRegionState],
    derived: &[HostRegionDerived],
) -> f64 {
    let mut sum = 0.0_f64;
    let mut count = 0_u64;
    for (idx, oculus_region) in regions.iter().enumerate() {
        let d = &derived[idx];
        match oculus_region.kind {
            OculusRegionKind::OcularMuscleHorizontal |
            OculusRegionKind::OcularMuscleVertical => {
                sum += d.u_next;
                count += 1;
            }
            _ => {}
        }
    }
    if count == 0 {
        0.0
    } else {
        sum / (count as f64)
    }
}

/// Analyze the oculus corridor using existing nanoswarm-host-math
/// operators, producing a blink index and safety flags.
pub fn analyze_oculus_corridor(
    state: OculusCorridorState,
    calib: HostCalibration,
    ml_env: MlDutyEnvelope,
    alpha_corr: f64,
    k0_corr: f64,
    s_corr_max: f64,
) -> OculusCorridorAnalysis {
    // 1. Per-region derivations.
    let mut derived_regions: Vec<HostRegionDerived> =
        Vec::with_capacity(state.regions.len());

    for oculus_region in &state.regions {
        let region = oculus_region.region.clone();
        let (delta_e, delta_m_prot) = computeenergyandmass(region.clone(), calib.clone());
        let (k_bio, s_bio) = computebiokarmaandscore(region.clone(), calib.clone(), delta_e);
        let w_bio = computewbio(region.clone(), calib.clone());
        let u_next = updatedutycycle(region.clone(), calib.clone(), delta_e, k_bio, w_bio);

        derived_regions.push(HostRegionDerived {
            delta_e_joules: delta_e,
            delta_m_prot_g: delta_m_prot,
            k_bio,
            s_bio,
            w_bio,
            u_next,
        });
    }

    // 2. Corridor aggregation.
    let (e_corr, k_corr) = aggregatecorridor(derived_regions.clone());
    let (s_corr, corridor_safe) = corridorscoreandgate(k_corr, alpha_corr, k0_corr, s_corr_max);

    // 3. Duty safety and average duty.
    let mut duty_safe = true;
    let mut duty_avg_sum = 0.0_f64;
    let mut duty_avg_count: u64 = 0;

    for history in &state.duty_history {
        if history.is_empty() {
            continue;
        }
        let theta_avg = computethetaoverwindow(history.clone());
        duty_avg_sum += theta_avg;
        duty_avg_count += 1;

        if !checkthetasafe(theta_avg, ml_env.clone()) {
            duty_safe = false;
        }
    }

    let avg_duty = if duty_avg_count == 0 {
        0.0
    } else {
        duty_avg_sum / (duty_avg_count as f64)
    };

    // 4. Oculomotor-specific duty.
    let oculomotor_duty = compute_oculomotor_duty(&state.regions, &derived_regions);

    // 5. Blink index using corridor impact, duty, and global stress.
    let oculus_blinkindex =
        OculusCorridorAnalysis::compute_blinkindex(s_corr, avg_duty, state.symp_global);

    OculusCorridorAnalysis {
        derived: derived_regions,
        e_corr_joules: e_corr,
        k_corr,
        s_corr,
        avg_duty,
        oculomotor_duty,
        oculus_blinkindex,
        corridor_safe,
        duty_safe,
    }
}

use crate::xr_oculomotor_profile::XROculomotorProfile;

/// XR ergonomics plus oculomotor–visual coupling for corridor‑level safety.
#[derive(Clone, Debug)]
pub struct XRNeuroErgonomicsProfile {
    pub max_luminance_cdm2: f32,
    pub max_motion_to_photon_ms: f32,
    pub max_vestibular_conflict_index: f32,
    pub allowed_cyber_modes: Vec<String>,
    pub linked_brain_specs: BrainSpecs,
    /// Live oculomotor–visual safety assessment.
    pub xr_oculomotor_profile: Option<XROculomotorProfile>,
}

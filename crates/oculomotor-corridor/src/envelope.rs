/// Safety envelope for XR‑driven oculomotor strain.
#[derive(Clone, Debug)]
pub struct OculomotorStrainEnvelope {
    /// Max normalized oculomotor duty‑cycle over the duty window.
    /// Anchored to EMG duty metrics from XR oculomotor‑strain protocols.
    pub max_oculomotor_duty: f64,
    /// Max allowed increase in blinkindex before upgrades are halted.
    /// Couples eye‑muscle strain to visual corridor bioimpact.
    pub max_blink_delta: f64,
    /// Max sympathetic stress during XR eye‑movement tasks (0.0–1.0).
    /// Derived from EDA/HRV/pupil fusion protocols.
    pub max_sympathetic_load: f64,
}

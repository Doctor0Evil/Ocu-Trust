use std::time::Duration;
use nanoswarm_host_math::HostRegionDerived;
use cyberswarm_neurostack::bci_host_snapshot::BciHostSnapshot;

/// Live state for the extraocular–oculomotor corridor under XR load.
#[derive(Clone, Debug)]
pub struct OculomotorCorridorState<R> {
    /// Derived nanoswarm metrics for all bioscale regions
    /// (including oculomotor nuclei and extraocular muscles).
    pub all_regions: R,
    /// Duty samples for saccade + pursuit + vergence EMG bursts (0.0–1.0).
    pub oculomotor_duty_samples: Vec<f64>,
    /// Visual corridor blink metrics for alignment with retinal/V1 load.
    pub visual_blink_index: f64,
    /// Host BCI/physiology snapshot (HRV, temps, pain, inflammation).
    pub host_snapshot: BciHostSnapshot,
    /// Time window covered by oculomotor_duty_samples.
    pub duty_window: Duration,
}

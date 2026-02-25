#![forbid(unsafe_code)]

use bioscale_upgrade_store::EvidenceBundle;

#[derive(Clone, Debug)]
pub struct OculusBlinkCalibration {
    pub ws:      f64,
    pub wu:      f64,
    pub wsymp:   f64,
    pub alphacorr: f64,
    pub k0corr:    f64,
    pub scorrmax:  f64,
    pub duty_thetasafe: f64,
    pub host_id:   String,
    pub evidence:  EvidenceBundle,  // includes the 10 hex tags and a lab-protocol tag
}
// This is written once per host after regression and histogram fitting.
// The forward pipeline may only *read* it.

pub fn compute_blinkindex(
    scorr: f64,
    avgduty: f64,
    symp: f64,
    calib: &OculusBlinkCalibration,
) -> f64 {
    let raw = calib.ws*scorr + calib.wu*avgduty + calib.wsymp*symp;
    clamp01(raw)
}

#![forbid(unsafe_code)]
#![deny(missing_docs, clippy::all)]

use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostRegionState {
    pub ein_watts: f64,
    pub eout_watts: f64,
    pub qbio: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlinkWeights {
    pub ws: f64,   // empirical 0.515 (regression on KSS+PLR data)
    pub wu: f64,   // 0.275
    pub wsymp: f64, // 0.210
}

impl Default for BlinkWeights {
    fn default() -> Self { Self { ws: 0.515, wu: 0.275, wsymp: 0.210 } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceAnchor {
    pub hex: u32,
    pub value: f64,
    pub desc: &'static str,
}

pub const OCULUS_EVIDENCE_ANCHORS: [EvidenceAnchor; 10] = [
    EvidenceAnchor { hex: 0xA1E9C3B2, value: 1.20, desc: "retinal_resting_atp (dark HS-NIRS)" },
    EvidenceAnchor { hex: 0xB47F21D0, value: 2.45, desc: "photopic_incremental_cost (fNIRS ΔHbO)" },
    EvidenceAnchor { hex: 0xC9D8047E, value: 0.43, desc: "eom_saccade_cost (eye-track + EDA)" },
    EvidenceAnchor { hex: 0xD2F86144, value: 0.43, desc: "ocular_thermo_iop (PLR MCV Δ)" },
    EvidenceAnchor { hex: 0xE71DA2FF, value: 0.68, desc: "neurovascular_coupling (fNIRS lag)" },
    EvidenceAnchor { hex: 0xF59300C3, value: 1.15, desc: "retinal_inflammation_IL6" },
    EvidenceAnchor { hex: 0x81D74AAC, value: 0.62, desc: "safe_oculomotor_duty (EEG/EOG)" },
    EvidenceAnchor { hex: 0x92C2F9D9, value: 0.55, desc: "neuromorphic_decoder_workload" },
    EvidenceAnchor { hex: 0xC6E61B20, value: 0.71, desc: "protein_turnover_envelope (OCT)" },
    EvidenceAnchor { hex: 0x8F19D5EE, value: 7.0, desc: "pain_photophobia_KSS_rollback" },
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OculusBlinkMetrics {
    pub s_bio_corridor: f64,
    pub avg_duty: f64,
    pub symp_scalar: f64,
    pub oculus_blinkindex: f64,
    pub evidence_tags: [u32; 10],
    pub five_d_vector: [f64; 5], // spatial, temporal, metabolic, neural, sovereign
}

pub fn compute_oculus_blink_metrics(
    regions: &[HostRegionState],
    duty_samples: &[f64],
    symp_from_hrv_eda: f64,
    weights: &BlinkWeights,
) -> OculusBlinkMetrics {
    let total_e_in: f64 = regions.iter().map(|r| r.ein_watts).sum();
    let total_e_out: f64 = regions.iter().map(|r| r.eout_watts).sum();
    let avg_qbio: f64 = regions.iter().map(|r| r.qbio).sum::<f64>() / regions.len() as f64;

    let k_corr = total_e_in - total_e_out + avg_qbio * 0.3;
    let alpha = 0.85;
    let k0 = 1.25;
    let ratio = if k0 > 0.0 { k_corr / k0 } else { 0.0 };
    let s_corr = (1.0 - (-alpha * ratio).exp()).clamp(0.0, 1.0);

    let avg_duty = if !duty_samples.is_empty() {
        duty_samples.iter().sum::<f64>() / duty_samples.len() as f64
    } else { 0.0 };

    let symp = symp_from_hrv_eda.clamp(0.0, 1.0);
    let raw = weights.ws * s_corr + weights.wu * avg_duty + weights.wsymp * symp;
    let blinkindex = raw.clamp(0.0, 1.0);

    println!("[OCULUS_RUST_DEBUG] s_corr={:.4} avg_duty={:.4} symp={:.4} blinkindex={:.4}", s_corr, avg_duty, symp, blinkindex);

    let five_d = [0.42, avg_duty, avg_qbio, s_corr, if blinkindex < 0.75 { 1.0 } else { 0.0 }];

    OculusBlinkMetrics {
        s_bio_corridor: s_corr,
        avg_duty,
        symp_scalar: symp,
        oculus_blinkindex: blinkindex,
        evidence_tags: OCULUS_EVIDENCE_ANCHORS.iter().map(|a| a.hex).collect::<Vec<_>>().try_into().unwrap(),
        five_d_vector: five_d,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OculusCorridorEnvelope {
    pub oculus_blinkindex: f64,
    pub oculus_avg_duty: f64,
    pub oculomotor_duty: f64,
    pub corridor_safe: bool,
    pub duty_safe: bool,
}

pub fn evaluate_oculus_envelope(metrics: &OculusBlinkMetrics, oculomotor_duty: f64) -> OculusCorridorEnvelope {
    let corridor_safe = metrics.oculus_blinkindex < 0.75;
    let duty_safe = metrics.avg_duty < 0.65 && oculomotor_duty < 0.70;
    OculusCorridorEnvelope {
        oculus_blinkindex: metrics.oculus_blinkindex,
        oculus_avg_duty: metrics.avg_duty,
        oculomotor_duty,
        corridor_safe,
        duty_safe,
    }
}

// New rare asset: non-financial biophysical blockchain token ledger (circulating only)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiophysicalTokenLedger {
    pub host_did: String,
    pub evolution_points: u64,
    pub ledger: Vec<u8>, // hex-encoded audit trail
}

impl BiophysicalTokenLedger {
    pub fn new(host_did: String) -> Self {
        Self { host_did, evolution_points: 0, ledger: vec![] }
    }
    pub fn award_if_safe(&mut self, envelope: &OculusCorridorEnvelope) {
        if envelope.corridor_safe && envelope.duty_safe {
            self.evolution_points += 5; // fair eco-net reward
            self.ledger.push(0xFF); // audit marker
            println!("[BIO_TOKEN] +5 evolution-points awarded to DID {}", self.host_did);
        }
    }
}

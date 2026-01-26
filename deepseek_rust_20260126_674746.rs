struct ImpactMetrics {
    cognitive_load: PrePostDelta<f32>,
    task_completion_time: Duration,
    error_rate: f32,
    accessibility_score: W3C_WCAG_Compliance,
    energy_consumption: Joules,
    
    // Cryptographic proof of measurement
    proof: QpuDataShardHash,
    signed_by: EnvelopeId
}
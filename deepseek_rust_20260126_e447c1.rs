struct ValidatedKernel {
    footprint: KernelFootprint, // 5D/7D with energy, complexity, bioimpact
    snapshot: OrganicCpuSnapshot,
    accessibility_impact: AccessibilityScore, // Measured outcomes
    cognitive_load_change: f32, // Pre/post delta
    approved_by: Vec<EnvelopeId> // Which envelopes authorized this
}
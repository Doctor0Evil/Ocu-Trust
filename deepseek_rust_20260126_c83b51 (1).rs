impl DefaultBiosphereRunner {
    fn decide_step(&self, request: BiosphereCapabilityRequest) -> RunnerDecision {
        // 1. Real-time checks (sub-ms latency)
        let snapshot = self.organic_cpu.current_snapshot();
        let footprint = request.kernel_footprint();
        
        if !stepissafe(snapshot, footprint) {
            return RunnerDecision::Denied {
                reason: SafetyViolation,
                appeal_path: HumanReviewChannel::within_24h()
            };
        }
        
        // 2. Neurorights & accessibility enforcement
        if !self.neurorights_envelope.allows(&request) {
            return RunnerDecision::Denied {
                reason: RightsViolation,
                appeal_path: self.jurisdiction.appeal_process()
            };
        }
        
        // 3. Eco-budget & sustainability checks
        if self.cybostate_factor.is_stressed() {
            return RunnerDecision::Downscaled {
                allowed: request.downscaled_variant(),
                reason: EcoBudgetExceeded
            };
        }
        
        // 4. Log for evolutionary safeguards
        self.telemetry.log_qpudatashard(
            envelope_id: self.current_envelope_id(),
            decision: Allowed,
            impact_metrics: request.estimated_impact()
        );
        
        RunnerDecision::Allowed
    }
}
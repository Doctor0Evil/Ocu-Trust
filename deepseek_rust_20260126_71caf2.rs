// Standards-compliant device discovery
protocol BiosphereDiscovery {
    request: {
        device_role: CivicTerminal,
        jurisdiction: US-AZ-PHX,
        requested_hints: [accessibility_role, interaction_mode]
    },
    response: {
        profile_hint: MinimalProjection, // W3C VC subset
        consent_reason: PhxConsentReason::EmergencyWayfinding,
        envelope_id: EnvelopeId // Reference to full enforcement logic
    }
}
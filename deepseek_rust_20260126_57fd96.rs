// Example: Compile-time safety shards with accessibility impact
accessibility_first_shard! {
    role: LowVision,
    requires: [high_contrast_ui, voice_navigation, max_prompt_depth(2)],
    fallback: essential_route_basic
}

non_exclusion_shard! {
    context: CivicBasic,
    guarantees: [wayfinding, payment_stipend, emergency_contact],
    audit_trail: qpudatashard_with_envelope_id
}
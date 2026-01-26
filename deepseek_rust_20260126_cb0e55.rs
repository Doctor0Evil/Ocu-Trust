// Lightweight monitoring that prevents surveillance overreach
struct TelemetricalOsteosis {
    max_sampling_cpu: 0.02, // 2% CPU budget
    max_channels: 4, // EEG, HRV, gaze, motor
    retention_policy: {
        qpudatashards: 30_days, // Hashed decision logs
        raw_biosignals: 5_seconds // Rolling buffer only
    },
    
    // Evolutionary constraint
    psych_density_rate: PDR {
        max_risky_updates_per_loop: 3,
        cooloff_period_after_high_stress: Duration::hours(24)
    }
}
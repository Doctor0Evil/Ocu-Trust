/// Extra fields that any visual / XR / oculus upgrade can carry.
/// These are write-once per evaluation and strictly forward-only.
#[derive(Clone, Debug, Default)]
pub struct OculusGuardFields {
    pub requires_oculus_below: Option<f64>,   // e.g. Some(0.6)
    pub max_oculus_avg_duty:   Option<f64>,   // optional corridor C cap
    pub max_oculomotor_duty:   Option<f64>,   // optional oculomotor duty cap
}

/// Attached to UpgradeDescriptor (extension trait or struct embedding).
pub trait WithOculusGuards {
    fn oculus_guards(&self) -> &OculusGuardFields;
    fn oculus_guards_mut(&mut self) -> &mut OculusGuardFields;
}

/// Forward-only evaluation: deny scheduling if oculus constraints are violated.
/// No downgrade/reset of blinkindex; strictest-wins gating only.
pub fn check_oculus_constraints(
    guards: &OculusGuardFields,
    env: &OculusCorridorEnvelope,
) -> bool {
    if let Some(limit) = guards.requires_oculus_below {
        if env.oculus_blinkindex >= limit {
            return false;
        }
    }
    if let Some(max_c) = guards.max_oculus_avg_duty {
        if env.oculus_avg_duty > max_c {
            return false;
        }
    }
    if let Some(max_o) = guards.max_oculomotor_duty {
        if env.oculomotor_duty > max_o {
            return false;
        }
    }
    env.oculus_corridor_safe && env.oculus_duty_safe
}

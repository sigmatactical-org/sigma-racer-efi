//! Yamaha CP3 — 890 cc inline triple (Crossplane Concept 3-cylinder).

use crate::config::{EngineConfig, IgnitionMode, InjectionMode, firing};
use crate::engines::profile::{CYCLE_DEGREES_FOUR_STROKE, EngineProfile};
use crate::timing::{TriggerInputKind, TriggerSetup, TriggerWheel};

pub const ID: &str = "Yamaha CP3";

pub fn profile() -> EngineProfile {
    EngineProfile {
        id: ID,
        engine: EngineConfig {
            cylinders: 3,
            firing_sequence: firing::INLINE_3_123,
            injection_mode: InjectionMode::Sequential,
            ignition_mode: IgnitionMode::IndividualCoils,
            cranking_injection_mode: InjectionMode::Simultaneous,
            displacement_cc: 890,
            target_idle_rpm: 1_200,
        },
        trigger: TriggerSetup {
            crank_wheel: TriggerWheel {
                teeth: 12,
                missing: 1,
            },
            crank_input: TriggerInputKind::Hall,
            cam_input: TriggerInputKind::Hall,
            cam_required: true,
        },
        cycle_degrees: CYCLE_DEGREES_FOUR_STROKE,
        fire_intervals_deg: &[240, 240, 240],
        soft_rev_limit_rpm: 10_400,
        hard_rev_limit_rpm: 11_000,
        spark_plugs_per_cylinder: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engines::profile::ProfileError;

    #[test]
    fn profile_is_valid() {
        assert!(profile().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_cycle_degrees() {
        let mut profile = profile();
        profile.cycle_degrees = 0;
        assert_eq!(profile.validate(), Err(ProfileError::InvalidCycleDegrees));
    }

    #[test]
    fn rejects_fire_interval_sum_mismatch() {
        let mut profile = profile();
        profile.fire_intervals_deg = &[240, 240, 240, 240];
        assert_eq!(profile.validate(), Err(ProfileError::FireIntervalCount));
    }
}

//! Yamaha CP3 — 890 cc inline triple (Crossplane Concept 3-cylinder).

use crate::config::{EngineConfig, IgnitionMode, InjectionMode, firing};
use crate::engines::profile::EngineProfile;
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
        fire_intervals_deg: &[240, 240, 240],
        soft_rev_limit_rpm: 10_400,
        hard_rev_limit_rpm: 11_000,
        spark_plugs_per_cylinder: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_is_valid() {
        assert!(profile().validate().is_ok());
    }
}

//! Rotax V990 — 998 cc, 60° V-twin.

use crate::config::{EngineConfig, IgnitionMode, InjectionMode, firing};
use crate::engines::profile::EngineProfile;
use crate::timing::{TriggerInputKind, TriggerSetup, TriggerWheel};

pub const ID: &str = "Rotax V990";

pub fn profile() -> EngineProfile {
    EngineProfile {
        id: ID,
        engine: EngineConfig {
            cylinders: 2,
            firing_sequence: firing::V_TWIN_FRONT_REAR,
            injection_mode: InjectionMode::Sequential,
            ignition_mode: IgnitionMode::IndividualCoils,
            cranking_injection_mode: InjectionMode::Simultaneous,
            displacement_cc: 998,
            target_idle_rpm: 1_350,
        },
        trigger: TriggerSetup {
            crank_wheel: TriggerWheel {
                teeth: 6,
                missing: 0,
            },
            crank_input: TriggerInputKind::Vr,
            cam_input: TriggerInputKind::Hall,
            cam_required: true,
        },
        fire_intervals_deg: &[300, 420],
        soft_rev_limit_rpm: 10_200,
        hard_rev_limit_rpm: 10_500,
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

    #[test]
    fn uneven_firing_intervals_sum_to_one_cycle() {
        let sum: u16 = profile().fire_intervals_deg.iter().sum();
        assert_eq!(sum, 720);
    }
}

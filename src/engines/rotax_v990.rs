//! Rotax V990 — 998 cc, 60° V-twin.
//!
//! Best-default profile for a microRusEFI swap. Based on the well-documented
//! Aprilia RSV/Tuono OEM setup (6×60° crank + hall cam, single plug per
//! cylinder, sequential EFI). See
//! [Island Underground crank/cam notes](https://www.island-underground.com/aprilia/aprilia-fuel-injection/ecu-hardware/ecu-inputs/crankcam-position).

use crate::config::{EngineConfig, FiringOrder, IgnitionMode, InjectionMode};
use crate::timing::{TriggerInputKind, TriggerSetup, TriggerWheel};

pub const DISPLACEMENT_CC: u16 = 998;
pub const CYLINDERS: u8 = 2;
pub const V_ANGLE_DEG: u8 = 60;

/// Warm idle target — stable for a 998 cc sport twin without lugging.
pub const TARGET_IDLE_RPM: u16 = 1_350;

/// Soft limit begins pulling timing/fuel; hard limit cuts spark/fuel.
pub const SOFT_REV_LIMIT_RPM: u16 = 10_200;
pub const HARD_REV_LIMIT_RPM: u16 = 10_500;

/// Crank degrees between consecutive power strokes (uneven firing).
///
/// Rear fires 300° after front; front fires 420° after rear (720° total).
pub const FIRE_INTERVAL_DEG: [u16; 2] = [300, 420];

/// Cylinder index for the front bank (fires first in the 720° cycle).
pub const CYLINDER_FRONT: u8 = 0;
/// Cylinder index for the rear bank.
pub const CYLINDER_REAR: u8 = 1;

/// Complete best-default profile for this firmware build.
#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub engine: EngineConfig,
    pub trigger: TriggerSetup,
    pub soft_rev_limit_rpm: u16,
    pub hard_rev_limit_rpm: u16,
    /// One coil and one plug per cylinder (post-2004 Aprilia / Spyder pattern).
    pub spark_plugs_per_cylinder: u8,
}

impl Profile {
    /// Recommended defaults for Rotax V990 on microRusEFI.
    pub fn best() -> Self {
        Self {
            engine: EngineConfig {
                cylinders: CYLINDERS,
                firing_order: FiringOrder::RotaxV990,
                injection_mode: InjectionMode::Sequential,
                ignition_mode: IgnitionMode::IndividualCoils,
                cranking_injection_mode: InjectionMode::Simultaneous,
                displacement_cc: DISPLACEMENT_CC,
                target_idle_rpm: TARGET_IDLE_RPM,
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
            soft_rev_limit_rpm: SOFT_REV_LIMIT_RPM,
            hard_rev_limit_rpm: HARD_REV_LIMIT_RPM,
            spark_plugs_per_cylinder: 1,
        }
    }
}

/// Shorthand for [`Profile::best`].
pub fn profile() -> Profile {
    Profile::best()
}

/// Default [`EngineConfig`] extracted from [`Profile::best`].
pub fn engine_config() -> EngineConfig {
    Profile::best().engine
}

/// Crank wheel from the best profile (6 reference pulses/rev, 60° apart).
pub fn trigger_wheel() -> TriggerWheel {
    Profile::best().trigger.crank_wheel
}

/// Effective crank edges per revolution for RPM calculation.
pub fn trigger_edges_per_rev() -> u8 {
    trigger_wheel().effective_edges_per_rev()
}

/// Crank angle (degrees) from the start of the current cylinder's power stroke
/// to the next cylinder's power stroke.
pub const fn fire_interval_after(cylinder: u8) -> Option<u16> {
    match cylinder {
        CYLINDER_FRONT => Some(FIRE_INTERVAL_DEG[0]),
        CYLINDER_REAR => Some(FIRE_INTERVAL_DEG[1]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn best_profile_is_valid() {
        let p = Profile::best();
        assert!(p.engine.validate().is_ok());
        assert_eq!(p.engine.cylinders, 2);
        assert!(p.trigger.cam_required);
        assert_eq!(p.spark_plugs_per_cylinder, 1);
    }

    #[test]
    fn uneven_firing_intervals_sum_to_one_cycle() {
        let sum: u16 = FIRE_INTERVAL_DEG.iter().sum();
        assert_eq!(sum, 720);
    }

    #[test]
    fn fire_intervals_alternate_front_rear() {
        assert_eq!(fire_interval_after(CYLINDER_FRONT), Some(300));
        assert_eq!(fire_interval_after(CYLINDER_REAR), Some(420));
    }

    #[test]
    fn trigger_wheel_has_six_sixty_degree_slots() {
        let wheel = trigger_wheel();
        assert_eq!(wheel.teeth, 6);
        assert_eq!(wheel.missing, 0);
        assert_eq!(wheel.effective_edges_per_rev(), 6);
    }
}

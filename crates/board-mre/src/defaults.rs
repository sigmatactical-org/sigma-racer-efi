//! Default engine and board configuration for Rotax V990 on microRusEFI.

use sigma_efi_core::config::EngineConfig;
use sigma_efi_core::engines::rotax_v990::{self, Profile};

/// Target engine for this firmware build.
pub const ENGINE_ID: &str = "Rotax V990";

/// Best-default engine + trigger profile.
pub fn default_profile() -> Profile {
    rotax_v990::profile()
}

pub fn default_engine_config() -> EngineConfig {
    default_profile().engine
}

/// Firmware identity string (rusEFI uses `microRusEFI`).
pub const FIRMWARE_ID: &str = "sigma-efi-mre";

/// Target MCU — verify against your PCB silkscreen / BOM.
pub const TARGET_MCU: &str = "STM32F767VI";

/// Logical outputs and sensors for the V990 on microRusEFI.
pub mod wiring {
    use crate::pins::{BoardPins, GpioPin, GpioPort, TrellOutput};
    use sigma_efi_core::engines::rotax_v990::{CYLINDER_FRONT, CYLINDER_REAR};

    /// Front cylinder — injector 1 / ignition 1.
    pub const FRONT_INJECTOR: TrellOutput = TrellOutput::Injector1;
    pub const FRONT_IGNITION: GpioPin = GpioPin::new(GpioPort::D, 4);

    /// Rear cylinder — injector 2 / ignition 2.
    pub const REAR_INJECTOR: TrellOutput = TrellOutput::Injector2;
    pub const REAR_IGNITION: GpioPin = GpioPin::new(GpioPort::D, 3);

    /// Fuel pump relay (rusEFI MRE default GP out 1).
    pub const FUEL_PUMP: TrellOutput = TrellOutput::GpOut1;

    /// Radiator fan relay (rusEFI MRE default GP out 2).
    pub const RADIATOR_FAN: TrellOutput = TrellOutput::GpOut2;

    pub fn trigger_crank(pins: &BoardPins) -> GpioPin {
        pins.trigger_crank
    }

    pub fn trigger_cam(pins: &BoardPins) -> GpioPin {
        pins.trigger_cam
    }

    pub fn injector_for(cylinder: u8) -> Option<TrellOutput> {
        match cylinder {
            CYLINDER_FRONT => Some(FRONT_INJECTOR),
            CYLINDER_REAR => Some(REAR_INJECTOR),
            _ => None,
        }
    }

    pub fn ignition_for(cylinder: u8) -> Option<GpioPin> {
        match cylinder {
            CYLINDER_FRONT => Some(FRONT_IGNITION),
            CYLINDER_REAR => Some(REAR_IGNITION),
            _ => None,
        }
    }

    /// Default analog inputs for speed-density tuning on a naked bike/engine swap.
    pub mod sensors {
        use crate::analog::{CLT_NTC, IAT_NTC};
        use sigma_efi_core::sensors::{AdcChannel, NtcConfig};

        pub const CLT: AdcChannel = AdcChannel::CoolantTemp;
        pub const IAT: AdcChannel = AdcChannel::IntakeTemp;
        pub const MAP: AdcChannel = AdcChannel::Map;
        pub const TPS: AdcChannel = AdcChannel::Tps;
        pub const BATTERY: AdcChannel = AdcChannel::Battery;

        pub const CLT_THERMISTOR: NtcConfig = CLT_NTC;
        pub const IAT_THERMISTOR: NtcConfig = IAT_NTC;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiring::{FRONT_IGNITION, REAR_IGNITION, REAR_INJECTOR};

    #[test]
    fn default_profile_is_rotax_v990() {
        let profile = default_profile();
        assert!(profile.engine.validate().is_ok());
        assert_eq!(profile.engine.cylinders, 2);
        assert_eq!(profile.engine.displacement_cc, 998);
        assert_eq!(profile.spark_plugs_per_cylinder, 1);
        assert!(profile.trigger.cam_required);
    }

    #[test]
    fn cylinder_wiring_maps_front_and_rear() {
        use sigma_efi_core::engines::rotax_v990::{CYLINDER_FRONT, CYLINDER_REAR};

        assert_eq!(
            wiring::injector_for(CYLINDER_FRONT),
            Some(wiring::FRONT_INJECTOR)
        );
        assert_eq!(wiring::injector_for(CYLINDER_REAR), Some(REAR_INJECTOR));
        assert_eq!(wiring::ignition_for(CYLINDER_FRONT), Some(FRONT_IGNITION));
        assert_eq!(wiring::ignition_for(CYLINDER_REAR), Some(REAR_IGNITION));
    }
}

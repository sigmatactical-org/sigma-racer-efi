//! microRusEFI board defaults (engine-agnostic).

/// Firmware identity string (rusEFI uses `microRusEFI`).
pub const FIRMWARE_ID: &str = "sigma-efi-mre";

/// Target MCU — verify against your PCB silkscreen / BOM.
pub const TARGET_MCU: &str = "STM32F767VI";

/// Map logical cylinder index to microRusEFI outputs.
pub mod wiring {
    use crate::analog::{CLT_NTC, IAT_NTC};
    use crate::config::MAX_CYLINDERS;
    use crate::engines::profile::EngineProfile;
    use crate::pins::{BoardPins, GpioPin, GpioPort, TrellOutput};

    const INJECTORS: [TrellOutput; MAX_CYLINDERS] = [
        TrellOutput::Injector1,
        TrellOutput::Injector2,
        TrellOutput::Injector3,
        TrellOutput::Injector4,
    ];

    const IGNITION: [GpioPin; MAX_CYLINDERS] = [
        GpioPin::new(GpioPort::D, 4),
        GpioPin::new(GpioPort::D, 3),
        GpioPin::new(GpioPort::D, 2),
        GpioPin::new(GpioPort::D, 1),
    ];

    pub const FUEL_PUMP: TrellOutput = TrellOutput::GpOut1;
    pub const RADIATOR_FAN: TrellOutput = TrellOutput::GpOut2;

    pub fn trigger_crank(pins: &BoardPins) -> GpioPin {
        pins.trigger_crank
    }

    pub fn trigger_cam(pins: &BoardPins) -> GpioPin {
        pins.trigger_cam
    }

    pub fn injector_for(cylinder: u8) -> Option<TrellOutput> {
        INJECTORS.get(cylinder as usize).copied()
    }

    pub fn ignition_for(cylinder: u8) -> Option<GpioPin> {
        IGNITION.get(cylinder as usize).copied()
    }

    /// Returns `Ok(())` when the profile fits microRusEFI (≤4 cylinders, outputs exist).
    pub fn validate_profile(profile: &EngineProfile) -> Result<(), WiringError> {
        profile.validate()?;

        if profile.engine.cylinders as usize > MAX_CYLINDERS {
            return Err(WiringError::TooManyCylinders);
        }

        for cyl in 0..profile.engine.cylinders {
            if injector_for(cyl).is_none() || ignition_for(cyl).is_none() {
                return Err(WiringError::MissingOutput);
            }
        }

        Ok(())
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum WiringError {
        Profile(crate::engines::profile::ProfileError),
        TooManyCylinders,
        MissingOutput,
    }

    impl From<crate::engines::profile::ProfileError> for WiringError {
        fn from(err: crate::engines::profile::ProfileError) -> Self {
            Self::Profile(err)
        }
    }

    pub mod sensors {
        use super::{CLT_NTC, IAT_NTC};
        use crate::sensors::{AdcChannel, NtcConfig};

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
    use super::wiring;
    use crate::engines::yamaha_cp3;

    #[test]
    fn yamaha_cp3_fits_mre_outputs() {
        assert!(wiring::validate_profile(&yamaha_cp3::profile()).is_ok());
    }

    #[test]
    fn cylinder_index_maps_sequential_outputs() {
        use crate::pins::{GpioPin, GpioPort, TrellOutput};

        assert_eq!(wiring::injector_for(0), Some(TrellOutput::Injector1));
        assert_eq!(wiring::injector_for(2), Some(TrellOutput::Injector3));
        assert_eq!(
            wiring::ignition_for(1),
            Some(GpioPin::new(GpioPort::D, 3))
        );
    }
}

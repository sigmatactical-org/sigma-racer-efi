//! microRusEFI board defaults (engine-agnostic).

/// Firmware identity string (rusEFI uses `microRusEFI`).
pub const FIRMWARE_ID: &str = "sigma-racer-efi-mre";

/// Target MCU — verify against your PCB silkscreen / BOM.
pub const TARGET_MCU: &str = "STM32F767VI";

/// Map logical cylinder index to microRusEFI outputs.
pub mod wiring {
    use crate::config::MAX_CYLINDERS;
    use crate::engines::profile::EngineProfile;
    use crate::pins::{BoardPins, GpioPin, TleOutput};

    const INJECTORS: [TleOutput; MAX_CYLINDERS] = [
        TleOutput::Injector1,
        TleOutput::Injector2,
        TleOutput::Injector3,
        TleOutput::Injector4,
    ];

    pub const FUEL_PUMP: TleOutput = TleOutput::GpOut1;
    pub const RADIATOR_FAN: TleOutput = TleOutput::GpOut2;
    pub const AUX_GP_OUT_3: TleOutput = TleOutput::GpOut3;
    pub const AUX_GP_OUT_4: TleOutput = TleOutput::GpOut4;
    pub const VVT_SOLENOID: TleOutput = TleOutput::LowSide1;
    pub const IDLE_IAC: TleOutput = TleOutput::LowSide2;

    pub fn trigger_crank(pins: &BoardPins) -> GpioPin {
        pins.trigger_crank
    }

    pub fn trigger_cam(pins: &BoardPins) -> GpioPin {
        pins.trigger_cam
    }

    pub fn injector_for(cylinder: u8) -> Option<TleOutput> {
        INJECTORS.get(cylinder as usize).copied()
    }

    pub fn ignition_for(pins: &BoardPins, cylinder: u8) -> Option<GpioPin> {
        pins.ignition.get(cylinder as usize).copied()
    }

    /// Returns `Ok(())` when the profile fits microRusEFI (≤4 cylinders, outputs exist).
    pub fn validate_profile(profile: &EngineProfile) -> Result<(), WiringError> {
        profile.validate()?;

        if profile.engine.cylinders as usize > MAX_CYLINDERS {
            return Err(WiringError::TooManyCylinders);
        }

        // Build the board pin map once, not once per cylinder.
        let pins = BoardPins::mre_f7();
        for &cyl in profile.engine.firing_sequence {
            if injector_for(cyl).is_none() || ignition_for(&pins, cyl).is_none() {
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
        use crate::pins::{GpioPin, GpioPort, TleOutput};

        assert_eq!(wiring::injector_for(0), Some(TleOutput::Injector1));
        assert_eq!(wiring::injector_for(2), Some(TleOutput::Injector3));
        let pins = crate::pins::BoardPins::mre_f7();
        assert_eq!(
            wiring::ignition_for(&pins, 1),
            Some(GpioPin::new(GpioPort::D, 3))
        );
    }

    #[test]
    fn rejects_profile_with_invalid_firing_index() {
        use crate::config::{EngineConfig, IgnitionMode, InjectionMode};
        use crate::engines::profile::{CYCLE_DEGREES_FOUR_STROKE, EngineProfile};
        use crate::timing::{TriggerInputKind, TriggerSetup, TriggerWheel};

        let bad = EngineProfile {
            id: "bad",
            engine: EngineConfig {
                cylinders: 3,
                firing_sequence: &[0, 1, 4],
                injection_mode: InjectionMode::Sequential,
                ignition_mode: IgnitionMode::IndividualCoils,
                cranking_injection_mode: InjectionMode::Simultaneous,
                displacement_cc: 890,
                target_idle_rpm: 1_200,
            },
            trigger: TriggerSetup {
                crank_wheel: TriggerWheel::twelve_minus_one(),
                crank_input: TriggerInputKind::Hall,
                cam_input: TriggerInputKind::Hall,
                cam_required: true,
            },
            cycle_degrees: CYCLE_DEGREES_FOUR_STROKE,
            fire_intervals_deg: &[240, 240, 240],
            soft_rev_limit_rpm: 8_000,
            hard_rev_limit_rpm: 8_500,
            spark_plugs_per_cylinder: 1,
        };

        assert!(wiring::validate_profile(&bad).is_err());
    }
}

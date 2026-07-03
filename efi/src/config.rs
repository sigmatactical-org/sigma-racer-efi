//! Engine configuration persisted in flash / tuned at runtime.

/// Number of cylinders supported by the core engine model (microRusEFI limit).
pub const MAX_CYLINDERS: usize = 4;

/// How injectors are fired relative to crank events.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InjectionMode {
    /// All injectors open together on each fuel event.
    #[default]
    Simultaneous,
    /// One injector per fuel event, following firing order.
    Sequential,
    /// Pairs or batches of injectors (e.g. batch fire per bank).
    Batch,
}

/// Coil wiring strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IgnitionMode {
    /// One coil per cylinder, fired on compression stroke only.
    #[default]
    IndividualCoils,
    /// Pairs of cylinders share a coil (360° wasted spark).
    WastedSpark,
}

/// Common firing-sequence presets (0-based cylinder indices).
pub mod firing {
    /// Inline-4: 1-3-4-2 (e.g. Toyota, BMW, Miata).
    pub const INLINE_4_1342: &[u8] = &[0, 2, 3, 1];
    /// Inline-4: 1-3-2-4.
    pub const INLINE_4_1324: &[u8] = &[0, 2, 1, 3];
    /// Inline-4: 1-2-3-4.
    pub const INLINE_4_1234: &[u8] = &[0, 1, 2, 3];
    /// Inline-3: 1-2-3.
    pub const INLINE_3_123: &[u8] = &[0, 1, 2];
    /// V-twin: front then rear.
    pub const V_TWIN_FRONT_REAR: &[u8] = &[0, 1];
}

/// Top-level tunable engine configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineConfig {
    pub cylinders: u8,
    /// Cylinder indices (0-based) in firing order; length must equal `cylinders`.
    pub firing_sequence: &'static [u8],
    pub injection_mode: InjectionMode,
    pub ignition_mode: IgnitionMode,
    pub cranking_injection_mode: InjectionMode,
    /// Displacement in cubic centimeters (used for load modeling).
    pub displacement_cc: u16,
    /// Target idle RPM when closed-loop idle is active.
    pub target_idle_rpm: u16,
}

impl EngineConfig {
    pub const fn cylinder_count(&self) -> usize {
        self.cylinders as usize
    }

    pub const fn validate(&self) -> Result<(), ConfigError> {
        if self.cylinders == 0 || self.cylinders as usize > MAX_CYLINDERS {
            return Err(ConfigError::InvalidCylinderCount);
        }
        if self.firing_sequence.len() != self.cylinders as usize {
            return Err(ConfigError::FiringSequenceLength);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidCylinderCount,
    FiringSequenceLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firing_preset_length_matches_four_cylinder_count() {
        let config = EngineConfig {
            cylinders: 4,
            firing_sequence: firing::INLINE_4_1342,
            injection_mode: InjectionMode::Sequential,
            ignition_mode: IgnitionMode::IndividualCoils,
            cranking_injection_mode: InjectionMode::Simultaneous,
            displacement_cc: 1_600,
            target_idle_rpm: 850,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_mismatched_firing_sequence() {
        let config = EngineConfig {
            cylinders: 3,
            firing_sequence: firing::INLINE_4_1342,
            injection_mode: InjectionMode::Sequential,
            ignition_mode: IgnitionMode::IndividualCoils,
            cranking_injection_mode: InjectionMode::Simultaneous,
            displacement_cc: 890,
            target_idle_rpm: 1_200,
        };
        assert_eq!(config.validate(), Err(ConfigError::FiringSequenceLength));
    }
}

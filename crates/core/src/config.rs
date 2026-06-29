//! Engine configuration persisted in flash / tuned at runtime.

/// Number of cylinders supported by the core engine model.
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

/// Standard 4-cylinder firing orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FiringOrder {
    /// 1-3-4-2 (common inline-4, e.g. Toyota/BMW/Miata).
    #[default]
    OneThreeFourTwo,
    /// 1-3-2-4 (common V4 / some fours).
    OneThreeTwoFour,
    /// 1-2-3-4 (some industrial / flat engines).
    OneTwoThreeFour,
    /// Rotax V990 60° V-twin: front (0) then rear (1), 300°/420° crank spacing.
    RotaxV990,
}

impl FiringOrder {
    /// Cylinder indices (0-based) in firing sequence.
    pub const fn sequence(self) -> &'static [u8] {
        match self {
            Self::OneThreeFourTwo => &[0, 2, 3, 1],
            Self::OneThreeTwoFour => &[0, 2, 1, 3],
            Self::OneTwoThreeFour => &[0, 1, 2, 3],
            Self::RotaxV990 => &[0, 1],
        }
    }
}

/// Top-level tunable engine configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineConfig {
    pub cylinders: u8,
    pub firing_order: FiringOrder,
    pub injection_mode: InjectionMode,
    pub ignition_mode: IgnitionMode,
    pub cranking_injection_mode: InjectionMode,
    /// Displacement in cubic centimeters (used for load modeling).
    pub displacement_cc: u16,
    /// Target idle RPM when closed-loop idle is active.
    pub target_idle_rpm: u16,
}

impl Default for EngineConfig {
    fn default() -> Self {
        crate::engines::rotax_v990::engine_config()
    }
}

impl EngineConfig {
    pub const fn cylinder_count(&self) -> usize {
        self.cylinders as usize
    }

    pub const fn validate(&self) -> Result<(), ConfigError> {
        if self.cylinders == 0 || self.cylinders as usize > MAX_CYLINDERS {
            return Err(ConfigError::InvalidCylinderCount);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidCylinderCount,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_rotax_v990() {
        let config = EngineConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.cylinders, 2);
        assert_eq!(config.displacement_cc, 998);
    }

    #[test]
    fn firing_order_sequence_respects_cylinder_count() {
        let order = FiringOrder::RotaxV990;
        assert_eq!(order.sequence().len(), 2);
    }
}

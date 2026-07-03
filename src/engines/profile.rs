//! Engine profile — tunable metadata independent of ECU board.

use crate::config::EngineConfig;
use crate::timing::TriggerSetup;

/// Full engine cycle in crank degrees for a four-stroke Otto cycle.
pub const CYCLE_DEGREES_FOUR_STROKE: u16 = 720;

/// Full engine cycle in crank degrees for a two-stroke engine.
pub const CYCLE_DEGREES_TWO_STROKE: u16 = 360;

/// Complete configuration for one engine type (factory specs + trigger pattern).
#[derive(Clone, Debug, PartialEq)]
pub struct EngineProfile {
    /// Human-readable name, e.g. `"Yamaha CP3"`.
    pub id: &'static str,
    pub engine: EngineConfig,
    pub trigger: TriggerSetup,
    /// Crank degrees for one full engine cycle (720° four-stroke, 360° two-stroke).
    pub cycle_degrees: u16,
    /// Crank degrees between consecutive power strokes; length must match cylinder count.
    pub fire_intervals_deg: &'static [u16],
    pub soft_rev_limit_rpm: u16,
    pub hard_rev_limit_rpm: u16,
    pub spark_plugs_per_cylinder: u8,
}

impl EngineProfile {
    pub fn validate(&self) -> Result<(), ProfileError> {
        self.engine.validate()?;

        if self.cycle_degrees == 0 {
            return Err(ProfileError::InvalidCycleDegrees);
        }

        if self.fire_intervals_deg.len() != self.engine.cylinders as usize {
            return Err(ProfileError::FireIntervalCount);
        }

        let sum: u16 = self.fire_intervals_deg.iter().sum();
        if sum != self.cycle_degrees {
            return Err(ProfileError::FireIntervalSum);
        }

        if self.spark_plugs_per_cylinder == 0 {
            return Err(ProfileError::InvalidSparkPlugCount);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    EngineConfig(crate::config::ConfigError),
    InvalidCycleDegrees,
    FireIntervalCount,
    FireIntervalSum,
    InvalidSparkPlugCount,
}

impl From<crate::config::ConfigError> for ProfileError {
    fn from(err: crate::config::ConfigError) -> Self {
        Self::EngineConfig(err)
    }
}

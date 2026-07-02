//! Engine profile — tunable metadata independent of ECU board.

use crate::config::EngineConfig;
use crate::timing::TriggerSetup;

/// Complete configuration for one engine type (factory specs + trigger pattern).
#[derive(Clone, Debug, PartialEq)]
pub struct EngineProfile {
    /// Human-readable name, e.g. `"Yamaha CP3"`.
    pub id: &'static str,
    pub engine: EngineConfig,
    pub trigger: TriggerSetup,
    /// Crank degrees between consecutive power strokes; length must match cylinder count.
    pub fire_intervals_deg: &'static [u16],
    pub soft_rev_limit_rpm: u16,
    pub hard_rev_limit_rpm: u16,
    pub spark_plugs_per_cylinder: u8,
}

impl EngineProfile {
    pub fn validate(&self) -> Result<(), ProfileError> {
        self.engine.validate()?;

        if self.fire_intervals_deg.len() != self.engine.cylinders as usize {
            return Err(ProfileError::FireIntervalCount);
        }

        let sum: u16 = self.fire_intervals_deg.iter().sum();
        if sum != 720 {
            return Err(ProfileError::FireIntervalSum);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    EngineConfig(crate::config::ConfigError),
    FireIntervalCount,
    FireIntervalSum,
}

impl From<crate::config::ConfigError> for ProfileError {
    fn from(err: crate::config::ConfigError) -> Self {
        Self::EngineConfig(err)
    }
}

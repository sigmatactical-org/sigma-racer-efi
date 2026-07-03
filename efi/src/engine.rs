//! Runtime engine state and fuel/ignition scheduling hooks.

use crate::config::EngineConfig;
use crate::timing::TriggerState;

/// Live engine runtime state updated by background tasks.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineState {
    pub trigger: TriggerState,
    /// Manifold absolute pressure in kPa (speed-density load input).
    pub map_kpa: f32,
    /// Coolant temperature in °C.
    pub clt_c: f32,
    /// Intake air temperature in °C.
    pub iat_c: f32,
    /// Battery voltage at connector (volts).
    pub vbatt: f32,
    /// Last computed base injection duration in milliseconds.
    pub base_injection_ms: f32,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            trigger: TriggerState::default(),
            map_kpa: 100.0,
            clt_c: 20.0,
            iat_c: 20.0,
            vbatt: 12.0,
            base_injection_ms: 0.0,
        }
    }
}

impl EngineState {
    /// Placeholder speed-density fuel pulse width.
    ///
    /// Real implementation will use VE tables, injector flow, and wall wetting
    /// modeled after rusEFI's fuel math, reimplemented here.
    pub fn compute_base_injection_ms(&mut self, config: &EngineConfig) -> f32 {
        if !self.trigger.synced || self.trigger.rpm < 100.0 {
            self.base_injection_ms = 3.0;
            return self.base_injection_ms;
        }

        let load = (self.map_kpa / 100.0).clamp(0.2, 1.5);
        let rpm_factor = (3_000.0 / self.trigger.rpm.max(500.0)).clamp(0.5, 3.0);
        let displacement_factor = config.displacement_cc as f32 / 1_600.0;

        self.base_injection_ms = 2.5 * load * rpm_factor * displacement_factor;
        self.base_injection_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cranking_injection_is_fixed() {
        use crate::engines::yamaha_cp3;

        let mut state = EngineState::default();
        let ms = state.compute_base_injection_ms(&yamaha_cp3::profile().engine);
        assert_eq!(ms, 3.0);
    }
}

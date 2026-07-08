//! Trigger sensor configuration and decoded engine phase.

use crate::trigger::TriggerWheel;

/// Sensor type wired to a trigger input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerInputKind {
    /// Variable reluctance crank/cam pickup.
    Vr,
    /// Hall-effect digital sensor.
    Hall,
}

/// Crank/cam decoder configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerSetup {
    pub crank_wheel: TriggerWheel,
    pub crank_input: TriggerInputKind,
    pub cam_input: TriggerInputKind,
    /// When true, fuel/ignition scheduling waits for cam sync before running sequential.
    pub cam_required: bool,
}

/// Engine phase derived from primary trigger edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TriggerState {
    pub rpm: f32,
    pub tooth_count: u32,
    /// 0.0–1.0 position within the current engine cycle (720° four-stroke).
    pub cycle_phase: f32,
    pub synced: bool,
}

impl Default for TriggerState {
    fn default() -> Self {
        Self {
            rpm: 0.0,
            tooth_count: 0,
            cycle_phase: 0.0,
            synced: false,
        }
    }
}

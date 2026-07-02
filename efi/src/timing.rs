//! Crank/cam trigger decoding and RPM estimation (stubs for early bring-up).

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

/// Wheel definition for crank decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TriggerWheel {
    pub teeth: u8,
    pub missing: u8,
}

impl TriggerWheel {
    pub const fn sixty_minus_two() -> Self {
        Self {
            teeth: 60,
            missing: 2,
        }
    }

    /// 12-1 wheel — common aftermarket pattern for high-RPM motorcycle ECU swaps.
    pub const fn twelve_minus_one() -> Self {
        Self {
            teeth: 12,
            missing: 1,
        }
    }

    pub const fn effective_edges_per_rev(self) -> u8 {
        self.teeth.saturating_sub(self.missing)
    }
}

/// Update RPM from period between consecutive trigger edges.
pub fn rpm_from_period_us(period_us: u32, teeth_per_rev: u8) -> f32 {
    if period_us == 0 || teeth_per_rev == 0 {
        return 0.0;
    }
    60_000_000.0 / (period_us as f32 * teeth_per_rev as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpm_from_period_at_idle() {
        // 1,200 RPM, 11 crank edges/rev (12-1): period ≈ 4,545 µs
        let rpm = rpm_from_period_us(4_545, 11);
        assert!((rpm - 1_200.0).abs() < 50.0);
    }
}

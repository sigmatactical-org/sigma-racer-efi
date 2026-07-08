//! Ride-by-wire monitor configuration and per-tick inputs.
//!
//! Sensor calibrations are ⚠ [MEASURE] — dual-sensor slopes and valid
//! windows come off the actual CP3 twistgrip and throttle body in mule
//! Phase 1 (items 8–9), never assumed.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorCal {
    /// Lowest electrically-plausible voltage (below ⇒ open circuit / sag).
    pub min_valid_v: f32,
    /// Highest electrically-plausible voltage (above ⇒ short to supply).
    pub max_valid_v: f32,
    /// Voltage at 0 %.
    pub v_at_0: f32,
    /// Voltage at 100 %.
    pub v_at_100: f32,
}

impl SensorCal {
    pub fn in_range(&self, volts: f32) -> bool {
        volts >= self.min_valid_v && volts <= self.max_valid_v
    }

    /// Normalize volts to percent (unclamped slope; caller range-checks
    /// first). Handles inverted slopes (v_at_100 < v_at_0) transparently,
    /// which is how mirrored second sensors normalize without special cases.
    pub fn to_pct(&self, volts: f32) -> f32 {
        let span = self.v_at_100 - self.v_at_0;
        if span == 0.0 {
            return 0.0;
        }
        (volts - self.v_at_0) / span * 100.0
    }
}

/// ⚠ [MEASURE] placeholder cal — a generic 0.5–4.5 V sensor. Replaced per
/// sensor by mule Phase-1 measurements.
pub const PLACEHOLDER_CAL: SensorCal = SensorCal {
    min_valid_v: 0.25,
    max_valid_v: 4.75,
    v_at_0: 0.5,
    v_at_100: 4.5,
};

/// Monitor configuration. Defaults carry ⚠ [MEASURE] placeholders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RbwConfig {
    pub app_a: SensorCal,
    pub app_b: SensorCal,
    pub tps_a: SensorCal,
    pub tps_b: SensorCal,
    /// Max |a − b| after normalization, percent points.
    pub pair_disagree_pct: f32,
    /// Disagreement must persist this long to trip (µs).
    pub pair_disagree_hold_us: u64,
    /// Max |commanded − actual| envelope, percent points.
    pub tracking_err_pct: f32,
    /// Tracking error must persist this long to trip (µs).
    pub tracking_hold_us: u64,
    /// Demand at or below this is "idle" for re-arm / start permit.
    pub idle_demand_pct: f32,
}

impl Default for RbwConfig {
    fn default() -> Self {
        Self {
            app_a: PLACEHOLDER_CAL,
            app_b: PLACEHOLDER_CAL,
            tps_a: PLACEHOLDER_CAL,
            tps_b: PLACEHOLDER_CAL,
            pair_disagree_pct: 5.0,
            pair_disagree_hold_us: 20_000,
            tracking_err_pct: 10.0,
            tracking_hold_us: 100_000,
            idle_demand_pct: 2.0,
        }
    }
}

/// One monitor tick's inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RbwInputs {
    pub t_us: u64,
    pub app_a_v: f32,
    pub app_b_v: f32,
    pub tps_a_v: f32,
    pub tps_b_v: f32,
    /// The controller's current plate target, percent.
    pub commanded_pct: f32,
}

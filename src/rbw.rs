//! Ride-by-wire independent safety monitor — `efi.md` §7, the crux.
//!
//! This module is the *monitor*, deliberately separate from any throttle
//! controller: it shares no code path with the loop it polices. Pure
//! `no_std` logic driven by a periodic tick (~1 kHz from the safety timer);
//! host tests run the full fault-injection matrix against it — the M3 gate
//! says every one of those tests passes before the motor ever drives a
//! real plate.
//!
//! Checks (each → latched fail-safe):
//! 1. **Range** — every APP/TPS signal inside its valid electrical window;
//!    open (≈0 V) and short (≈5 V) trip instantly.
//! 2. **Pair plausibility** — the two APP sensors must agree after
//!    normalization, likewise the two TPS sensors; disagreement must
//!    persist past a debounce window to trip (EMC glitch tolerance).
//! 3. **Tracking** — actual plate position must follow the commanded
//!    target within an error envelope and time window; stuck plate and
//!    runaway both land here.
//!
//! Fail-safe is **latched**: healthy inputs do not un-trip. Recovery is a
//! deliberate re-arm (zero demand + all checks healthy — the "key cycle or
//! zero-demand" rule), and the engine start permit is denied unless armed,
//! plausible, and demand is at idle.
//!
//! All calibration values are ⚠ [MEASURE] — dual-sensor slopes and valid
//! windows come off the actual CP3 twistgrip and throttle body in mule
//! Phase 1 (items 8–9), never assumed.

/// Linear calibration for one sensor: volts ↔ percent, plus the electrical
/// window outside which the signal is an open/short fault.
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

/// Why the monitor tripped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TripCause {
    AppOutOfRange(Side),
    TpsOutOfRange(Side),
    AppDisagreement,
    TpsDisagreement,
    /// Plate not following the command: stuck, runaway, or motor fault.
    Tracking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RbwState {
    Armed,
    Tripped(TripCause),
}

/// The monitor's verdict for this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RbwCommand {
    /// All checks healthy: rider demand, normalized percent.
    Normal { demand_pct: f32 },
    /// Cut the H-bridge → spring closes the plate → cut fuel. Latched.
    FailSafe,
}

#[derive(Debug)]
pub struct RbwMonitor {
    cfg: RbwConfig,
    state: RbwState,
    app_disagree_since: Option<u64>,
    tps_disagree_since: Option<u64>,
    tracking_err_since: Option<u64>,
}

impl RbwMonitor {
    pub fn new(cfg: RbwConfig) -> Self {
        Self {
            cfg,
            state: RbwState::Armed,
            app_disagree_since: None,
            tps_disagree_since: None,
            tracking_err_since: None,
        }
    }

    pub fn state(&self) -> RbwState {
        self.state
    }

    fn trip(&mut self, cause: TripCause) -> RbwCommand {
        self.state = RbwState::Tripped(cause);
        RbwCommand::FailSafe
    }

    /// Evaluate one tick. Call at the safety-task rate for every tick the
    /// system is live — including while tripped (the latch is here).
    pub fn evaluate(&mut self, inputs: &RbwInputs) -> RbwCommand {
        if matches!(self.state, RbwState::Tripped(_)) {
            return RbwCommand::FailSafe;
        }

        // 1 · Range — instant trips, no debounce: an open or short is not a
        // glitch, and waiting on one means trusting a broken sensor.
        if !self.cfg.app_a.in_range(inputs.app_a_v) {
            return self.trip(TripCause::AppOutOfRange(Side::A));
        }
        if !self.cfg.app_b.in_range(inputs.app_b_v) {
            return self.trip(TripCause::AppOutOfRange(Side::B));
        }
        if !self.cfg.tps_a.in_range(inputs.tps_a_v) {
            return self.trip(TripCause::TpsOutOfRange(Side::A));
        }
        if !self.cfg.tps_b.in_range(inputs.tps_b_v) {
            return self.trip(TripCause::TpsOutOfRange(Side::B));
        }

        let app_a = self.cfg.app_a.to_pct(inputs.app_a_v);
        let app_b = self.cfg.app_b.to_pct(inputs.app_b_v);
        let tps_a = self.cfg.tps_a.to_pct(inputs.tps_a_v);
        let tps_b = self.cfg.tps_b.to_pct(inputs.tps_b_v);

        // 2 · Pair plausibility, debounced.
        if exceeded(
            libm::fabsf(app_a - app_b) > self.cfg.pair_disagree_pct,
            &mut self.app_disagree_since,
            inputs.t_us,
            self.cfg.pair_disagree_hold_us,
        ) {
            return self.trip(TripCause::AppDisagreement);
        }
        if exceeded(
            libm::fabsf(tps_a - tps_b) > self.cfg.pair_disagree_pct,
            &mut self.tps_disagree_since,
            inputs.t_us,
            self.cfg.pair_disagree_hold_us,
        ) {
            return self.trip(TripCause::TpsDisagreement);
        }

        // 3 · Tracking envelope, debounced. Covers stuck plate (actual
        // frozen below command) and runaway (actual above command) alike.
        let actual = (tps_a + tps_b) / 2.0;
        if exceeded(
            libm::fabsf(inputs.commanded_pct - actual) > self.cfg.tracking_err_pct,
            &mut self.tracking_err_since,
            inputs.t_us,
            self.cfg.tracking_hold_us,
        ) {
            return self.trip(TripCause::Tracking);
        }

        RbwCommand::Normal {
            demand_pct: (app_a + app_b) / 2.0,
        }
    }

    /// Deliberate recovery: allowed only from a tripped state, with every
    /// sensor healthy and demand at idle (the zero-demand rule; a key cycle
    /// constructs a fresh monitor and is inherently a re-arm).
    pub fn rearm(&mut self, inputs: &RbwInputs) -> bool {
        if !matches!(self.state, RbwState::Tripped(_)) {
            return false;
        }
        if !self.inputs_healthy(inputs) {
            return false;
        }
        let demand = (self.cfg.app_a.to_pct(inputs.app_a_v)
            + self.cfg.app_b.to_pct(inputs.app_b_v))
            / 2.0;
        if demand > self.cfg.idle_demand_pct {
            return false;
        }
        self.state = RbwState::Armed;
        self.app_disagree_since = None;
        self.tps_disagree_since = None;
        self.tracking_err_since = None;
        true
    }

    /// Engine start permit (`efi.md` §7): armed, plausible, demand at idle.
    /// Kill/sidestand/clutch interlocks are the caller's to AND in.
    pub fn start_permit(&self, inputs: &RbwInputs) -> bool {
        if self.state != RbwState::Armed || !self.inputs_healthy(inputs) {
            return false;
        }
        let demand = (self.cfg.app_a.to_pct(inputs.app_a_v)
            + self.cfg.app_b.to_pct(inputs.app_b_v))
            / 2.0;
        demand <= self.cfg.idle_demand_pct
    }

    fn inputs_healthy(&self, inputs: &RbwInputs) -> bool {
        self.cfg.app_a.in_range(inputs.app_a_v)
            && self.cfg.app_b.in_range(inputs.app_b_v)
            && self.cfg.tps_a.in_range(inputs.tps_a_v)
            && self.cfg.tps_b.in_range(inputs.tps_b_v)
            && libm::fabsf(
                self.cfg.app_a.to_pct(inputs.app_a_v) - self.cfg.app_b.to_pct(inputs.app_b_v),
            ) <= self.cfg.pair_disagree_pct
            && libm::fabsf(
                self.cfg.tps_a.to_pct(inputs.tps_a_v) - self.cfg.tps_b.to_pct(inputs.tps_b_v),
            ) <= self.cfg.pair_disagree_pct
    }
}

/// Debounce helper: has `condition` held continuously for `hold_us`?
fn exceeded(condition: bool, since: &mut Option<u64>, t_us: u64, hold_us: u64) -> bool {
    if !condition {
        *since = None;
        return false;
    }
    let start = *since.get_or_insert(t_us);
    t_us.saturating_sub(start) >= hold_us
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Percent → volts through the placeholder cal.
    fn v(pct: f32) -> f32 {
        0.5 + pct / 100.0 * 4.0
    }

    /// Healthy inputs at a given demand/plate position.
    fn healthy(t_us: u64, demand_pct: f32, plate_pct: f32) -> RbwInputs {
        RbwInputs {
            t_us,
            app_a_v: v(demand_pct),
            app_b_v: v(demand_pct),
            tps_a_v: v(plate_pct),
            tps_b_v: v(plate_pct),
            commanded_pct: plate_pct,
        }
    }

    fn monitor() -> RbwMonitor {
        RbwMonitor::new(RbwConfig::default())
    }

    /// Run `n` 1 kHz ticks of the same inputs; return the last command.
    fn run_ticks(m: &mut RbwMonitor, base: RbwInputs, n: u32) -> RbwCommand {
        let mut last = RbwCommand::FailSafe;
        for i in 0..n {
            let inputs = RbwInputs {
                t_us: base.t_us + i as u64 * 1_000,
                ..base
            };
            last = m.evaluate(&inputs);
        }
        last
    }

    #[test]
    fn healthy_inputs_pass_demand_through() {
        let mut m = monitor();
        let cmd = run_ticks(&mut m, healthy(0, 30.0, 30.0), 10);
        match cmd {
            RbwCommand::Normal { demand_pct } => assert!((demand_pct - 30.0).abs() < 0.5),
            RbwCommand::FailSafe => panic!("healthy inputs tripped"),
        }
        assert_eq!(m.state(), RbwState::Armed);
    }

    // ---- Fault-injection matrix: every single-sensor electrical fault ----

    #[test]
    fn matrix_single_sensor_open_and_short_trips_instantly() {
        // (mutator, expected cause) for each sensor and each fault polarity.
        type Mutate = fn(&mut RbwInputs, f32);
        let cases: &[(Mutate, TripCause)] = &[
            (|i, f| i.app_a_v = f, TripCause::AppOutOfRange(Side::A)),
            (|i, f| i.app_b_v = f, TripCause::AppOutOfRange(Side::B)),
            (|i, f| i.tps_a_v = f, TripCause::TpsOutOfRange(Side::A)),
            (|i, f| i.tps_b_v = f, TripCause::TpsOutOfRange(Side::B)),
        ];
        for &fault_v in &[0.0_f32, 5.0] {
            for (mutate, cause) in cases {
                let mut m = monitor();
                let mut inputs = healthy(0, 20.0, 20.0);
                mutate(&mut inputs, fault_v);
                assert_eq!(
                    m.evaluate(&inputs),
                    RbwCommand::FailSafe,
                    "fault {fault_v} V should trip instantly"
                );
                assert_eq!(m.state(), RbwState::Tripped(*cause));
            }
        }
    }

    #[test]
    fn app_disagreement_trips_after_hold() {
        let mut m = monitor();
        let mut inputs = healthy(0, 30.0, 30.0);
        inputs.app_b_v = v(60.0);
        // 10 ms of disagreement (hold is 20 ms): armed, not yet tripped.
        run_ticks(&mut m, inputs, 10);
        assert_eq!(m.state(), RbwState::Armed);
        // Sustained past the hold window: tripped and latched.
        inputs.t_us = 10_000;
        run_ticks(&mut m, inputs, 30);
        assert_eq!(m.state(), RbwState::Tripped(TripCause::AppDisagreement));
    }

    #[test]
    fn tps_disagreement_trips_after_hold() {
        let mut m = monitor();
        let mut inputs = healthy(0, 30.0, 30.0);
        inputs.tps_b_v = v(55.0);
        run_ticks(&mut m, inputs, 30);
        assert_eq!(m.state(), RbwState::Tripped(TripCause::TpsDisagreement));
    }

    #[test]
    fn brief_disagreement_glitch_does_not_trip() {
        let mut m = monitor();
        // 5 ms of disagreement (hold is 20 ms), then healthy again.
        let mut bad = healthy(0, 30.0, 30.0);
        bad.app_b_v = v(60.0);
        run_ticks(&mut m, bad, 5);
        let cmd = run_ticks(&mut m, healthy(5_000, 30.0, 30.0), 50);
        assert!(matches!(cmd, RbwCommand::Normal { .. }));
        assert_eq!(m.state(), RbwState::Armed);
    }

    #[test]
    fn stuck_plate_trips_tracking() {
        let mut m = monitor();
        // Commanded 50 %, plate frozen at 10 %.
        let mut inputs = healthy(0, 50.0, 10.0);
        inputs.commanded_pct = 50.0;
        run_ticks(&mut m, inputs, 150);
        assert_eq!(m.state(), RbwState::Tripped(TripCause::Tracking));
    }

    #[test]
    fn runaway_plate_trips_tracking() {
        let mut m = monitor();
        // Commanded 10 %, plate at 80 % — the dangerous direction.
        let mut inputs = healthy(0, 10.0, 80.0);
        inputs.commanded_pct = 10.0;
        run_ticks(&mut m, inputs, 150);
        assert_eq!(m.state(), RbwState::Tripped(TripCause::Tracking));
    }

    #[test]
    fn transient_tracking_error_within_hold_is_tolerated() {
        let mut m = monitor();
        // 50 ms of lag (hold is 100 ms) — a plate mid-slew, not a fault.
        let mut lag = healthy(0, 40.0, 20.0);
        lag.commanded_pct = 40.0;
        run_ticks(&mut m, lag, 50);
        let cmd = run_ticks(&mut m, healthy(50_000, 40.0, 40.0), 10);
        assert!(matches!(cmd, RbwCommand::Normal { .. }));
    }

    #[test]
    fn supply_sag_on_app_pair_trips_range() {
        let mut m = monitor();
        // Both APP droop together: pair check would pass — range catches it.
        let mut inputs = healthy(0, 30.0, 30.0);
        inputs.app_a_v = 0.1;
        inputs.app_b_v = 0.1;
        assert_eq!(m.evaluate(&inputs), RbwCommand::FailSafe);
        assert_eq!(m.state(), RbwState::Tripped(TripCause::AppOutOfRange(Side::A)));
    }

    // ---- Latch, re-arm, start permit ----

    #[test]
    fn fail_safe_is_latched_despite_healthy_inputs() {
        let mut m = monitor();
        let mut bad = healthy(0, 20.0, 20.0);
        bad.app_a_v = 0.0;
        m.evaluate(&bad);
        // Healthy again — still fail-safe, forever, until re-armed.
        let cmd = run_ticks(&mut m, healthy(1_000, 20.0, 20.0), 500);
        assert_eq!(cmd, RbwCommand::FailSafe);
    }

    #[test]
    fn rearm_requires_zero_demand_and_healthy_sensors() {
        let mut m = monitor();
        let mut bad = healthy(0, 20.0, 20.0);
        bad.tps_b_v = 5.0;
        m.evaluate(&bad);

        // Throttle open: denied.
        assert!(!m.rearm(&healthy(10_000, 20.0, 0.0)));
        // Sensor still faulty: denied.
        let mut still_bad = healthy(11_000, 0.0, 0.0);
        still_bad.tps_b_v = 5.0;
        assert!(!m.rearm(&still_bad));
        // Healthy, closed throttle, zero demand: re-armed.
        assert!(m.rearm(&healthy(12_000, 0.0, 0.0)));
        assert_eq!(m.state(), RbwState::Armed);
        // And not re-armable while already armed.
        assert!(!m.rearm(&healthy(13_000, 0.0, 0.0)));
    }

    #[test]
    fn start_permit_rules() {
        let m = monitor();
        // Healthy, idle demand: permitted.
        assert!(m.start_permit(&healthy(0, 0.0, 0.0)));
        // Demand above idle: denied.
        assert!(!m.start_permit(&healthy(0, 10.0, 0.0)));
        // Implausible sensor: denied.
        let mut bad = healthy(0, 0.0, 0.0);
        bad.app_a_v = 5.0;
        assert!(!m.start_permit(&bad));
        // Tripped monitor: denied.
        let mut tripped = monitor();
        let mut fault = healthy(0, 0.0, 0.0);
        fault.tps_a_v = 0.0;
        tripped.evaluate(&fault);
        assert!(!tripped.start_permit(&healthy(1_000, 0.0, 0.0)));
    }

    #[test]
    fn mirrored_second_sensor_normalizes_transparently() {
        // APP B wired as a mirror (5 V − A): inverted cal, same percent.
        let cfg = RbwConfig {
            app_b: SensorCal {
                min_valid_v: 0.25,
                max_valid_v: 4.75,
                v_at_0: 4.5,
                v_at_100: 0.5,
            },
            ..RbwConfig::default()
        };
        let mut m = RbwMonitor::new(cfg);
        let inputs = RbwInputs {
            t_us: 0,
            app_a_v: v(30.0),
            app_b_v: 5.0 - v(30.0), // mirrored
            tps_a_v: v(30.0),
            tps_b_v: v(30.0),
            commanded_pct: 30.0,
        };
        let cmd = m.evaluate(&inputs);
        assert!(matches!(cmd, RbwCommand::Normal { .. }), "{cmd:?}");
    }
}

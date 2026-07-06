//! Stage-1 characterization data logging (mule runbook Phase 1).
//!
//! Before this firmware controls anything, it serves as a passive data
//! logger on the engine: sample the analog channels, timestamp crank/cam
//! trigger edges, and stream parseable records over defmt/RTT. The interval
//! statistics here are the tooth-pattern discovery tool — a missing-tooth
//! gap shows up as a period ratio of ~(1 + missing) between consecutive
//! edges, without assuming any wheel geometry up front.
//!
//! Everything in this module is host-testable `no_std` logic; the Embassy
//! tasks in `src/bin/tasks/` are thin wrappers around it.

use crate::analog::{ADC_VREF, ANALOG_INPUT_DIVIDER, CLT_NTC, IAT_NTC, VBATT_SCALING};

/// Full-scale ADC counts at 12-bit resolution.
pub const ADC_FULL_SCALE: f32 = 4095.0;

/// Convert raw 12-bit ADC counts to volts at the MCU pin.
pub fn counts_to_pin_volts(raw: u16) -> f32 {
    raw as f32 * ADC_VREF / ADC_FULL_SCALE
}

/// Pin volts from one sampling sweep, before any connector scaling.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RawSweep {
    pub vbatt: f32,
    pub clt: f32,
    pub iat: f32,
    pub tps_map: f32,
    pub an_volt1: f32,
    pub an_volt2: f32,
}

/// One scaled sensor frame, ready to log or publish.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensorFrame {
    /// Microseconds since boot.
    pub t_us: u64,
    /// Battery voltage at the connector.
    pub vbatt_v: f32,
    /// Coolant temperature (NTC beta model), °C.
    pub clt_c: f32,
    /// Intake air temperature, °C.
    pub iat_c: f32,
    /// TPS/MAP shared input (MRE default wiring), volts at connector.
    pub tps_map_v: f32,
    /// AN volt 1 (connector pin 27), volts at connector.
    pub an_volt1_v: f32,
    /// AN volt 2 (connector pin 26), volts at connector.
    pub an_volt2_v: f32,
}

impl SensorFrame {
    /// Scale one raw sweep using the MRE analog front-end constants.
    pub fn from_sweep(t_us: u64, sweep: RawSweep) -> Self {
        Self {
            t_us,
            vbatt_v: VBATT_SCALING.raw_to_volts(sweep.vbatt),
            clt_c: CLT_NTC.volts_to_celsius(sweep.clt, ADC_VREF),
            iat_c: IAT_NTC.volts_to_celsius(sweep.iat, ADC_VREF),
            tps_map_v: sweep.tps_map * ANALOG_INPUT_DIVIDER,
            an_volt1_v: sweep.an_volt1 * ANALOG_INPUT_DIVIDER,
            an_volt2_v: sweep.an_volt2 * ANALOG_INPUT_DIVIDER,
        }
    }
}

/// Which trigger line an edge came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerLine {
    Crank,
    Cam,
}

/// A timestamped trigger edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeEvent {
    pub t_us: u64,
    pub line: TriggerLine,
    pub rising: bool,
}

/// Interval statistics for one trigger line.
///
/// The gap ratio (this period ÷ previous period, ×100 fixed point) is the
/// Phase-1 signal: ~100 tooth-to-tooth at steady speed, ~200 entering a
/// single-missing-tooth gap (~300 for two missing), then back under ~50 on
/// the first tooth after the gap.
#[derive(Debug, Default)]
pub struct EdgeIntervals {
    count: u32,
    last_t_us: Option<u64>,
    last_period_us: Option<u32>,
}

/// Derived numbers for one recorded edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeReport {
    /// Total edges seen on this line, including this one.
    pub count: u32,
    /// Microseconds since the previous edge.
    pub period_us: u32,
    /// This period ÷ previous period, ×100. Zero when no previous period.
    pub gap_ratio_x100: u32,
}

impl EdgeIntervals {
    pub const fn new() -> Self {
        Self {
            count: 0,
            last_t_us: None,
            last_period_us: None,
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    /// Record an edge timestamp. Returns `None` for the very first edge on
    /// the line (no period exists yet).
    pub fn record(&mut self, t_us: u64) -> Option<EdgeReport> {
        self.count = self.count.wrapping_add(1);
        let prev_t = self.last_t_us.replace(t_us)?;
        let period_us = t_us.saturating_sub(prev_t).min(u32::MAX as u64) as u32;
        let gap_ratio_x100 = match self.last_period_us {
            Some(prev) if prev > 0 => {
                ((period_us as u64 * 100) / prev as u64).min(u32::MAX as u64) as u32
            }
            _ => 0,
        };
        self.last_period_us = Some(period_us);
        Some(EdgeReport {
            count: self.count,
            period_us,
            gap_ratio_x100,
        })
    }
}

/// Estimate crank RPM from one tooth period, given teeth per revolution.
pub fn rpm_from_tooth_period(period_us: u32, teeth_per_rev: u32) -> f32 {
    if period_us == 0 || teeth_per_rev == 0 {
        return 0.0;
    }
    60_000_000.0 / (period_us as f32 * teeth_per_rev as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_scale_to_pin_volts() {
        assert!((counts_to_pin_volts(0) - 0.0).abs() < 1e-6);
        assert!((counts_to_pin_volts(4095) - ADC_VREF).abs() < 1e-6);
        assert!((counts_to_pin_volts(2048) - ADC_VREF / 2.0).abs() < 0.002);
    }

    #[test]
    fn sensor_frame_scales_vbatt_to_connector_volts() {
        let frame = SensorFrame::from_sweep(
            42,
            RawSweep {
                vbatt: 1.457,
                ..Default::default()
            },
        );
        assert_eq!(frame.t_us, 42);
        assert!((frame.vbatt_v - 12.0).abs() < 0.1);
    }

    #[test]
    fn sensor_frame_applies_input_divider_to_volt_channels() {
        let frame = SensorFrame::from_sweep(
            0,
            RawSweep {
                tps_map: 1.0,
                an_volt1: 2.0,
                ..Default::default()
            },
        );
        assert!((frame.tps_map_v - ANALOG_INPUT_DIVIDER).abs() < 1e-6);
        assert!((frame.an_volt1_v - 2.0 * ANALOG_INPUT_DIVIDER).abs() < 1e-6);
    }

    #[test]
    fn first_edge_yields_no_report() {
        let mut intervals = EdgeIntervals::new();
        assert_eq!(intervals.record(1_000), None);
        assert_eq!(intervals.count(), 1);
    }

    #[test]
    fn uniform_edges_report_unity_gap_ratio() {
        let mut intervals = EdgeIntervals::new();
        intervals.record(0);
        intervals.record(1_000);
        let report = intervals.record(2_000).unwrap();
        assert_eq!(report.period_us, 1_000);
        assert_eq!(report.gap_ratio_x100, 100);
    }

    #[test]
    fn missing_tooth_gap_shows_double_then_half_ratio() {
        let mut intervals = EdgeIntervals::new();
        // Uniform teeth at 1 ms, then a 2 ms gap (one missing tooth),
        // then back to 1 ms.
        intervals.record(0);
        intervals.record(1_000);
        intervals.record(2_000);
        let gap = intervals.record(4_000).unwrap();
        assert_eq!(gap.period_us, 2_000);
        assert_eq!(gap.gap_ratio_x100, 200);
        let after = intervals.record(5_000).unwrap();
        assert_eq!(after.period_us, 1_000);
        assert_eq!(after.gap_ratio_x100, 50);
    }

    #[test]
    fn rpm_from_period_matches_wheel_math() {
        // 60 teeth at 1000 rpm → one tooth per millisecond.
        let rpm = rpm_from_tooth_period(1_000, 60);
        assert!((rpm - 1_000.0).abs() < 0.01);
        assert_eq!(rpm_from_tooth_period(0, 60), 0.0);
        assert_eq!(rpm_from_tooth_period(1_000, 0), 0.0);
    }
}

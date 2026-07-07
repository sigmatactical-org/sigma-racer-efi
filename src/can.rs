//! ECU side of the M7 safety-bus CAN contract.
//!
//! The dictionary (message IDs, DBC, codec) lives in the shared
//! `sigma-racer-wingman-m7-can` crate — the same crate the cockpit's
//! `vehicle-service` and M7 firmware consume, so the two domains cannot
//! disagree on scaling. This module owns only what the ECU contributes:
//! mapping engine state into [`M7Signals`] and the transmit schedule.
//!
//! Bus rules (`efi.md` §11): **classic CAN, ≤8-byte frames** — the MRE's
//! F7 has no FDCAN; the cockpit's FD controllers speak classic natively.
//! The ECU transmits its snapshot; fields owned by other nodes (ABS wheel
//! speed, odometer, fuel level) ride at their dictionary defaults until
//! those nodes provide them.

pub use sigma_racer_wingman_m7_can::{M7Signals, MESSAGE_IDS, PerformanceMode, parse};

use crate::engines::EngineProfile;

/// What the ECU knows and publishes each broadcast tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EcuSnapshot {
    pub rpm: f32,
    pub coolant_c: f32,
    /// Oil temperature — 0 until the added oil-temp sensor is wired
    /// (`engine.md` §3 extreme-cooling spec).
    pub oil_c: f32,
    pub battery_v: f32,
    /// Throttle plate position from the RbW monitor's normalized TPS pair.
    pub throttle_pct: f32,
    pub side_stand: bool,
    /// Latched fault count (RbW trips, sensor faults, decoder desyncs).
    pub dtc_count: u8,
}

/// Map the ECU's contribution onto the shared contract type.
///
/// `redline` derives from the profile's soft rev limit — the same limit the
/// rev limiter acts on, so the cockpit warning and the cut agree.
pub fn m7_signals(snap: &EcuSnapshot, profile: &EngineProfile) -> M7Signals {
    M7Signals {
        engine_rpm: clamp(snap.rpm, 0.0, 65_535.0),
        coolant_c: clamp(snap.coolant_c, -40.0, 215.0) as i16,
        oil_c: clamp(snap.oil_c, -40.0, 215.0) as i16,
        redline: snap.rpm >= profile.soft_rev_limit_rpm as f32,
        // Upper bounds sit safely inside the DBC maxima: the exact decimal
        // limits (102.3, 40.95) are not f32-representable and overshoot the
        // range check when widened to f64 inside the codec.
        throttle_pct: clamp(snap.throttle_pct, 0.0, 102.0),
        battery_v: clamp(snap.battery_v, 0.0, 40.9),
        side_stand: snap.side_stand,
        dtc_count: snap.dtc_count,
        ..M7Signals::default()
    }
}

/// NaN-safe clamp: sensor math can produce NaN (unplugged NTC) and a NaN
/// must encode as the range floor, not poison the frame.
fn clamp(value: f32, lo: f32, hi: f32) -> f32 {
    if value.is_nan() {
        return lo;
    }
    value.clamp(lo, hi)
}

/// Transmit schedule, Hz per message ID — fast for rider-critical, slow for
/// bookkeeping. The broadcast task divides its base tick by these.
pub const TX_RATE_HZ: [(u32, u16); 5] = [
    (sigma_racer_wingman_m7_can::ENGINE_STATUS, 50),
    (sigma_racer_wingman_m7_can::THROTTLE_GEAR, 50),
    (sigma_racer_wingman_m7_can::WHEEL_SPEED, 20),
    (sigma_racer_wingman_m7_can::CHASSIS_ELECTRICAL, 10),
    (sigma_racer_wingman_m7_can::TRIP_ODOMETER, 1),
];

#[cfg(test)]
mod tests {
    use super::*;
    use sigma_racer_wingman_m7_can::{decode_into, encode_frames};

    fn profile() -> EngineProfile {
        crate::engines::yamaha_cp3::profile()
    }

    fn snapshot() -> EcuSnapshot {
        EcuSnapshot {
            rpm: 7_450.0,
            coolant_c: 84.4,
            oil_c: 96.0,
            battery_v: 13.87,
            throttle_pct: 42.3,
            side_stand: true,
            dtc_count: 2,
        }
    }

    #[test]
    fn round_trip_through_the_shared_dictionary() {
        let dbc = parse().unwrap();
        let signals = m7_signals(&snapshot(), &profile());
        let frames = encode_frames(&dbc, &signals).unwrap();
        assert_eq!(frames.len(), 5);

        let mut decoded = M7Signals::default();
        for (id, data) in &frames {
            assert!(decode_into(&dbc, *id, data, &mut decoded), "id {id:#x}");
        }

        assert_eq!(decoded.engine_rpm, 7_450.0);
        assert_eq!(decoded.coolant_c, 84);
        assert_eq!(decoded.oil_c, 96);
        assert!(!decoded.redline);
        assert!((decoded.throttle_pct - 42.3).abs() < 0.1);
        assert!((decoded.battery_v - 13.87).abs() < 0.01);
        assert!(decoded.side_stand);
        assert_eq!(decoded.dtc_count, 2);
    }

    #[test]
    fn redline_flag_matches_the_soft_rev_limit() {
        let profile = profile();
        let mut snap = snapshot();
        snap.rpm = profile.soft_rev_limit_rpm as f32 - 1.0;
        assert!(!m7_signals(&snap, &profile).redline);
        snap.rpm = profile.soft_rev_limit_rpm as f32;
        assert!(m7_signals(&snap, &profile).redline);
    }

    #[test]
    fn out_of_range_and_nan_inputs_still_encode() {
        let dbc = parse().unwrap();
        let snap = EcuSnapshot {
            rpm: 99_999.0,
            coolant_c: f32::NAN, // unplugged NTC reads NaN
            oil_c: 400.0,
            battery_v: -3.0,
            throttle_pct: 250.0,
            side_stand: false,
            dtc_count: 255,
        };
        let signals = m7_signals(&snap, &profile());
        // Must encode without range errors — clamped, not rejected.
        let frames = encode_frames(&dbc, &signals).unwrap();

        let mut decoded = M7Signals::default();
        for (id, data) in &frames {
            decode_into(&dbc, *id, data, &mut decoded);
        }
        assert_eq!(decoded.engine_rpm, 65_535.0);
        assert_eq!(decoded.coolant_c, -40, "NaN clamps to range floor");
        assert_eq!(decoded.oil_c, 215);
        assert_eq!(decoded.battery_v, 0.0);
        assert!(decoded.throttle_pct <= 102.3);
    }

    #[test]
    fn tx_schedule_covers_every_dictionary_message() {
        for id in MESSAGE_IDS {
            assert!(
                TX_RATE_HZ.iter().any(|(mid, hz)| *mid == id && *hz > 0),
                "message {id:#x} missing from the tx schedule"
            );
        }
    }
}

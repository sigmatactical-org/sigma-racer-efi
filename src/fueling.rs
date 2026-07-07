//! Speed-density fueling and coil dwell math (`efi.md` §5–6).
//!
//! `air = f(MAP, IAT, RPM, VE)`, `fuel = air / AFR_target`, corrected and
//! turned into an injector pulse width via the injector model. Pure
//! `no_std`; every calibration constant is ⚠ [MEASURE] — the shapes here
//! exist so the pipeline is testable, the dyno owns the numbers.

use crate::tables::{Curve, Table};

/// Ideal-gas constant for dry air, J/(kg·K).
const R_AIR: f32 = 287.05;

/// Cylinder air mass per intake event, milligrams — ideal gas at manifold
/// conditions scaled by volumetric efficiency.
///
/// `displacement_cc` is **per cylinder**; `ve_pct` from the VE table;
/// `map_kpa` absolute; `iat_c` charge temperature.
pub fn cylinder_air_mass_mg(displacement_cc: f32, ve_pct: f32, map_kpa: f32, iat_c: f32) -> f32 {
    let t_kelvin = iat_c + 273.15;
    if t_kelvin <= 0.0 || map_kpa <= 0.0 {
        return 0.0;
    }
    // ρ [kg/m³] = P / (R·T); volume in m³ = cc × 1e-6; mass mg = kg × 1e6.
    let density = (map_kpa * 1_000.0) / (R_AIR * t_kelvin);
    density * displacement_cc * (ve_pct / 100.0)
}

/// Fuel mass for one injection event, milligrams.
pub fn fuel_mass_mg(air_mass_mg: f32, afr_target: f32) -> f32 {
    if afr_target <= 0.0 {
        return 0.0;
    }
    air_mass_mg / afr_target
}

/// Injector model: flow slope + battery-dependent dead time.
///
/// ⚠ [MEASURE] per injector on the bench (runbook Phase 1, item 9 measures
/// resistance; flow slope and dead-time curve come from injector data or
/// the flow bench).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InjectorModel {
    /// Static flow, mg of fuel per millisecond of open time.
    pub flow_mg_per_ms: f32,
    /// Dead time vs battery voltage, ms (opening minus closing latency).
    pub dead_time_ms: Curve<4>,
}

/// Placeholder CP3-class injector: ~10 g/min-ish four-hole, generic
/// dead-time shape. ⚠ [MEASURE].
pub const PLACEHOLDER_INJECTOR: InjectorModel = InjectorModel {
    flow_mg_per_ms: 3.5,
    dead_time_ms: Curve {
        x: [8.0, 11.0, 13.5, 16.0],
        y: [1.8, 1.2, 0.9, 0.7],
    },
};

impl InjectorModel {
    /// Pulse width for a fuel mass at a battery voltage, ms.
    ///
    /// Zero fuel commands zero pulse — dead time is only added to real
    /// deliveries (an injector commanded "dead time only" still clicks and
    /// wears; never emit it as an idle artifact).
    pub fn pulse_width_ms(&self, fuel_mg: f32, vbatt: f32) -> f32 {
        if fuel_mg <= 0.0 || self.flow_mg_per_ms <= 0.0 {
            return 0.0;
        }
        fuel_mg / self.flow_mg_per_ms + self.dead_time_ms.lookup(vbatt)
    }
}

/// Coil dwell vs battery voltage with an over-dwell ceiling.
///
/// ⚠ [MEASURE] against the CP3 coils (smart-vs-dumb determination comes
/// first — runbook Phase 1, item 9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DwellModel {
    pub dwell_ms: Curve<4>,
    /// Hard ceiling, ms — over-dwell protection regardless of the curve.
    pub max_dwell_ms: f32,
}

pub const PLACEHOLDER_DWELL: DwellModel = DwellModel {
    dwell_ms: Curve {
        x: [8.0, 11.0, 13.5, 16.0],
        y: [6.0, 4.2, 3.2, 2.6],
    },
    max_dwell_ms: 8.0,
};

impl DwellModel {
    pub fn dwell_ms(&self, vbatt: f32) -> f32 {
        let d = self.dwell_ms.lookup(vbatt);
        if d > self.max_dwell_ms {
            self.max_dwell_ms
        } else {
            d
        }
    }
}

/// ⚠ [MEASURE] placeholder VE shape — flat 80 % with a mild torque hump.
/// Exists so the pipeline runs end to end; the dyno replaces it (M6).
pub const PLACEHOLDER_VE: Table<4, 4> = Table {
    row_axis: [1_000.0, 4_000.0, 7_000.0, 10_500.0],
    col_axis: [30.0, 60.0, 85.0, 100.0],
    values: [
        [55.0, 65.0, 72.0, 75.0],
        [60.0, 75.0, 85.0, 88.0],
        [62.0, 78.0, 90.0, 93.0],
        [58.0, 72.0, 84.0, 87.0],
    ],
};

/// One end-to-end base pulse computation: VE lookup → air → fuel → pulse.
pub fn base_pulse_ms(
    ve: &Table<4, 4>,
    injector: &InjectorModel,
    rpm: f32,
    map_kpa: f32,
    iat_c: f32,
    displacement_per_cyl_cc: f32,
    afr_target: f32,
    vbatt: f32,
) -> f32 {
    let ve_pct = ve.lookup(rpm, map_kpa);
    let air = cylinder_air_mass_mg(displacement_per_cyl_cc, ve_pct, map_kpa, iat_c);
    injector.pulse_width_ms(fuel_mass_mg(air, afr_target), vbatt)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CP3: 890 cc / 3 cylinders.
    const CYL_CC: f32 = 890.0 / 3.0;

    #[test]
    fn air_mass_matches_ideal_gas_at_standard_conditions() {
        // 100 kPa, 20 °C, VE 100 %: ρ ≈ 1.189 kg/m³ → ~352 mg per 296.7 cc.
        let mg = cylinder_air_mass_mg(CYL_CC, 100.0, 100.0, 20.0);
        assert!((mg - 352.0).abs() < 5.0, "air mass {mg} mg");
    }

    #[test]
    fn air_mass_scales_with_map_and_inverse_temperature() {
        let base = cylinder_air_mass_mg(CYL_CC, 100.0, 100.0, 20.0);
        assert!((cylinder_air_mass_mg(CYL_CC, 100.0, 50.0, 20.0) - base / 2.0).abs() < 0.5);
        // Hotter charge = less air.
        assert!(cylinder_air_mass_mg(CYL_CC, 100.0, 100.0, 60.0) < base);
        assert_eq!(cylinder_air_mass_mg(CYL_CC, 100.0, 0.0, 20.0), 0.0);
    }

    #[test]
    fn stoich_fuel_for_standard_cylinder_fill_is_plausible() {
        // ~352 mg air / 14.7 ≈ 24 mg fuel ≈ 6.8 ms at 3.5 mg/ms + dead time.
        let air = cylinder_air_mass_mg(CYL_CC, 100.0, 100.0, 20.0);
        let fuel = fuel_mass_mg(air, 14.7);
        assert!((fuel - 24.0).abs() < 1.0, "fuel {fuel} mg");
        let pw = PLACEHOLDER_INJECTOR.pulse_width_ms(fuel, 13.5);
        assert!(pw > 6.0 && pw < 9.0, "pulse {pw} ms");
    }

    #[test]
    fn zero_fuel_commands_zero_pulse_not_dead_time() {
        assert_eq!(PLACEHOLDER_INJECTOR.pulse_width_ms(0.0, 13.5), 0.0);
    }

    #[test]
    fn dead_time_grows_as_battery_sags() {
        let sagged = PLACEHOLDER_INJECTOR.pulse_width_ms(24.0, 9.0);
        let healthy = PLACEHOLDER_INJECTOR.pulse_width_ms(24.0, 14.0);
        assert!(sagged > healthy);
    }

    #[test]
    fn dwell_clamps_at_overdwell_ceiling() {
        // At 8 V the curve asks 6 ms — under the 8 ms ceiling.
        assert!(PLACEHOLDER_DWELL.dwell_ms(8.0) <= PLACEHOLDER_DWELL.max_dwell_ms);
        // A pathological curve request is clamped.
        let hot = DwellModel {
            max_dwell_ms: 3.0,
            ..PLACEHOLDER_DWELL
        };
        assert_eq!(hot.dwell_ms(8.0), 3.0);
    }

    #[test]
    fn base_pulse_end_to_end_moves_with_load() {
        let idle = base_pulse_ms(
            &PLACEHOLDER_VE,
            &PLACEHOLDER_INJECTOR,
            1_200.0,
            35.0,
            30.0,
            CYL_CC,
            14.7,
            13.5,
        );
        let wot = base_pulse_ms(
            &PLACEHOLDER_VE,
            &PLACEHOLDER_INJECTOR,
            7_000.0,
            100.0,
            30.0,
            CYL_CC,
            13.0,
            13.5,
        );
        assert!(idle > 0.5 && idle < wot, "idle {idle} ms, wot {wot} ms");
        assert!(wot < 12.0, "wot {wot} ms");
    }
}

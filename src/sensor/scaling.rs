//! Analog scaling helpers: linear front-end scaling and the NTC thermistor
//! model.

/// Analog front-end scaling applied after ADC conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalogScaling {
    /// Multiply ADC pin volts by this factor to get connector volts.
    pub multiplier: f32,
    /// Fixed bias added after scaling (volts at connector).
    pub offset: f32,
}

impl AnalogScaling {
    pub const fn raw_to_volts(&self, adc_volts: f32) -> f32 {
        adc_volts * self.multiplier + self.offset
    }
}

/// NTC thermistor configuration (bias resistor to 3.3 V).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NtcConfig {
    pub bias_resistor_ohms: f32,
    pub beta: f32,
    pub resistance_at_25c: f32,
    /// ⚠ [MEASURE] — voltage the bias resistor pulls the divider to. The
    /// rusEFI MRE wiring doc says the AT-input pull-up goes to **5 V**, not
    /// ADC vref (3.3 V). Settled empirically by the wiring doc's Session-0
    /// warmup-cycle fit / 2.5 kΩ resistor check.
    pub bias_supply_volts: f32,
    /// ⚠ [MEASURE] — ratio from ADC-pin volts back to divider-node volts
    /// (1.0 if the AT path bypasses the board's 1.68 input divider,
    /// 1.68 if it doesn't). Same Session-0 check settles it.
    pub input_divider: f32,
}

impl Default for NtcConfig {
    fn default() -> Self {
        Self {
            bias_resistor_ohms: 2_700.0,
            beta: 3_500.0,
            resistance_at_25c: 2_500.0,
            bias_supply_volts: 5.0,
            input_divider: 1.0,
        }
    }
}

impl NtcConfig {
    /// Convert ADC-pin voltage to Celsius via the beta equation.
    ///
    /// Reconstructs the divider-node voltage from pin volts using
    /// `input_divider`, then solves the bias divider against
    /// `bias_supply_volts`. Sufficient for ECU sensor modeling.
    pub fn volts_to_celsius(&self, adc_volts: f32) -> f32 {
        let node_volts = adc_volts * self.input_divider;
        if node_volts <= 0.0 || node_volts >= self.bias_supply_volts {
            return f32::NAN;
        }
        let resistance =
            self.bias_resistor_ohms * node_volts / (self.bias_supply_volts - node_volts);
        let t0 = 25.0 + 273.15;
        let inv_t: f32 = 1.0 / t0 + libm::logf(resistance / self.resistance_at_25c) / self.beta;
        1.0 / inv_t - 273.15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ntc_at_bias_midpoint_is_near_room_temp() {
        // Node at half the bias supply ⇒ thermistor resistance equals the
        // bias resistor. Default config has R25 ≠ Rbias, so expect the
        // temperature where R(T) = 2.7 kΩ — just below 25 °C.
        let ntc = NtcConfig::default();
        let c = ntc.volts_to_celsius(2.5);
        assert!((c - 25.0).abs() < 5.0);
    }

    #[test]
    fn input_divider_reconstructs_node_volts() {
        let direct = NtcConfig::default();
        let divided = NtcConfig {
            input_divider: 1.68,
            ..NtcConfig::default()
        };
        // Same node voltage seen through the 1.68 divider at the pin.
        let c_direct = direct.volts_to_celsius(2.5);
        let c_divided = divided.volts_to_celsius(2.5 / 1.68);
        assert!((c_direct - c_divided).abs() < 0.1);
    }

    #[test]
    fn out_of_range_pin_volts_yield_nan() {
        let ntc = NtcConfig::default();
        assert!(ntc.volts_to_celsius(0.0).is_nan());
        assert!(ntc.volts_to_celsius(5.1).is_nan());
    }
}

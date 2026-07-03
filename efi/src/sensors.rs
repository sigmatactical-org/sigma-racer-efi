//! Sensor channel identifiers and analog scaling helpers.

/// Logical ADC channel on the ECU connector (maps to MCU ADC inputs per board).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdcChannel {
    /// Connector pin 1 — battery sense (after divider).
    Battery,
    /// AN volt 1 — connector pin 27.
    AnalogVolt1,
    /// AN volt 2 — connector pin 26.
    AnalogVolt2,
    /// AN volt 3 — connector pin 31.
    AnalogVolt3,
    /// AN volt 4 — connector pin 19.
    AnalogVolt4,
    /// AN volt 5 — connector pin 20.
    AnalogVolt5,
    /// AN volt 6 — connector pin 32 (often wideband).
    AnalogVolt6,
    /// AN volt 7 — connector pin 30.
    AnalogVolt7,
    /// AN temp 1 — connector pin 18 (CLT default on MRE).
    CoolantTemp,
    /// AN temp 2 — connector pin 23 (IAT default on MRE).
    IntakeTemp,
    /// AN temp 3 — connector pin 24.
    AuxTemp3,
    /// AN temp 4 — connector pin 22.
    AuxTemp4,
    /// TPS — connector pin 28 (MAP default on MRE; TPS optional).
    Tps,
    /// MAP — shares TPS pin on default MRE wiring.
    Map,
}

/// Analog front-end scaling applied after ADC conversion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalogScaling {
    /// Volts at the ADC pin per volt at the connector (divider ratio).
    pub divider: f32,
    /// Fixed bias subtracted after scaling (volts at connector).
    pub offset: f32,
}

impl AnalogScaling {
    pub const fn raw_to_volts(&self, adc_volts: f32) -> f32 {
        adc_volts / self.divider + self.offset
    }
}

/// NTC thermistor configuration (bias resistor to 3.3 V).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NtcConfig {
    pub bias_resistor_ohms: f32,
    pub beta: f32,
    pub resistance_at_25c: f32,
}

impl Default for NtcConfig {
    fn default() -> Self {
        Self {
            bias_resistor_ohms: 2_700.0,
            beta: 3_500.0,
            resistance_at_25c: 2_500.0,
        }
    }
}

impl NtcConfig {
    /// Convert divider voltage (at ADC pin) to Celsius.
    ///
    /// Uses the beta equation; sufficient for ECU sensor modeling.
    pub fn volts_to_celsius(&self, adc_volts: f32, vref: f32) -> f32 {
        if adc_volts <= 0.0 || adc_volts >= vref {
            return f32::NAN;
        }
        let resistance = self.bias_resistor_ohms * adc_volts / (vref - adc_volts);
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
        let ntc = NtcConfig::default();
        let c = ntc.volts_to_celsius(1.65, 3.3);
        assert!((c - 25.0).abs() < 5.0);
    }
}

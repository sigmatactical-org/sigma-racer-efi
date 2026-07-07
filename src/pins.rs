//! STM32 GPIO and peripheral assignments for microRusEFI (rev 0.6.x wiring).
//!
//! Injectors and several outputs route through the onboard TLE8888 smart driver;
//! logical connector names are preserved as documentation on each field.

/// MCU pin names for the microRusEFI PCB (STM32F767).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardPins {
    // --- Status LEDs (on PCB) ---
    pub led_comms: GpioPin,
    pub led_running: GpioPin,
    pub led_warning: GpioPin,
    pub led_critical: GpioPin,

    // --- Ignition (TC4427, 5 V logic) ---
    pub ignition: [GpioPin; 4],

    // --- Trigger inputs ---
    /// Primary crank — VR conditioner via TLE8888 (pin 45).
    pub trigger_crank: GpioPin,
    /// Cam hall input (pin 25).
    pub trigger_cam: GpioPin,

    // --- TLE8888 (injectors + low-side outputs) SPI1 ---
    pub tle8888_spi_mosi: GpioPin,
    pub tle8888_spi_miso: GpioPin,
    pub tle8888_spi_sck: GpioPin,
    pub tle8888_spi_cs: GpioPin,
    /// Active-high enable for TLE8888 injector outputs (PD11 + pulldown).
    pub tle8888_inj_en: GpioPin,
    /// Active-high enable for TLE8888 ignition outputs (PD10 + pulldown).
    pub tle8888_ign_en: GpioPin,

    // --- Electronic throttle (TLE9201) ---
    pub etb_pwm: GpioPin,
    pub etb_dir_a: GpioPin,
    /// TLE9201 DIS — low disables the H-bridge (rusEFI `setupTLE9201` disable pin).
    pub etb_disable: GpioPin,

    // --- CAN ---
    pub can_tx: GpioPin,
    pub can_rx: GpioPin,

    // --- Onboard microSD (SPI2, v0.6.0+) ---
    pub sd_spi_mosi: GpioPin,
    pub sd_spi_miso: GpioPin,
    pub sd_spi_sck: GpioPin,
    pub sd_spi_cs: GpioPin,

    // --- Expansion SPI3 (header, disabled by default in rusEFI) ---
    pub exp_spi_mosi: GpioPin,
    pub exp_spi_miso: GpioPin,
    pub exp_spi_sck: GpioPin,

    // --- Console UART (USART3 on J12/J13 — conflicts with SD DMA in rusEFI) ---
    pub console_tx: GpioPin,
    pub console_rx: GpioPin,
}

/// STM32 port/pin identifier (e.g. `PD4` → port D, pin 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GpioPin {
    pub port: GpioPort,
    pub pin: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GpioPort {
    A,
    B,
    C,
    D,
    E,
}

impl GpioPin {
    pub const fn new(port: GpioPort, pin: u8) -> Self {
        Self { port, pin }
    }
}

impl BoardPins {
    /// Pin map matching rusEFI `board_configuration.cpp` defaults for MRE F7.
    pub const fn mre_f7() -> Self {
        Self {
            led_comms: GpioPin::new(GpioPort::E, 2),
            led_running: GpioPin::new(GpioPort::E, 4),
            led_warning: GpioPin::new(GpioPort::E, 1),
            led_critical: GpioPin::new(GpioPort::E, 3),

            ignition: [
                GpioPin::new(GpioPort::D, 4), // pin 9  — INJ/IGN 1
                GpioPin::new(GpioPort::D, 3), // pin 10
                GpioPin::new(GpioPort::D, 2), // pin 11
                GpioPin::new(GpioPort::D, 1), // pin 12
            ],

            trigger_crank: GpioPin::new(GpioPort::C, 6), // pin 45 VR/Hall
            trigger_cam: GpioPin::new(GpioPort::A, 5),   // pin 25 Hall cam

            tle8888_spi_mosi: GpioPin::new(GpioPort::B, 5),
            tle8888_spi_miso: GpioPin::new(GpioPort::B, 4),
            tle8888_spi_sck: GpioPin::new(GpioPort::B, 3),
            tle8888_spi_cs: GpioPin::new(GpioPort::D, 5),
            tle8888_inj_en: GpioPin::new(GpioPort::D, 11),
            tle8888_ign_en: GpioPin::new(GpioPort::D, 10),

            etb_pwm: GpioPin::new(GpioPort::C, 7),
            etb_dir_a: GpioPin::new(GpioPort::A, 8),
            etb_disable: GpioPin::new(GpioPort::C, 8),

            can_tx: GpioPin::new(GpioPort::B, 6),
            can_rx: GpioPin::new(GpioPort::B, 12),

            sd_spi_mosi: GpioPin::new(GpioPort::B, 15),
            sd_spi_miso: GpioPin::new(GpioPort::B, 14),
            sd_spi_sck: GpioPin::new(GpioPort::B, 13),
            sd_spi_cs: GpioPin::new(GpioPort::E, 15),

            exp_spi_mosi: GpioPin::new(GpioPort::C, 12),
            exp_spi_miso: GpioPin::new(GpioPort::C, 11),
            exp_spi_sck: GpioPin::new(GpioPort::C, 10),

            console_tx: GpioPin::new(GpioPort::B, 10),
            console_rx: GpioPin::new(GpioPort::B, 11),
        }
    }
}

/// TLE8888-driven outputs (injectors, GP outs, low sides) by connector name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TleOutput {
    Injector1,
    Injector2,
    Injector3,
    Injector4,
    GpOut1,
    GpOut2,
    GpOut3,
    GpOut4,
    LowSide1,
    LowSide2,
}

/// Default rusEFI output assignment on microRusEFI.
impl TleOutput {
    pub const fn default_function(self) -> &'static str {
        match self {
            Self::Injector1 => "Injector 1 (pin 37)",
            Self::Injector2 => "Injector 2 (pin 38)",
            Self::Injector3 => "Injector 3 (pin 41)",
            Self::Injector4 => "Injector 4 (pin 42)",
            Self::GpOut1 => "Fuel pump (pin 35)",
            Self::GpOut2 => "Radiator fan (pin 34)",
            Self::GpOut3 => "General purpose (pin 33)",
            Self::GpOut4 => "General purpose (pin 43)",
            Self::LowSide1 => "VVT / high-current solenoid (pin 7)",
            Self::LowSide2 => "Idle IAC solenoid (pin 3)",
        }
    }
}

#[cfg(feature = "firmware")]
pub mod embassy;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignition_pins_match_rusefi_board_config() {
        let pins = BoardPins::mre_f7();
        assert_eq!(pins.ignition[0], GpioPin::new(GpioPort::D, 4));
        assert_eq!(pins.ignition[3], GpioPin::new(GpioPort::D, 1));
    }

    #[test]
    fn tle8888_enables_match_mre_wiring() {
        let pins = BoardPins::mre_f7();
        assert_eq!(pins.tle8888_inj_en, GpioPin::new(GpioPort::D, 11));
        assert_eq!(pins.tle8888_ign_en, GpioPin::new(GpioPort::D, 10));
    }

    #[test]
    fn etb_disable_is_tle9201_dis_pin() {
        let pins = BoardPins::mre_f7();
        assert_eq!(pins.etb_disable, GpioPin::new(GpioPort::C, 8));
    }
}

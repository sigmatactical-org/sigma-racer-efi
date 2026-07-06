//! Bind abstract [`BoardPins`] to Embassy STM32 peripherals for the MRE F7.
//!
//! [`MreBoard::init`] is the single place `Peripherals` is split, so every
//! pin claim is visible in one function and double-claims cannot compile.

use crate::pins::BoardPins;
use embassy_stm32::adc::{Adc, AdcChannel as _, AnyAdcChannel};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::{Peripherals, bind_interrupts, exti, interrupt};

bind_interrupts!(struct Irqs {
    EXTI9_5 => exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
});

/// ADC channels for the stage-1 sensor sweep (MRE default wiring).
///
/// MCU pins follow the rusEFI `EFI_ADC_x` map in [`crate::analog`]:
/// battery = ADC11/PC1, CLT = ADC0/PA0, IAT = ADC1/PA1, TPS/MAP = ADC4/PA4,
/// AN volt 1 = ADC10/PC0, AN volt 2 = ADC6/PA6.
pub struct SensorChannels {
    pub vbatt: AnyAdcChannel<'static, ADC1>,
    pub clt: AnyAdcChannel<'static, ADC1>,
    pub iat: AnyAdcChannel<'static, ADC1>,
    pub tps_map: AnyAdcChannel<'static, ADC1>,
    pub an_volt1: AnyAdcChannel<'static, ADC1>,
    pub an_volt2: AnyAdcChannel<'static, ADC1>,
}

/// The microRusEFI F7 board, split into owned Embassy devices.
pub struct MreBoard {
    pub led_comms: Output<'static>,
    pub led_running: Output<'static>,
    pub led_warning: Output<'static>,
    pub led_critical: Output<'static>,
    pub adc: Adc<'static, ADC1>,
    pub sensors: SensorChannels,
    /// Primary crank trigger (pin 45, VR conditioner output) — PC6/EXTI6.
    pub crank: ExtiInput<'static, Async>,
    /// Cam hall input (pin 25) — PA5/EXTI5.
    pub cam: ExtiInput<'static, Async>,
}

impl MreBoard {
    /// Split `Peripherals` into the MRE F7 board devices.
    ///
    /// Verifies `pins` matches the compiled-in MRE map: this binder only
    /// implements the rev 0.6.x wiring, and a mismatched map means the
    /// caller expects different hardware than this function claims.
    pub fn init(pins: BoardPins, p: Peripherals) -> Self {
        if pins != BoardPins::mre_f7() {
            defmt::panic!("BoardPins do not match the MRE F7 map");
        }

        Self {
            led_comms: Output::new(p.PE2, Level::Low, Speed::Low),
            led_running: Output::new(p.PE4, Level::Low, Speed::Low),
            led_warning: Output::new(p.PE1, Level::Low, Speed::Low),
            led_critical: Output::new(p.PE3, Level::Low, Speed::Low),
            adc: Adc::new(p.ADC1),
            sensors: SensorChannels {
                vbatt: p.PC1.degrade_adc(),
                clt: p.PA0.degrade_adc(),
                iat: p.PA1.degrade_adc(),
                tps_map: p.PA4.degrade_adc(),
                an_volt1: p.PC0.degrade_adc(),
                an_volt2: p.PA6.degrade_adc(),
            },
            crank: ExtiInput::new(p.PC6, p.EXTI6, Pull::None, Irqs),
            cam: ExtiInput::new(p.PA5, p.EXTI5, Pull::None, Irqs),
        }
    }
}

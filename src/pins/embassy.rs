//! microRusEFI board bring-up: safe GPIO and TLE8888.

mod tle8888;

pub use tle8888::{Tle8888Bus, init_or_log, spi_config};

use crate::pins::BoardPins;
use embassy_stm32::adc::{Adc, AdcChannel as _, AnyAdcChannel};
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::spi::Spi;
use embassy_stm32::{bind_interrupts, exti, interrupt};

bind_interrupts!(struct Irqs {
    EXTI9_5 => exti::InterruptHandler<interrupt::typelevel::EXTI9_5>;
});

/// ADC channels for the stage-1 sensor sweep (MRE default wiring).
pub struct SensorChannels {
    pub vbatt: AnyAdcChannel<'static, ADC1>,
    pub clt: AnyAdcChannel<'static, ADC1>,
    pub iat: AnyAdcChannel<'static, ADC1>,
    pub tps_map: AnyAdcChannel<'static, ADC1>,
    pub an_volt1: AnyAdcChannel<'static, ADC1>,
    pub an_volt2: AnyAdcChannel<'static, ADC1>,
}

/// All actuator pins driven to a known-safe state before any go/no-go decision.
pub struct SafeOutputs {
    pub ignition: [Output<'static>; 4],
    pub inj_en: Output<'static>,
    pub ign_en: Output<'static>,
    pub etb_pwm: Output<'static>,
    pub etb_dir: Output<'static>,
    pub etb_disable: Output<'static>,
}

impl SafeOutputs {
    /// Injectors/coils off, TLE8888 enables low, ETB H-bridge disabled.
    pub fn drive_off(&mut self) {
        for coil in &mut self.ignition {
            coil.set_low();
        }
        self.inj_en.set_low();
        self.ign_en.set_low();
        self.etb_pwm.set_low();
        self.etb_dir.set_low();
        // TLE9201 DIS low = motor disabled.
        self.etb_disable.set_low();
    }
}

/// The microRusEFI F7 board, split into owned Embassy devices.
pub struct MreBoard {
    pub led_comms: Output<'static>,
    pub led_running: Output<'static>,
    pub led_warning: Output<'static>,
    pub led_critical: Output<'static>,
    pub safe: SafeOutputs,
    pub adc: Adc<'static, ADC1>,
    pub sensors: SensorChannels,
    pub crank: ExtiInput<'static, Async>,
    pub cam: ExtiInput<'static, Async>,
}

impl MreBoard {
    /// Split `Peripherals` into board devices and a TLE8888 SPI bus.
    ///
    /// Safe outputs are driven off before returning. Returns the IWDG peripheral
    /// for the caller to configure (not consumed by board init).
    pub fn init(
        pins: BoardPins,
        p: embassy_stm32::Peripherals,
    ) -> (Self, Tle8888Bus, embassy_stm32::Peri<'static, embassy_stm32::peripherals::IWDG>) {
        if pins != BoardPins::mre_f7() {
            defmt::panic!("BoardPins do not match the MRE F7 map");
        }

        let iwdg = p.IWDG;

        let mut safe = SafeOutputs {
            ignition: [
                Output::new(p.PD4, Level::Low, Speed::Low),
                Output::new(p.PD3, Level::Low, Speed::Low),
                Output::new(p.PD2, Level::Low, Speed::Low),
                Output::new(p.PD1, Level::Low, Speed::Low),
            ],
            inj_en: Output::new(p.PD11, Level::Low, Speed::Low),
            ign_en: Output::new(p.PD10, Level::Low, Speed::Low),
            etb_pwm: Output::new(p.PC7, Level::Low, Speed::Low),
            etb_dir: Output::new(p.PA8, Level::Low, Speed::Low),
            etb_disable: Output::new(p.PC8, Level::Low, Speed::Low),
        };
        safe.drive_off();

        let cs = Output::new(p.PD5, Level::High, Speed::High);
        let spi = Spi::new_blocking(p.SPI1, p.PB3, p.PB5, p.PB4, spi_config());
        let tle = Tle8888Bus::new(spi, cs);

        let board = Self {
            led_comms: Output::new(p.PE2, Level::Low, Speed::Low),
            led_running: Output::new(p.PE4, Level::Low, Speed::Low),
            led_warning: Output::new(p.PE1, Level::Low, Speed::Low),
            led_critical: Output::new(p.PE3, Level::Low, Speed::Low),
            safe,
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
        };

        (board, tle, iwdg)
    }
}

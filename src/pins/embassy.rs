//! Bind abstract [`BoardPins`] / [`GpioPin`] to Embassy STM32 peripherals.

use crate::pins::{BoardPins, GpioPin, GpioPort};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::Peripherals;

/// Bind the MRE F7 status LEDs from a [`BoardPins`] map.
pub fn board_leds(
    pins: BoardPins,
    level: Level,
    speed: Speed,
    p: Peripherals,
) -> (Output<'static>, Output<'static>) {
    let expected = BoardPins::mre_f7();
    if pins.led_running != expected.led_running || pins.led_comms != expected.led_comms {
        defmt::panic!("LED pins do not match MRE F7 map");
    }

    match (pins.led_running, pins.led_comms) {
        (
            GpioPin {
                port: GpioPort::E,
                pin: 4,
            },
            GpioPin {
                port: GpioPort::E,
                pin: 2,
            },
        ) => (
            Output::new(p.PE4, level, speed),
            Output::new(p.PE2, level, speed),
        ),
        _ => defmt::panic!("unbound MRE F7 LED pair"),
    }
}

/// Bind one GPIO output when it is the only peripheral taken from the bundle.
pub fn output(
    pin: GpioPin,
    level: Level,
    speed: Speed,
    p: Peripherals,
) -> Output<'static> {
    match (pin.port, pin.pin) {
        (GpioPort::E, 1) => Output::new(p.PE1, level, speed),
        (GpioPort::E, 2) => Output::new(p.PE2, level, speed),
        (GpioPort::E, 3) => Output::new(p.PE3, level, speed),
        (GpioPort::E, 4) => Output::new(p.PE4, level, speed),
        (GpioPort::D, 1) => Output::new(p.PD1, level, speed),
        (GpioPort::D, 2) => Output::new(p.PD2, level, speed),
        (GpioPort::D, 3) => Output::new(p.PD3, level, speed),
        (GpioPort::D, 4) => Output::new(p.PD4, level, speed),
        _ => defmt::panic!("gpio output not bound for MRE F7"),
    }
}

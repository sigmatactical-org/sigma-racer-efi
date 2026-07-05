#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Speed};
use embassy_time::Timer;
use sigma_racer_efi::{
    FIRMWARE_ID, TARGET_MCU, active_profile, defaults::wiring, pins::BoardPins,
    pins::embassy,
};
use {defmt_rtt as _, panic_probe as _};

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let pins = BoardPins::mre_f7();
    let profile = active_profile();

    wiring::validate_profile(&profile).unwrap();

    let p = embassy_stm32::init(Default::default());

    info!(
        "{} on {} — engine: {}",
        FIRMWARE_ID, TARGET_MCU, profile.id
    );
    info!(
        "Engine: {} cylinders, {} cc, {}° cycle",
        profile.engine.cylinders,
        profile.engine.displacement_cc,
        profile.cycle_degrees
    );

    let (mut running_led, comms_led) =
        embassy::board_leds(pins, Level::Low, Speed::Low, p);

    spawner.spawn(heartbeat(comms_led).unwrap());

    loop {
        running_led.set_high();
        Timer::after_millis(500).await;
        running_led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn heartbeat(mut comms_led: embassy_stm32::gpio::Output<'static>) {
    loop {
        comms_led.set_high();
        Timer::after_millis(50).await;
        comms_led.set_low();
        Timer::after_secs(2).await;
    }
}

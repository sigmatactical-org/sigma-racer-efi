#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use sigma_efi_board_mre::{ENGINE_ID, FIRMWARE_ID, TARGET_MCU, default_profile, pins::BoardPins};
use {defmt_rtt as _, panic_probe as _};

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let pins = BoardPins::mre_f7();
    let profile = default_profile();
    let config = profile.engine;

    let p = embassy_stm32::init(Default::default());

    info!("{} starting on {} ({})", FIRMWARE_ID, TARGET_MCU, ENGINE_ID);
    info!(
        "Engine: {} cylinders, {} cc",
        config.cylinders, config.displacement_cc
    );

    let mut running_led = Output::new(p.PE4, Level::Low, Speed::Low);
    let comms_led = Output::new(p.PE2, Level::Low, Speed::Low);

    // Sanity check: board pin constants match the Embassy PAC pin names we drive.
    assert_eq!(pins.led_running.pin, 4);
    assert_eq!(pins.led_comms.pin, 2);

    spawner.spawn(heartbeat(comms_led).unwrap());

    loop {
        running_led.set_high();
        Timer::after_millis(500).await;
        running_led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn heartbeat(mut comms_led: Output<'static>) {
    loop {
        comms_led.set_high();
        Timer::after_millis(50).await;
        comms_led.set_low();
        Timer::after_secs(2).await;
    }
}

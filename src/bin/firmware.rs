#![no_std]
#![no_main]

use defmt::{error, info};
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

    // Initialise the MCU first so all outputs are driven to a known-safe Low
    // state (injectors/coils off) before we make any go/no-go decision.
    let p = embassy_stm32::init(Default::default());
    let (mut running_led, comms_led) =
        embassy::board_leds(pins, Level::Low, Speed::Low, p);

    // Refuse to start engine control with an invalid profile. Rather than
    // panicking (which would also halt with outputs safe, but silently), enter
    // an explicit fault state: outputs stay off and the running LED fast-blinks.
    if let Err(_err) = wiring::validate_profile(&profile) {
        error!("invalid engine profile — entering safe state, engine control disabled");
        loop {
            running_led.set_high();
            Timer::after_millis(120).await;
            running_led.set_low();
            Timer::after_millis(120).await;
        }
    }

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

    // Heartbeat is non-critical; log and continue if the executor is out of
    // task slots rather than aborting the whole firmware.
    match heartbeat(comms_led) {
        Ok(token) => spawner.spawn(token),
        Err(_) => error!("failed to spawn heartbeat task"),
    }

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

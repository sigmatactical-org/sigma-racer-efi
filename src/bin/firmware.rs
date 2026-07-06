#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::Timer;
use sigma_racer_efi::{
    FIRMWARE_ID, TARGET_MCU, active_profile, defaults::wiring, pins::BoardPins,
    pins::embassy::MreBoard,
};
use {defmt_rtt as _, panic_probe as _};

mod tasks;

/// 216 MHz sysclk from the internal 16 MHz HSI.
///
/// HSI is deliberate for bring-up: it works on any board regardless of the
/// fitted crystal. Once the physical MRE's HSE crystal is verified, switch
/// `pll_src` to HSE for CAN-grade clock accuracy (HSI is ±1% — fine for
/// ADC/EXTI logging, marginal for high-rate CAN).
///
/// HSI 16 MHz /8 → 2 MHz ×216 → 432 MHz VCO; /2 → 216 MHz sysclk (overdrive
/// is enabled by the RCC driver); /9 → 48 MHz for USB/SDMMC. APB1 54 MHz,
/// APB2 108 MHz (both at their maximums; ADC runs from APB2/4 = 27 MHz).
fn rcc_config() -> embassy_stm32::Config {
    use embassy_stm32::rcc::*;

    let mut config = embassy_stm32::Config::default();
    config.rcc.hsi = true;
    config.rcc.pll_src = PllSource::HSI;
    config.rcc.pll = Some(Pll {
        prediv: PllPreDiv::DIV8,
        mul: PllMul::MUL216,
        divp: Some(PllPDiv::DIV2),
        divq: Some(PllQDiv::DIV9),
        divr: None,
    });
    config.rcc.sys = Sysclk::PLL1_P;
    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV4;
    config.rcc.apb2_pre = APBPrescaler::DIV2;
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let pins = BoardPins::mre_f7();
    let profile = active_profile();

    // Initialise the MCU first so all outputs are driven to a known-safe Low
    // state (injectors/coils off) before we make any go/no-go decision.
    let p = embassy_stm32::init(rcc_config());
    let mut board = MreBoard::init(pins, p);

    // Refuse to run with an invalid profile. Rather than panicking (which
    // would also halt with outputs safe, but silently), enter an explicit
    // fault state: outputs stay off and the critical LED fast-blinks.
    if let Err(_err) = wiring::validate_profile(&profile) {
        error!("invalid engine profile — entering safe state, engine control disabled");
        loop {
            board.led_critical.set_high();
            Timer::after_millis(120).await;
            board.led_critical.set_low();
            Timer::after_millis(120).await;
        }
    }

    info!("{} on {} — engine: {}", FIRMWARE_ID, TARGET_MCU, profile.id);
    info!(
        "Engine: {} cylinders, {} cc, {}° cycle",
        profile.engine.cylinders, profile.engine.displacement_cc, profile.cycle_degrees
    );
    info!("stage 1: characterization data logger (DL,S sensor / DL,T trigger records)");
    defmt::warn!(
        "profile trigger geometry + rev limits are UNVERIFIED placeholders — logger stage only"
    );

    // Stage-1 data logger: analog sweep + crank/cam edge capture.
    // Failing to allocate any of these tasks is a broken build, not a
    // runtime condition to limp through — log loudly and stay safe.
    let logger_tokens = (
        tasks::sensors::sample(board.adc, board.sensors),
        tasks::trigger::crank_capture(board.crank),
        tasks::trigger::cam_capture(board.cam),
        tasks::trigger::edge_logger(),
    );
    match logger_tokens {
        (Ok(sensors), Ok(crank), Ok(cam), Ok(edges)) => {
            spawner.spawn(sensors);
            spawner.spawn(crank);
            spawner.spawn(cam);
            spawner.spawn(edges);
        }
        _ => {
            error!("failed to spawn data-logger tasks — entering safe state");
            loop {
                board.led_critical.set_high();
                Timer::after_millis(120).await;
                board.led_critical.set_low();
                Timer::after_millis(120).await;
            }
        }
    }

    // Heartbeat is non-critical; log and continue if the executor is out of
    // task slots rather than aborting the whole firmware.
    match heartbeat(board.led_comms) {
        Ok(token) => spawner.spawn(token),
        Err(_) => error!("failed to spawn heartbeat task"),
    }

    loop {
        board.led_running.set_high();
        Timer::after_millis(500).await;
        board.led_running.set_low();
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

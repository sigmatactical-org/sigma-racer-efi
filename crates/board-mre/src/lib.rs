//! microRusEFI board support.
//!
//! Pin assignments and defaults are derived from rusEFI board metadata:
//! - `firmware/config/boards/microrusefi/board_configuration.cpp`
//! - `firmware/config/boards/microrusefi/connectors/main.yaml`
//!
//! Hardware reference: [microRusEFI shop page](https://www.shop.rusefi.com/shop/p/microrusefi-assembled-ecu-development-module)

#![cfg_attr(not(test), no_std)]

pub mod analog;
pub mod defaults;
pub mod pins;

pub use defaults::{ENGINE_ID, FIRMWARE_ID, TARGET_MCU, default_engine_config, default_profile};
pub use pins::BoardPins;

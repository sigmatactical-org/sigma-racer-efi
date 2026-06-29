//! Engine control types and microRusEFI board support for Sigma EFI.
//!
//! Hardware-agnostic logic is `#![no_std]` and conceptually ported from
//! [rusEFI](https://github.com/rusefi/rusefi), reimplemented under MIT/Apache.

#![cfg_attr(not(test), no_std)]

pub mod analog;
pub mod config;
pub mod defaults;
pub mod engine;
pub mod engines;
pub mod pins;
pub mod sensors;
pub mod timing;

pub use config::EngineConfig;
pub use defaults::{ENGINE_ID, FIRMWARE_ID, TARGET_MCU, default_engine_config, default_profile};
pub use engine::EngineState;
pub use engines::rotax_v990_profile;
pub use pins::BoardPins;

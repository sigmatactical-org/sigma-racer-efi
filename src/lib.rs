//! Engine control types and microRusEFI board support for Sigma Racer EFI.
//!
//! Core logic is `#![no_std]` and engine-agnostic. Select an engine at
//! compile time via Cargo features (`engine-yamaha-cp3`).

#![cfg_attr(not(test), no_std)]

pub mod analog;
pub mod config;
pub mod datalog;
pub mod defaults;
pub mod engine;
pub mod engines;
pub mod pins;
pub mod sensors;
pub mod timing;

pub use config::EngineConfig;
pub use defaults::{FIRMWARE_ID, TARGET_MCU};
pub use engine::EngineState;
pub use engines::EngineProfile;
pub use engines::profile::CYCLE_DEGREES_FOUR_STROKE;
#[cfg(feature = "engine-yamaha-cp3")]
pub use engines::active_profile;
pub use pins::BoardPins;
pub use pins::TleOutput;

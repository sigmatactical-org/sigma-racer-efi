//! Engine control types and microRusEFI board support for Sigma Racer EFI.
//!
//! Core logic is `#![no_std]` and engine-agnostic. Select an engine at
//! compile time via Cargo features (`engine-yamaha-cp3`).

#![cfg_attr(not(test), no_std)]

pub mod analog;
#[cfg(feature = "firmware")]
pub mod bor;
pub mod can;
pub mod config;
pub mod datalog;
pub mod decoder;
pub mod defaults;
pub mod engine;
pub mod engines;
pub mod fueling;
#[cfg(feature = "firmware")]
pub mod heap;
pub mod pins;
pub mod rbw;
pub mod replay;
pub mod scheduler;
pub mod safety;
pub mod sensors;
pub mod tables;
pub mod timing;
pub mod tle8888;

pub use config::EngineConfig;
pub use defaults::{FIRMWARE_ID, TARGET_MCU};
pub use engine::EngineState;
pub use engines::EngineProfile;
pub use engines::profile::CYCLE_DEGREES_FOUR_STROKE;
#[cfg(feature = "engine-yamaha-cp3")]
pub use engines::active_profile;
pub use pins::BoardPins;
pub use pins::TleOutput;

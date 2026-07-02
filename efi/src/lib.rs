//! Engine control types and microRusEFI board support for Sigma EFI.
//!
//! Core logic is `#![no_std]` and engine-agnostic. Select an engine at
//! compile time via Cargo features (`engine-yamaha-cp3`, `engine-rotax-v990`).

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
pub use defaults::{FIRMWARE_ID, TARGET_MCU};
pub use engine::EngineState;
pub use engines::EngineProfile;
#[cfg(any(feature = "engine-yamaha-cp3", feature = "engine-rotax-v990"))]
pub use engines::active_profile;
pub use pins::BoardPins;

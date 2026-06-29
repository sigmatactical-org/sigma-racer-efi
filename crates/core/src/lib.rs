//! Core engine-control types and algorithms for Sigma EFI.
//!
//! This crate is `#![no_std]` and holds hardware-agnostic logic ported conceptually
//! from [rusEFI](https://github.com/rusefi/rusefi), reimplemented under MIT/Apache.

#![cfg_attr(not(test), no_std)]

pub mod config;
pub mod engine;
pub mod engines;
pub mod sensors;
pub mod timing;

pub use config::EngineConfig;
pub use engine::EngineState;
pub use engines::rotax_v990_profile;

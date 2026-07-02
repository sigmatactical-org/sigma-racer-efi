//! Known engine profiles and compile-time engine selection.

pub mod profile;
pub mod yamaha_cp3;

pub use profile::EngineProfile;

/// Engine profile selected at compile time via Cargo features.
#[cfg(feature = "engine-yamaha-cp3")]
pub fn active_profile() -> EngineProfile {
    yamaha_cp3::profile()
}

#[cfg(all(feature = "firmware", not(feature = "engine-yamaha-cp3")))]
compile_error!("firmware builds require an engine feature: engine-yamaha-cp3");

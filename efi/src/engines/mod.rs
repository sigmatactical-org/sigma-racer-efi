//! Known engine profiles and compile-time engine selection.

pub mod profile;
pub mod rotax_v990;
pub mod yamaha_cp3;

pub use profile::EngineProfile;

/// Engine profile selected at compile time via Cargo features.
///
/// If both `engine-yamaha-cp3` and `engine-rotax-v990` are enabled, CP3 takes
/// precedence. Prefer enabling exactly one engine feature per build.
#[cfg(feature = "engine-yamaha-cp3")]
pub fn active_profile() -> EngineProfile {
    yamaha_cp3::profile()
}

#[cfg(all(not(feature = "engine-yamaha-cp3"), feature = "engine-rotax-v990"))]
pub fn active_profile() -> EngineProfile {
    rotax_v990::profile()
}

#[cfg(all(
    feature = "firmware",
    not(any(feature = "engine-yamaha-cp3", feature = "engine-rotax-v990"))
))]
compile_error!(
    "firmware builds require an engine feature: engine-yamaha-cp3 or engine-rotax-v990"
);

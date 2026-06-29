//! Known engine profiles (factory specs, trigger patterns, defaults).

pub mod rotax_v990;

pub use rotax_v990::{
    Profile as RotaxV990Profile, engine_config as rotax_v990_config, profile as rotax_v990_profile,
};

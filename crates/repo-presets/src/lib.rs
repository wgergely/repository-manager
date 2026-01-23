//! Preset providers for Repository Manager.
//!
//! This crate provides preset detection and configuration providers
//! for various development environments.

pub mod context;
pub mod error;
pub mod provider;
pub mod python;

pub use context::Context;
pub use error::{Error, Result};
pub use provider::{ActionType, ApplyReport, CheckReport, PresetProvider, PresetStatus};
pub use python::UvProvider;

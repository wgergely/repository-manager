//! Preset providers for Repository Manager.
//!
//! This crate provides preset detection and configuration providers
//! for various development environments.

pub mod context;
pub mod error;
pub mod node;
pub mod provider;
pub mod python;
pub mod rust;

pub use context::Context;
pub use error::{Error, Result};
pub use node::NodeProvider;
pub use provider::{
    ActionType, ApplyReport, ApplyStatus, PresetCheckReport, PresetProvider, PresetStatus,
};
pub use python::{UvProvider, VenvProvider};
pub use rust::RustProvider;

//! Preset providers for Repository Manager.
//!
//! This crate provides preset detection and configuration providers
//! for various development environments.

pub mod error;
pub mod provider;
pub mod context;
pub mod python;

pub use error::{Error, Result};

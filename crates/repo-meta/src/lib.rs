//! Metadata and configuration management for Repository Manager.
//!
//! This crate provides configuration types and a provider registry
//! for managing repository metadata.

pub mod config;
pub mod error;
pub mod registry;

pub use error::{Error, Result};
pub use registry::Registry;

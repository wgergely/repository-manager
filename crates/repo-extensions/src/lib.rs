//! Extension system for Repository Manager.
//!
//! This crate provides manifest parsing, configuration, and a registry
//! for repository-manager extensions.

pub mod config;
pub mod error;
pub mod manifest;
pub mod registry;

pub use config::ExtensionConfig;
pub use error::Error;
pub use manifest::{EntryPoints, ExtensionManifest, ResolvedCommand, ResolvedEntryPoints};
pub use registry::{ExtensionEntry, ExtensionRegistry};

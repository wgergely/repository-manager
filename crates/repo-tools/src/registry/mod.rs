//! Unified Tool Registry - Single Source of Truth
//!
//! This module provides a centralized registry for tool definitions,
//! eliminating the 3-location duplication in the old dispatcher.

mod builtins;
mod store;
mod types;

pub use builtins::{builtin_registrations, BUILTIN_COUNT};
pub use store::ToolRegistry;
pub use types::{ToolCategory, ToolRegistration};

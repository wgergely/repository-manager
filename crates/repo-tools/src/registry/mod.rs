//! Unified Tool Registry - Single Source of Truth
//!
//! This module provides a centralized registry for tool definitions,
//! eliminating the 3-location duplication in the old dispatcher.

mod store;
mod types;

pub use store::ToolRegistry;
pub use types::{ToolCategory, ToolRegistration};

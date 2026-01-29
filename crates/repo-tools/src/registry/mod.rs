//! Unified Tool Registry - Single Source of Truth
//!
//! This module provides a centralized registry for tool definitions,
//! eliminating the 3-location duplication in the old dispatcher.

mod types;

pub use types::{ToolCategory, ToolRegistration};

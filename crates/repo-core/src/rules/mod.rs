//! Rule Registry module
//!
//! Provides central rule management with UUID-based identification.
//! Rule UUIDs are used as managed block markers in tool config files.

mod registry;
mod rule;

pub use registry::RuleRegistry;
pub use rule::Rule;

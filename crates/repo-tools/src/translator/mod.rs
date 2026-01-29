//! Capability-based translation layer
//!
//! This module translates rules and other content into tool-specific
//! formats, respecting each tool's declared capabilities.

mod content;
mod rules;

pub use content::TranslatedContent;
pub use rules::RuleTranslator;

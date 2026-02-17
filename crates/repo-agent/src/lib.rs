//! Agent orchestration integration for Repository Manager
//!
//! This crate provides the `AgentManager` which discovers and manages
//! the vaultspec agent framework. It handles:
//!
//! - Discovery of Python 3.13+ interpreter
//! - Location and validation of the `.vaultspec/` framework directory
//! - Health checking of the agent subsystem
//!
//! The agent subsystem is optional -- when Python or vaultspec are not
//! available, commands gracefully degrade with helpful error messages.

pub mod discovery;
pub mod error;
pub mod types;

pub use discovery::AgentManager;
pub use error::{AgentError, Result};
pub use types::{AgentInfo, HealthReport, TaskStatus};

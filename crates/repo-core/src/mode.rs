//! Repository mode abstraction
//!
//! This module re-exports [`repo_meta::RepositoryMode`] as [`Mode`] for
//! convenient use within the core crate and downstream consumers.
//!
//! The canonical definition lives in `repo-meta`. This type alias avoids
//! a duplicate enum while preserving the short `Mode` name used throughout
//! `repo-core` and `repo-cli`.

/// Repository operation mode (type alias for [`repo_meta::RepositoryMode`]).
pub type Mode = repo_meta::RepositoryMode;

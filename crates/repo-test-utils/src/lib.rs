//! Shared test utilities for the repository-manager workspace.
//!
//! This crate provides standardised test fixtures to eliminate duplication
//! across crate test suites. It is a dev-dependency only — never published.
//!
//! # Modules
//!
//! - [`git`] — git repository fixtures at three realism levels
//! - [`repo`] — [`TestRepo`] builder for full repository-manager setup

pub mod git;
pub mod repo;

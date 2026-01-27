//! MCP Resource implementations
//!
//! This module contains the resource handlers for the MCP server.
//! Resources provide read-only access to repository state.
//!
//! # Available Resources
//!
//! | URI | Description | Content-Type |
//! |-----|-------------|--------------|
//! | `repo://config` | Repository configuration (.repository/config.toml) | application/toml |
//! | `repo://state` | Computed state from ledger (.repository/ledger.toml) | application/toml |
//! | `repo://rules` | Aggregated view of all active rules | text/markdown |

// TODO: Implement resource handlers

//! MCP Server for Repository Manager
//!
//! This crate exposes Repository Manager functionality via the Model Context Protocol (MCP),
//! allowing Agentic IDEs (like Claude Desktop, Windsurf, Cursor) to manage repository
//! structure, configuration, and controlled Git operations.
//!
//! # Architecture
//!
//! The `repo-mcp` crate acts as a facade layer over the core `repo-manager` library:
//!
//! ```text
//! [ MCP Client (Claude/IDE) ]
//!        | (JSON-RPC)
//!        v
//! [ repo-mcp (MCP Server) ]
//!        | (Rust API)
//!        v
//! [ repo-core (Core Logic) ]
//!        |
//!        +--> [ .repository/ (Config Store) ]
//!        +--> [ git / worktrees (Filesystem) ]
//! ```
//!
//! # Tools
//!
//! The server exposes tools for:
//! - Repository lifecycle (init, check, fix, sync)
//! - Branch management (create, delete, list, checkout)
//! - Git primitives (push, pull, merge)
//! - Configuration management (tools, presets, rules)
//!
//! # Resources
//!
//! Read-only resources exposed:
//! - `repo://config` - Repository configuration
//! - `repo://state` - Computed state from ledger
//! - `repo://rules` - Aggregated active rules

pub mod error;
pub mod resources;
pub mod server;
pub mod tools;

pub use error::{Error, Result};
pub use server::RepoMcpServer;
pub use tools::{get_tool_definitions, ToolContent, ToolDefinition, ToolResult};

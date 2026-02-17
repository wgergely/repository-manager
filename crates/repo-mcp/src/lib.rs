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
pub mod handlers;
pub mod protocol;
pub mod resource_handlers;
pub mod resources;
pub mod server;
pub mod tools;

pub use error::{Error, Result};
pub use handlers::handle_tool_call;
pub use resource_handlers::read_resource;
pub use server::RepoMcpServer;
pub use tools::{ToolContent, ToolDefinition, ToolResult, get_tool_definitions};

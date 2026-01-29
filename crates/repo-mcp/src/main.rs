//! Repository Manager MCP Server
//!
//! A Model Context Protocol server that exposes Repository Manager functionality
//! to agentic IDEs like Claude Desktop, Windsurf, and Cursor.
//!
//! # Usage
//!
//! ```bash
//! repo-mcp [--root <path>]
//! ```
//!
//! # Environment Variables
//!
//! - `RUST_LOG`: Control log verbosity (default: `repo_mcp=info`)
//!
//! # Protocol
//!
//! The server communicates via JSON-RPC 2.0 over stdio:
//! - Requests/responses go through stdout
//! - Logs go to stderr (to avoid interfering with the protocol)

use std::path::PathBuf;

use clap::Parser;
use repo_mcp::RepoMcpServer;

/// MCP server for Repository Manager
#[derive(Parser)]
#[command(name = "repo-mcp")]
#[command(about = "MCP server for Repository Manager")]
#[command(version)]
struct Args {
    /// Repository root path
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("repo_mcp=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    tracing::info!(root = ?args.root, "Starting repo-mcp server");

    let mut server = RepoMcpServer::new(args.root);
    server.run().await?;

    Ok(())
}

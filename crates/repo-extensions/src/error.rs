use std::fmt;
use std::path::PathBuf;

/// Newtype wrapper that displays an `Option<String>` as either an empty string
/// (when `None`) or the contained string (when `Some`). Used for optional hint
/// messages in error variants.
struct OptHint<'a>(&'a Option<String>);

impl fmt::Display for OptHint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hint) = self.0 {
            write!(f, "{}", hint)
        } else {
            Ok(())
        }
    }
}

/// Errors that can occur in the extension system.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to parse extension manifest TOML.
    #[error("failed to parse extension manifest: {0}")]
    ManifestParse(#[from] toml::de::Error),

    /// Extension manifest file not found at the expected path.
    #[error("extension manifest not found: {0}")]
    ManifestNotFound(PathBuf),

    /// Invalid semver version string.
    #[error("invalid version '{version}': {source}")]
    InvalidVersion {
        version: String,
        source: semver::Error,
    },

    /// Invalid extension name.
    #[error("invalid extension name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    /// Failed to serialize extension manifest.
    #[error("failed to serialize extension manifest: {0}")]
    ManifestSerialize(String),

    /// I/O error reading or writing extension files.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Extension not found in registry.
    #[error("unknown extension: {0}")]
    UnknownExtension(String),

    /// MCP config file declared in extension manifest was not found.
    #[error("MCP config not found at {path} for extension '{extension}'")]
    McpConfigNotFound {
        path: PathBuf,
        extension: String,
    },

    /// Failed to parse MCP config JSON.
    #[error("failed to parse MCP config at {path}: {reason}")]
    McpConfigParse {
        path: PathBuf,
        reason: String,
    },

    /// Dependency cycle detected in the extension/preset graph.
    #[error("dependency cycle detected among: {}", participants.join(", "))]
    DependencyCycle {
        participants: Vec<String>,
    },

    /// Failed to parse a version constraint string.
    #[error("invalid version constraint '{constraint}': {reason}")]
    VersionConstraintParse {
        constraint: String,
        reason: String,
    },

    /// A version constraint was not satisfied.
    #[error("version constraint not satisfied: {constraint} (have {actual})")]
    VersionConstraintNotSatisfied {
        constraint: String,
        actual: String,
    },

    /// Failed to parse or write the lock file.
    #[error("lock file error: {0}")]
    LockFileParse(String),

    /// Unknown package manager value in manifest.
    #[error("unknown package_manager '{value}'; known values: uv, pip, npm, yarn, pnpm, cargo, bun")]
    InvalidPackageManager { value: String },

    /// The extension's install command failed.
    #[error("install failed for '{name}': command '{command}' exited with {exit_code:?} — check output above for details")]
    InstallFailed {
        name: String,
        command: String,
        exit_code: Option<i32>,
    },

    /// A required binary was not found on PATH.
    #[error("install requires '{tool}' but it was not found on PATH{}", OptHint(hint))]
    PackageManagerNotFound { tool: String, hint: Option<String> },

    /// Invalid packages declaration.
    #[error("invalid packages declaration: {reason}")]
    InvalidPackages { reason: String },

    /// Invalid venv_path in manifest (absolute path or path escaping extension dir).
    #[error("invalid venv_path '{path}': must be a relative path within the extension directory")]
    InvalidVenvPath { path: String },

    /// Extension is not installed (not found in the lock file).
    #[error("extension '{0}' is not installed")]
    ExtensionNotInstalled(String),
}

pub type Result<T> = std::result::Result<T, Error>;

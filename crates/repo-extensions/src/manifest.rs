//! Extension manifest parsing for `repo_extension.toml` files.
//!
//! An extension manifest declares metadata, runtime requirements, entry points,
//! and output directories for a repository-manager extension. The canonical
//! filename is [`MANIFEST_FILENAME`](crate::MANIFEST_FILENAME)
//! (`repo_extension.toml`).
//!
//! # Example TOML
//!
//! ```toml
//! [extension]
//! name = "vaultspec"
//! version = "0.1.0"
//! description = "A governed development framework for AI agents"
//!
//! [requires.python]
//! version = ">=3.13"
//!
//! [runtime]
//! type = "python"
//! install = "pip install -e '.[dev]'"
//!
//! [entry_points]
//! cli = ".vaultspec/lib/scripts/cli.py"
//! mcp = ".vaultspec/lib/scripts/subagent.py serve"
//!
//! [provides]
//! mcp = ["vs-subagent-mcp"]
//! mcp_config = "mcp.json"
//! content_types = ["rules", "agents", "skills", "system", "templates"]
//!
//! [outputs]
//! claude_dir = ".claude"
//! gemini_dir = ".gemini"
//! agent_dir = ".agent"
//! agents_md = "AGENTS.md"
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Complete extension manifest loaded from `repo_extension.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtensionManifest {
    /// Core extension metadata.
    pub extension: ExtensionMeta,
    /// Language/runtime requirements.
    #[serde(default)]
    pub requires: Option<Requirements>,
    /// Runtime configuration.
    #[serde(default)]
    pub runtime: Option<RuntimeConfig>,
    /// Entry points for CLI and MCP.
    #[serde(default)]
    pub entry_points: Option<EntryPoints>,
    /// Capabilities this extension provides.
    #[serde(default)]
    pub provides: Option<Provides>,
    /// Output directory/file mappings.
    #[serde(default)]
    pub outputs: Option<Outputs>,
}

/// Basic metadata about an extension.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionMeta {
    /// Extension name (e.g., "vaultspec").
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Language/runtime requirements.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Requirements {
    /// Python version requirement.
    #[serde(default)]
    pub python: Option<PythonRequirement>,
}

/// Python version requirement.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PythonRequirement {
    /// Version constraint string (e.g., ">=3.13").
    pub version: String,
    /// PEP 508 package specifiers to install (e.g., `["httpx>=0.27"]`).
    #[serde(default)]
    pub packages: Vec<String>,
}

/// Runtime configuration for the extension.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeConfig {
    /// Runtime type (e.g., "python", "node").
    #[serde(rename = "type")]
    pub runtime_type: String,
    /// Package manager required by this extension (e.g., "uv", "npm").
    #[serde(default)]
    pub package_manager: Option<String>,
    /// Path to the virtual environment, relative to the extension source directory.
    ///
    /// When set, overrides the default `.repository/extensions/{name}/.venv/` location.
    /// Must be a relative path with no `..` components.
    #[serde(default)]
    pub venv_path: Option<String>,
    /// Install command to set up the extension.
    #[serde(default)]
    pub install: Option<String>,
}

/// Entry points exposed by the extension.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EntryPoints {
    /// CLI entry point path.
    #[serde(default)]
    pub cli: Option<String>,
    /// MCP server entry point command.
    #[serde(default)]
    pub mcp: Option<String>,
}

/// A resolved command with absolute program path and arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCommand {
    /// Absolute path to the program to execute.
    pub program: PathBuf,
    /// Arguments to pass to the program.
    pub args: Vec<String>,
}

/// Resolved entry points with absolute paths.
#[derive(Debug, Clone)]
pub struct ResolvedEntryPoints {
    /// Resolved CLI entry point.
    pub cli: Option<ResolvedCommand>,
    /// Resolved MCP server entry point.
    pub mcp: Option<ResolvedCommand>,
}

impl EntryPoints {
    /// Resolve entry points against a Python interpreter path and source directory.
    ///
    /// Relative entry point paths are resolved against `source_dir`.
    /// The entry point string is split into a command and arguments
    /// (e.g., `"subagent.py serve"` becomes command `subagent.py` with args `["serve"]`).
    /// The command is then prepended with the Python interpreter path.
    pub fn resolve(&self, python_path: &Path, source_dir: &Path) -> ResolvedEntryPoints {
        ResolvedEntryPoints {
            cli: self
                .cli
                .as_ref()
                .map(|ep| Self::resolve_one(python_path, source_dir, ep)),
            mcp: self
                .mcp
                .as_ref()
                .map(|ep| Self::resolve_one(python_path, source_dir, ep)),
        }
    }

    fn resolve_one(python_path: &Path, source_dir: &Path, entry_point: &str) -> ResolvedCommand {
        let parts: Vec<&str> = entry_point.split_whitespace().collect();
        let (script, args) = match parts.split_first() {
            Some((first, rest)) => (*first, rest.iter().map(|s| s.to_string()).collect()),
            None => (entry_point, Vec::new()),
        };

        let script_path = Path::new(script);

        // Security: Reject absolute paths in entry points. Entry points must be
        // relative to the extension source directory to prevent executing arbitrary
        // binaries outside the extension.
        let resolved_script = if script_path.is_absolute() {
            tracing::warn!(
                "Extension entry_point uses absolute path {:?} — forcing relative resolution",
                script
            );
            // Strip the leading / and resolve relative to source_dir
            let relative = script.trim_start_matches('/').trim_start_matches('\\');
            source_dir.join(relative)
        } else {
            source_dir.join(script_path)
        };

        ResolvedCommand {
            program: python_path.to_path_buf(),
            args: std::iter::once(resolved_script.to_string_lossy().into_owned())
                .chain(args)
                .collect(),
        }
    }
}

/// Capabilities the extension provides.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Provides {
    /// MCP server names provided.
    #[serde(default)]
    pub mcp: Vec<String>,
    /// Path to an `mcp.json` file relative to the extension source directory.
    ///
    /// When set, the repo manager reads MCP server definitions from this file,
    /// resolves runtime paths (e.g., Python venv, repo root), and writes the
    /// resolved configuration into each tool that supports MCP.
    #[serde(default)]
    pub mcp_config: Option<String>,
    /// Content types this extension manages.
    #[serde(default)]
    pub content_types: Vec<String>,
}

/// Output directory/file mappings.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Outputs {
    /// Claude AI output directory.
    #[serde(default)]
    pub claude_dir: Option<String>,
    /// Gemini AI output directory.
    #[serde(default)]
    pub gemini_dir: Option<String>,
    /// Generic agent output directory.
    #[serde(default)]
    pub agent_dir: Option<String>,
    /// Agents markdown file path.
    #[serde(default)]
    pub agents_md: Option<String>,
}

impl ExtensionManifest {
    /// Parse an extension manifest from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self> {
        let manifest: Self = toml::from_str(content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Read and parse an extension manifest from a file path.
    pub fn from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(Error::ManifestNotFound(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Serialize the manifest back to a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::ManifestSerialize(e.to_string()))
    }

    /// Validate the manifest fields.
    fn validate(&self) -> Result<()> {
        // Validate extension name is non-empty and uses valid characters
        let name = &self.extension.name;
        if name.is_empty() {
            return Err(Error::InvalidName {
                name: name.clone(),
                reason: "extension name must not be empty".to_string(),
            });
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(Error::InvalidName {
                name: name.clone(),
                reason: "extension name must contain only alphanumeric characters, hyphens, or underscores".to_string(),
            });
        }

        // Validate that the version is valid semver
        semver::Version::parse(&self.extension.version).map_err(|e| Error::InvalidVersion {
            version: self.extension.version.clone(),
            source: e,
        })?;

        // Validate that Python version constraint is parseable (if declared)
        if let Some(ref requires) = self.requires {
            if let Some(ref python_req) = requires.python {
                crate::version::VersionConstraint::parse(&python_req.version)?;
            }
        }

        // Validate package_manager value (if declared)
        const KNOWN_PACKAGE_MANAGERS: &[&str] =
            &["uv", "pip", "npm", "yarn", "pnpm", "cargo", "bun"];
        if let Some(ref rt) = self.runtime {
            if let Some(ref pm) = rt.package_manager {
                if !KNOWN_PACKAGE_MANAGERS.contains(&pm.as_str()) {
                    return Err(Error::InvalidPackageManager { value: pm.clone() });
                }
            }
        }

        // Validate packages entries (if declared): non-empty, no shell metacharacters
        if let Some(ref requires) = self.requires {
            if let Some(ref req) = requires.python {
                const SHELL_META: &[char] = &[';', '&', '|', '`', '$', '(', ')'];
                for pkg in &req.packages {
                    if pkg.is_empty() {
                        return Err(Error::InvalidPackages {
                            reason: "empty specifier".to_string(),
                        });
                    }
                    if pkg.chars().any(|c| SHELL_META.contains(&c)) {
                        return Err(Error::InvalidPackages {
                            reason: format!("shell metacharacter in package specifier: {:?}", pkg),
                        });
                    }
                }
            }
        }

        // Validate venv_path confinement (if declared): must be relative, no .. components
        if let Some(ref rt) = self.runtime {
            if let Some(ref vp) = rt.venv_path {
                let path = std::path::Path::new(vp);
                if path.has_root() {
                    return Err(Error::InvalidVenvPath { path: vp.clone() });
                }
                if path.components().any(|c| c == std::path::Component::ParentDir) {
                    return Err(Error::InvalidVenvPath { path: vp.clone() });
                }
            }
        }

        Ok(())
    }

    /// Check whether a given Python version satisfies this manifest's
    /// `[requires.python]` constraint.
    ///
    /// Returns `Ok(true)` if satisfied or no constraint is declared.
    /// Returns `Ok(false)` if the constraint is not satisfied.
    /// Returns `Err` if the constraint string is malformed.
    pub fn python_version_satisfied(&self, python_version: &str) -> Result<bool> {
        let constraint_str = match self
            .requires
            .as_ref()
            .and_then(|r| r.python.as_ref())
        {
            Some(req) => &req.version,
            None => return Ok(true), // No constraint declared
        };

        let constraint = crate::version::VersionConstraint::parse(constraint_str)?;
        Ok(constraint.satisfies(python_version))
    }

    /// Return the implicit preset dependencies inferred from this manifest.
    ///
    /// For example, an extension with `runtime.type = "python"` implicitly
    /// depends on the `env:python` preset.
    pub fn implicit_preset_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();

        if let Some(ref runtime) = self.runtime {
            deps.push(format!("env:{}", runtime.runtime_type));
        }

        // Explicit Python requirement also implies env:python (dedup)
        if self.requires.as_ref().is_some_and(|r| r.python.is_some())
            && !deps.contains(&"env:python".to_string())
        {
            deps.push("env:python".to_string());
        }

        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VAULTSPEC_TOML: &str = r#"
[extension]
name = "vaultspec"
version = "0.1.0"
description = "A governed development framework for AI agents"

[requires.python]
version = ">=3.13"

[runtime]
type = "python"
package_manager = "uv"
install = "pip install -e '.[dev]'"

[entry_points]
cli = ".vaultspec/lib/scripts/cli.py"
mcp = ".vaultspec/lib/scripts/subagent.py serve"

[provides]
mcp = ["vs-subagent-mcp"]
content_types = ["rules", "agents", "skills", "system", "templates"]

[outputs]
claude_dir = ".claude"
gemini_dir = ".gemini"
agent_dir = ".agent"
agents_md = "AGENTS.md"
"#;

    #[test]
    fn test_parse_full_manifest() {
        let manifest = ExtensionManifest::from_toml(VAULTSPEC_TOML).unwrap();

        assert_eq!(manifest.extension.name, "vaultspec");
        assert_eq!(manifest.extension.version, "0.1.0");
        assert_eq!(
            manifest.extension.description.as_deref(),
            Some("A governed development framework for AI agents")
        );

        let requires = manifest.requires.unwrap();
        assert_eq!(requires.python.unwrap().version, ">=3.13");

        let runtime = manifest.runtime.unwrap();
        assert_eq!(runtime.runtime_type, "python");
        assert_eq!(runtime.package_manager.as_deref(), Some("uv"));
        assert_eq!(runtime.install.as_deref(), Some("pip install -e '.[dev]'"));

        let entry_points = manifest.entry_points.unwrap();
        assert_eq!(
            entry_points.cli.as_deref(),
            Some(".vaultspec/lib/scripts/cli.py")
        );
        assert_eq!(
            entry_points.mcp.as_deref(),
            Some(".vaultspec/lib/scripts/subagent.py serve")
        );

        let provides = manifest.provides.unwrap();
        assert_eq!(provides.mcp, vec!["vs-subagent-mcp"]);
        assert_eq!(
            provides.content_types,
            vec!["rules", "agents", "skills", "system", "templates"]
        );

        let outputs = manifest.outputs.unwrap();
        assert_eq!(outputs.claude_dir.as_deref(), Some(".claude"));
        assert_eq!(outputs.gemini_dir.as_deref(), Some(".gemini"));
        assert_eq!(outputs.agent_dir.as_deref(), Some(".agent"));
        assert_eq!(outputs.agents_md.as_deref(), Some("AGENTS.md"));
    }

    #[test]
    fn test_parse_minimal_manifest() {
        let toml = r#"
[extension]
name = "minimal"
version = "1.0.0"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.extension.name, "minimal");
        assert_eq!(manifest.extension.version, "1.0.0");
        assert!(manifest.requires.is_none());
        assert!(manifest.runtime.is_none());
        assert!(manifest.entry_points.is_none());
        assert!(manifest.provides.is_none());
        assert!(manifest.outputs.is_none());
    }

    #[test]
    fn test_invalid_version_rejected() {
        let toml = r#"
[extension]
name = "bad"
version = "not-a-version"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::InvalidVersion { .. }));
    }

    #[test]
    fn test_missing_name_rejected() {
        let toml = r#"
[extension]
version = "1.0.0"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::ManifestParse(_)));
    }

    #[test]
    fn test_missing_version_rejected() {
        let toml = r#"
[extension]
name = "no-version"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::ManifestParse(_)));
    }

    #[test]
    fn test_missing_extension_section_rejected() {
        let toml = r#"
[runtime]
type = "python"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::ManifestParse(_)));
    }

    #[test]
    fn test_empty_name_rejected() {
        let toml = r#"
[extension]
name = ""
version = "1.0.0"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::InvalidName { .. }));
    }

    #[test]
    fn test_name_with_spaces_rejected() {
        let toml = r#"
[extension]
name = "bad name"
version = "1.0.0"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::InvalidName { .. }));
    }

    #[test]
    fn test_name_with_hyphens_and_underscores_accepted() {
        let toml = r#"
[extension]
name = "my-cool_extension"
version = "1.0.0"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.extension.name, "my-cool_extension");
    }

    #[test]
    fn test_unknown_top_level_section_accepted() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[unknown_section]
foo = "bar"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.extension.name, "test");
    }

    #[test]
    fn test_unknown_field_in_extension_section_rejected() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"
author = "someone"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(matches!(err, Error::ManifestParse(_)));
    }

    #[test]
    fn test_empty_provides_vectors() {
        let toml = r#"
[extension]
name = "empty-provides"
version = "1.0.0"

[provides]
mcp = []
content_types = []
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let provides = manifest.provides.unwrap();
        assert!(provides.mcp.is_empty());
        assert!(provides.content_types.is_empty());
    }

    #[test]
    fn test_toml_round_trip() {
        let manifest = ExtensionManifest::from_toml(VAULTSPEC_TOML).unwrap();
        let serialized = manifest.to_toml().unwrap();
        let reparsed = ExtensionManifest::from_toml(&serialized).unwrap();

        assert_eq!(manifest.extension.name, reparsed.extension.name);
        assert_eq!(manifest.extension.version, reparsed.extension.version);
        assert_eq!(
            manifest.extension.description,
            reparsed.extension.description
        );
        assert_eq!(
            manifest.runtime.as_ref().map(|r| &r.runtime_type),
            reparsed.runtime.as_ref().map(|r| &r.runtime_type)
        );
        assert_eq!(
            manifest.provides.as_ref().map(|p| &p.mcp),
            reparsed.provides.as_ref().map(|p| &p.mcp)
        );
        assert_eq!(
            manifest.provides.as_ref().map(|p| &p.content_types),
            reparsed.provides.as_ref().map(|p| &p.content_types)
        );
    }

    #[test]
    fn test_from_path_reads_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let file_path = dir.path().join(crate::MANIFEST_FILENAME);
        std::fs::write(&file_path, VAULTSPEC_TOML).unwrap();

        let manifest = ExtensionManifest::from_path(&file_path).unwrap();
        assert_eq!(manifest.extension.name, "vaultspec");
        assert_eq!(manifest.extension.version, "0.1.0");
    }

    #[test]
    fn test_from_path_not_found() {
        let err =
            ExtensionManifest::from_path(Path::new("/nonexistent/repo_extension.toml")).unwrap_err();
        assert!(matches!(err, Error::ManifestNotFound(_)));
    }

    #[test]
    fn test_error_messages_are_actionable() {
        // Invalid version error should include the version string
        let toml = r#"
[extension]
name = "test"
version = "abc"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("abc"),
            "error should include the invalid version: {msg}"
        );

        // Invalid name error should include the name
        let toml = r#"
[extension]
name = "bad name!"
version = "1.0.0"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("bad name!"),
            "error should include the invalid name: {msg}"
        );
    }

    #[test]
    fn test_resolve_entry_points_cli_only() {
        let ep = EntryPoints {
            cli: Some(".vaultspec/lib/scripts/cli.py".to_string()),
            mcp: None,
        };
        let source = Path::new("/src/ext");
        let resolved = ep.resolve(Path::new("/usr/bin/python3"), source);

        let cli = resolved.cli.unwrap();
        assert_eq!(cli.program, PathBuf::from("/usr/bin/python3"));
        let expected = source
            .join(".vaultspec/lib/scripts/cli.py")
            .to_string_lossy()
            .into_owned();
        assert_eq!(cli.args, vec![expected]);
        assert!(resolved.mcp.is_none());
    }

    #[test]
    fn test_resolve_entry_points_mcp_with_args() {
        let ep = EntryPoints {
            cli: None,
            mcp: Some(".vaultspec/lib/scripts/subagent.py serve".to_string()),
        };
        let source = Path::new("/src/ext");
        let resolved = ep.resolve(Path::new("/venv/bin/python"), source);

        assert!(resolved.cli.is_none());
        let mcp = resolved.mcp.unwrap();
        assert_eq!(mcp.program, PathBuf::from("/venv/bin/python"));
        let expected_script = source
            .join(".vaultspec/lib/scripts/subagent.py")
            .to_string_lossy()
            .into_owned();
        assert_eq!(mcp.args, vec![expected_script, "serve".to_string()]);
    }

    #[test]
    fn test_resolve_entry_points_both() {
        let ep = EntryPoints {
            cli: Some("scripts/cli.py".to_string()),
            mcp: Some("scripts/mcp.py serve --port 8080".to_string()),
        };
        let source = Path::new("/ext");
        let resolved = ep.resolve(Path::new("/py"), source);

        let cli = resolved.cli.unwrap();
        assert_eq!(cli.program, PathBuf::from("/py"));
        let expected_cli = source.join("scripts/cli.py").to_string_lossy().into_owned();
        assert_eq!(cli.args, vec![expected_cli]);

        let mcp = resolved.mcp.unwrap();
        assert_eq!(mcp.program, PathBuf::from("/py"));
        let expected_mcp = source.join("scripts/mcp.py").to_string_lossy().into_owned();
        assert_eq!(
            mcp.args,
            vec![
                expected_mcp,
                "serve".to_string(),
                "--port".to_string(),
                "8080".to_string()
            ]
        );
    }

    #[test]
    fn test_resolve_entry_points_empty() {
        let ep = EntryPoints {
            cli: None,
            mcp: None,
        };
        let resolved = ep.resolve(Path::new("/py"), Path::new("/ext"));
        assert!(resolved.cli.is_none());
        assert!(resolved.mcp.is_none());
    }

    #[test]
    fn test_parse_provides_with_mcp_config() {
        let toml = r#"
[extension]
name = "mcp-ext"
version = "1.0.0"

[provides]
mcp = ["my-server"]
mcp_config = "mcp.json"
content_types = []
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let provides = manifest.provides.unwrap();
        assert_eq!(provides.mcp, vec!["my-server"]);
        assert_eq!(provides.mcp_config.as_deref(), Some("mcp.json"));
    }

    #[test]
    fn test_parse_provides_without_mcp_config() {
        let toml = r#"
[extension]
name = "no-mcp-ext"
version = "1.0.0"

[provides]
mcp = ["server"]
content_types = ["rules"]
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let provides = manifest.provides.unwrap();
        assert!(provides.mcp_config.is_none());
    }

    // --- Version constraint validation ---

    #[test]
    fn test_invalid_python_constraint_rejected() {
        let toml = r#"
[extension]
name = "bad-constraint"
version = "1.0.0"

[requires.python]
version = ">=abc"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::VersionConstraintParse { .. }),
            "Expected VersionConstraintParse, got: {err:?}"
        );
    }

    #[test]
    fn test_valid_python_constraint_accepted() {
        let toml = r#"
[extension]
name = "good-constraint"
version = "1.0.0"

[requires.python]
version = ">=3.10,<3.14"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let req = manifest.requires.unwrap().python.unwrap();
        assert_eq!(req.version, ">=3.10,<3.14");
    }

    // --- python_version_satisfied ---

    #[test]
    fn test_python_version_satisfied_no_constraint() {
        let toml = r#"
[extension]
name = "no-req"
version = "1.0.0"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert!(manifest.python_version_satisfied("3.8.0").unwrap());
    }

    #[test]
    fn test_python_version_satisfied_passes() {
        let manifest = ExtensionManifest::from_toml(VAULTSPEC_TOML).unwrap();
        // VAULTSPEC_TOML requires >=3.13
        assert!(manifest.python_version_satisfied("3.13.0").unwrap());
        assert!(manifest.python_version_satisfied("3.14.0").unwrap());
    }

    #[test]
    fn test_python_version_satisfied_fails() {
        let manifest = ExtensionManifest::from_toml(VAULTSPEC_TOML).unwrap();
        // VAULTSPEC_TOML requires >=3.13
        assert!(!manifest.python_version_satisfied("3.12.0").unwrap());
    }

    // --- implicit_preset_dependencies ---

    #[test]
    fn test_implicit_deps_python_runtime() {
        let manifest = ExtensionManifest::from_toml(VAULTSPEC_TOML).unwrap();
        let deps = manifest.implicit_preset_dependencies();
        assert!(deps.contains(&"env:python".to_string()));
    }

    #[test]
    fn test_implicit_deps_no_runtime() {
        let toml = r#"
[extension]
name = "minimal"
version = "1.0.0"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert!(manifest.implicit_preset_dependencies().is_empty());
    }

    #[test]
    fn test_implicit_deps_rust_runtime() {
        let toml = r#"
[extension]
name = "rust-ext"
version = "1.0.0"

[runtime]
type = "rust"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let deps = manifest.implicit_preset_dependencies();
        assert_eq!(deps, vec!["env:rust"]);
    }

    // --- package_manager validation (Phase 2 / ADR-008) ---

    #[test]
    fn test_invalid_package_manager_rejected() {
        let toml = r#"
[extension]
name = "bad-pm"
version = "1.0.0"

[runtime]
type = "python"
package_manager = "poetry"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::InvalidPackageManager { ref value } if value == "poetry"),
            "expected InvalidPackageManager, got: {err:?}"
        );
    }

    #[test]
    fn test_known_package_managers_accepted() {
        for pm in &["uv", "pip", "npm", "yarn", "pnpm", "cargo", "bun"] {
            let toml = format!(
                r#"
[extension]
name = "ext"
version = "1.0.0"

[runtime]
type = "python"
package_manager = "{pm}"
"#
            );
            let manifest = ExtensionManifest::from_toml(&toml)
                .unwrap_or_else(|e| panic!("expected {pm} to be accepted, got: {e:?}"));
            assert_eq!(
                manifest.runtime.unwrap().package_manager.as_deref(),
                Some(*pm)
            );
        }
    }

    #[test]
    fn test_package_manager_field_round_trips() {
        let toml = r#"
[extension]
name = "uv-ext"
version = "1.0.0"

[runtime]
type = "python"
package_manager = "uv"
install = "uv sync"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(
            manifest.runtime.as_ref().unwrap().package_manager.as_deref(),
            Some("uv")
        );

        let serialized = manifest.to_toml().unwrap();
        let reparsed = ExtensionManifest::from_toml(&serialized).unwrap();
        assert_eq!(
            reparsed.runtime.as_ref().unwrap().package_manager.as_deref(),
            Some("uv")
        );
    }

    // --- packages validation (Phase 3 / ADR-009) ---

    #[test]
    fn test_packages_metacharacter_rejected() {
        let toml = r#"
[extension]
name = "bad-pkg"
version = "1.0.0"

[requires.python]
version = ">=3.12"
packages = ["httpx; rm -rf /"]
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::InvalidPackages { .. }),
            "Expected InvalidPackages, got: {err:?}"
        );
    }

    #[test]
    fn test_packages_empty_string_rejected() {
        let toml = r#"
[extension]
name = "bad-pkg"
version = "1.0.0"

[requires.python]
version = ">=3.12"
packages = [""]
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::InvalidPackages { ref reason } if reason.contains("empty")),
            "Expected InvalidPackages with 'empty' reason, got: {err:?}"
        );
    }

    #[test]
    fn test_packages_valid_accepted() {
        let toml = r#"
[extension]
name = "good-pkg"
version = "1.0.0"

[requires.python]
version = ">=3.12"
packages = ["httpx>=0.27", "pydantic>=2.0,<3.0"]
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let packages = &manifest.requires.unwrap().python.unwrap().packages;
        assert_eq!(packages, &["httpx>=0.27", "pydantic>=2.0,<3.0"]);
    }

    #[test]
    fn test_packages_default_empty_when_omitted() {
        let toml = r#"
[extension]
name = "no-pkg"
version = "1.0.0"

[requires.python]
version = ">=3.12"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        let packages = &manifest.requires.unwrap().python.unwrap().packages;
        assert!(packages.is_empty());
    }

    // --- venv_path validation (Phase 4 / ADR-010) ---

    #[test]
    fn test_venv_path_absolute_rejected() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[runtime]
type = "python"
venv_path = "/absolute/path"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::InvalidVenvPath { .. }),
            "expected InvalidVenvPath, got: {err:?}"
        );
    }

    #[test]
    fn test_venv_path_parent_dir_rejected() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[runtime]
type = "python"
venv_path = "../escape"
"#;
        let err = ExtensionManifest::from_toml(toml).unwrap_err();
        assert!(
            matches!(err, Error::InvalidVenvPath { .. }),
            "expected InvalidVenvPath, got: {err:?}"
        );
    }

    #[test]
    fn test_venv_path_valid_relative_accepted() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[runtime]
type = "python"
venv_path = ".venv"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(
            manifest.runtime.unwrap().venv_path.as_deref(),
            Some(".venv")
        );
    }

    #[test]
    fn test_venv_path_nested_relative_accepted() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[runtime]
type = "python"
venv_path = ".vaultspec/.venv"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert_eq!(
            manifest.runtime.unwrap().venv_path.as_deref(),
            Some(".vaultspec/.venv")
        );
    }

    #[test]
    fn test_venv_path_default_none_when_omitted() {
        let toml = r#"
[extension]
name = "test"
version = "1.0.0"

[runtime]
type = "python"
"#;
        let manifest = ExtensionManifest::from_toml(toml).unwrap();
        assert!(manifest.runtime.unwrap().venv_path.is_none());
    }
}

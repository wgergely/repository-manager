//! MCP Tool Handlers
//!
//! This module implements the handlers for MCP tool calls, delegating to repo-core
//! for the actual operations.
//!
//! Note: Handler functions use `async fn` for consistency with the MCP server's
//! tokio runtime, even though the current implementations perform synchronous I/O.
//! This allows for future migration to async file operations without API changes.

use std::fs;
use std::path::Path;

use git2::Repository;
use repo_core::{
    CheckStatus, Manifest, Mode, ModeBackend, StandardBackend, SyncEngine, SyncOptions,
    WorktreeBackend,
};
use repo_fs::NormalizedPath;
use repo_git::{ClassicLayout, ContainerLayout, LayoutProvider};
use repo_meta::Registry;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Error, Result};

/// Handle a tool call by dispatching to the appropriate handler
pub async fn handle_tool_call(root: &Path, tool_name: &str, arguments: Value) -> Result<Value> {
    match tool_name {
        // Repository Lifecycle
        "repo_check" => handle_repo_check(root).await,
        "repo_sync" => handle_repo_sync(root, arguments).await,
        "repo_fix" => handle_repo_fix(root, arguments).await,
        "repo_init" => handle_repo_init(root, arguments).await,

        // Branch Management
        "branch_list" => handle_branch_list(root).await,
        "branch_create" => handle_branch_create(root, arguments).await,
        "branch_delete" => handle_branch_delete(root, arguments).await,

        // Git Primitives
        "git_push" => handle_git_push(root, arguments).await,
        "git_pull" => handle_git_pull(root, arguments).await,
        "git_merge" => handle_git_merge(root, arguments).await,


        // Configuration Management
        "tool_add" => handle_tool_add(root, arguments).await,
        "tool_remove" => handle_tool_remove(root, arguments).await,
        "rule_add" => handle_rule_add(root, arguments).await,
        "rule_remove" => handle_rule_remove(root, arguments).await,

        // Preset Management
        "preset_list" => handle_preset_list(root).await,
        "preset_add" => handle_preset_add(root, arguments).await,
        "preset_remove" => handle_preset_remove(root, arguments).await,

        // Extension Management
        "extension_install" => handle_extension_install(arguments).await,
        "extension_add" => handle_extension_add(arguments).await,
        "extension_init" => handle_extension_init(arguments).await,
        "extension_remove" => handle_extension_remove(arguments).await,
        "extension_list" => handle_extension_list().await,

        _ => Err(Error::UnknownTool(tool_name.to_string())),
    }
}

// ============================================================================
// Repository Lifecycle Handlers
// ============================================================================

/// Handle repo_check - Check configuration validity and consistency
async fn handle_repo_check(root: &Path) -> Result<Value> {
    let ctx = RepoContext::new(root)?;
    let engine = ctx.sync_engine()?;
    let report = engine.check().map_err(Error::Core)?;

    Ok(json!({
        "status": format!("{:?}", report.status),
        "healthy": report.status == CheckStatus::Healthy,
        "drifted": report.drifted.len(),
        "missing": report.missing.len(),
        "details": {
            "drifted": report.drifted.iter().map(|d| json!({
                "intent_id": d.intent_id,
                "tool": d.tool,
                "file": d.file,
                "description": d.description,
            })).collect::<Vec<_>>(),
            "missing": report.missing.iter().map(|m| json!({
                "intent_id": m.intent_id,
                "tool": m.tool,
                "file": m.file,
                "description": m.description,
            })).collect::<Vec<_>>(),
            "messages": report.messages,
        }
    }))
}

/// Handle repo_sync - Regenerate tool configurations from rules
async fn handle_repo_sync(root: &Path, arguments: Value) -> Result<Value> {
    let ctx = RepoContext::new(root)?;
    let engine = ctx.sync_engine()?;

    let dry_run = arguments
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let options = SyncOptions { dry_run };
    let report = engine.sync_with_options(options).map_err(Error::Core)?;

    Ok(json!({
        "success": report.success,
        "dry_run": dry_run,
        "actions": report.actions,
        "errors": report.errors,
    }))
}

/// Handle repo_fix - Repair configuration inconsistencies
async fn handle_repo_fix(root: &Path, arguments: Value) -> Result<Value> {
    let ctx = RepoContext::new(root)?;
    let engine = ctx.sync_engine()?;

    let dry_run = arguments
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let options = SyncOptions { dry_run };
    let report = engine.fix_with_options(options).map_err(Error::Core)?;

    Ok(json!({
        "success": report.success,
        "dry_run": dry_run,
        "actions": report.actions,
        "errors": report.errors,
    }))
}

/// Arguments for repo_init
#[derive(Debug, Deserialize)]
struct RepoInitArgs {
    name: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    tools: Option<Vec<String>>,
    #[serde(default)]
    extensions: Option<Vec<String>>,
}

/// Handle repo_init - Initialize a new repository configuration
async fn handle_repo_init(root: &Path, arguments: Value) -> Result<Value> {
    let args: RepoInitArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);

    // Determine mode
    let mode_str = args.mode.as_deref().unwrap_or("standard");
    let mode: Mode = mode_str
        .parse()
        .map_err(|_| Error::InvalidArgument(format!("Invalid mode: {}", mode_str)))?;

    // Check if .repository already exists
    let repo_dir = normalized_root.join(".repository");
    if repo_dir.exists() {
        return Ok(json!({
            "success": false,
            "message": "Repository already initialized (.repository directory exists)",
        }));
    }

    // Create .repository directory
    fs::create_dir_all(repo_dir.as_ref())?;

    // Create config.toml
    let tools = args.tools.unwrap_or_default();
    let extensions = args.extensions.unwrap_or_default();
    // Escape all user-supplied values for safe TOML interpolation
    let escape_toml = |s: &str| -> String {
        let mut escaped = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                c if c.is_control() => {}
                c => escaped.push(c),
            }
        }
        escaped
    };

    let tools_toml = if tools.is_empty() {
        "tools = []".to_string()
    } else {
        let escaped: Vec<String> = tools
            .iter()
            .map(|t| format!("\"{}\"", escape_toml(t)))
            .collect();
        format!("tools = [{}]", escaped.join(", "))
    };

    let escaped_name = escape_toml(&args.name);
    let mut config_content = format!(
        r#"# Repository Manager Configuration
# Project: {}

{}

[core]
mode = "{}"
"#,
        escaped_name, tools_toml, mode
    );

    // Add extensions sections
    if !extensions.is_empty() {
        use repo_extensions::ExtensionRegistry;
        let registry = ExtensionRegistry::with_known();
        for ext in &extensions {
            let escaped_ext = escape_toml(ext);
            config_content.push('\n');
            if let Some(entry) = registry.get(ext) {
                config_content.push_str(&format!(
                    "[extensions.\"{}\"]\nsource = \"{}\"\nref = \"main\"\n",
                    escaped_ext,
                    escape_toml(&entry.source)
                ));
            } else {
                config_content.push_str(&format!(
                    "[extensions.\"{}\"]\nsource = \"{}\"\nref = \"main\"\n",
                    escaped_ext, escaped_ext
                ));
            }
        }
    }

    let config_path = repo_dir.join("config.toml");
    fs::write(config_path.as_ref(), &config_content)?;

    // Create rules directory
    let rules_dir = repo_dir.join("rules");
    fs::create_dir_all(rules_dir.as_ref())?;

    Ok(json!({
        "success": true,
        "message": format!("Initialized repository '{}' in {} mode", args.name, mode),
        "config_path": config_path.as_str(),
    }))
}

// ============================================================================
// Branch Management Handlers
// ============================================================================

/// Handle branch_list - List active branches
async fn handle_branch_list(root: &Path) -> Result<Value> {
    let ctx = RepoContext::new(root)?;
    let backend = ctx.backend()?;
    let branches = backend.list_branches().map_err(Error::Core)?;

    let branch_data: Vec<Value> = branches
        .iter()
        .map(|b| {
            json!({
                "name": b.name,
                "path": b.path.as_ref().map(|p| p.as_str().to_string()),
                "is_current": b.is_current,
                "is_main": b.is_main,
            })
        })
        .collect();

    Ok(json!({
        "branches": branch_data,
        "count": branches.len(),
    }))
}

/// Arguments for branch_create
#[derive(Debug, Deserialize)]
struct BranchCreateArgs {
    name: String,
    #[serde(default)]
    base: Option<String>,
}

/// Validate a branch name for safety.
///
/// Rejects names that could be interpreted as git flags, contain path traversal
/// sequences, null bytes, or other dangerous characters.
fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::InvalidArgument(
            "Branch name must not be empty".to_string(),
        ));
    }
    if name.starts_with('-') {
        return Err(Error::InvalidArgument(
            "Branch name must not start with '-' (would be interpreted as a git flag)".to_string(),
        ));
    }
    if name.contains('\0') {
        return Err(Error::InvalidArgument(
            "Branch name must not contain null bytes".to_string(),
        ));
    }
    if name.contains("..") {
        return Err(Error::InvalidArgument(
            "Branch name must not contain '..' (path traversal)".to_string(),
        ));
    }
    if name.len() > 255 {
        return Err(Error::InvalidArgument(
            "Branch name exceeds maximum length of 255 characters".to_string(),
        ));
    }
    // Git ref restrictions: no space, ~, ^, :, ?, *, [, \, control chars
    let invalid_chars = [' ', '~', '^', ':', '?', '*', '[', '\\'];
    for ch in &invalid_chars {
        if name.contains(*ch) {
            return Err(Error::InvalidArgument(format!(
                "Branch name contains invalid character '{}'",
                ch
            )));
        }
    }
    if name.ends_with('/') || name.ends_with('.') || name.ends_with(".lock") {
        return Err(Error::InvalidArgument(
            "Branch name must not end with '/', '.', or '.lock'".to_string(),
        ));
    }
    Ok(())
}

/// Handle branch_create - Create a new branch (with worktree in worktrees mode)
async fn handle_branch_create(root: &Path, arguments: Value) -> Result<Value> {
    let args: BranchCreateArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    // Validate branch names before passing to git
    validate_branch_name(&args.name)?;
    if let Some(ref base) = args.base {
        validate_branch_name(base)?;
    }

    let ctx = RepoContext::new(root)?;
    let backend = ctx.backend()?;

    backend
        .create_branch(&args.name, args.base.as_deref())
        .map_err(Error::Core)?;

    let path = if ctx.mode == Mode::Worktrees {
        // In worktree mode, return the worktree path
        // The worktree is created in the container, which is the parent of root
        // if root is a worktree, or root itself if it's the container
        let container = find_container(&ctx.root)?;
        Some(container.join(&args.name))
    } else {
        None
    };

    Ok(json!({
        "success": true,
        "branch": args.name,
        "base": args.base,
        "path": path.as_ref().map(|p| p.as_str().to_string()),
        "message": format!("Created branch '{}'", args.name),
    }))
}

/// Arguments for branch_delete
#[derive(Debug, Deserialize)]
struct BranchDeleteArgs {
    name: String,
}

/// Handle branch_delete - Remove a branch and its worktree
async fn handle_branch_delete(root: &Path, arguments: Value) -> Result<Value> {
    let args: BranchDeleteArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    // Validate branch name before passing to git
    validate_branch_name(&args.name)?;

    let ctx = RepoContext::new(root)?;
    let backend = ctx.backend()?;

    backend.delete_branch(&args.name).map_err(Error::Core)?;

    Ok(json!({
        "success": true,
        "branch": args.name,
        "message": format!("Deleted branch '{}'", args.name),
    }))
}

// ============================================================================
// Git Primitive Handlers
// ============================================================================

/// Arguments for git_push
#[derive(Debug, Deserialize)]
struct GitPushArgs {
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    branch: Option<String>,
}

/// Handle git_push - Push current branch to remote
async fn handle_git_push(root: &Path, arguments: Value) -> Result<Value> {
    let args: GitPushArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let ctx = RepoContext::new(root)?;
    let provider = create_git_provider(&ctx.root, ctx.mode)?;
    let repo = Repository::open(provider.main_worktree().to_native())
        .map_err(repo_git::Error::from)?;

    let remote_name = args.remote.as_deref().unwrap_or("origin");
    let branch_ref = args.branch.as_deref();

    // Resolve the branch name for reporting before the closure consumes provider
    let pushed_branch = match &args.branch {
        Some(b) => b.clone(),
        None => provider
            .current_branch()
            .unwrap_or_else(|_| "unknown".to_string()),
    };

    let current_branch_fn = || provider.current_branch();
    repo_git::push(&repo, Some(remote_name), branch_ref, current_branch_fn)?;

    Ok(json!({
        "success": true,
        "remote": remote_name,
        "branch": pushed_branch,
        "message": format!("Pushed '{}' to '{}'", pushed_branch, remote_name),
    }))
}

/// Arguments for git_pull
#[derive(Debug, Deserialize)]
struct GitPullArgs {
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    branch: Option<String>,
}

/// Handle git_pull - Pull changes from remote
async fn handle_git_pull(root: &Path, arguments: Value) -> Result<Value> {
    let args: GitPullArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let ctx = RepoContext::new(root)?;
    let provider = create_git_provider(&ctx.root, ctx.mode)?;
    let repo = Repository::open(provider.main_worktree().to_native())
        .map_err(repo_git::Error::from)?;

    let remote_name = args.remote.as_deref().unwrap_or("origin");
    let branch_ref = args.branch.as_deref();

    // Resolve the branch name for reporting before the closure consumes provider
    let pulled_branch = match &args.branch {
        Some(b) => b.clone(),
        None => provider
            .current_branch()
            .unwrap_or_else(|_| "unknown".to_string()),
    };

    let current_branch_fn = || provider.current_branch();
    repo_git::pull(&repo, Some(remote_name), branch_ref, current_branch_fn, None)?;

    Ok(json!({
        "success": true,
        "remote": remote_name,
        "branch": pulled_branch,
        "message": format!("Pulled '{}' from '{}'", pulled_branch, remote_name),
    }))
}

/// Arguments for git_merge
#[derive(Debug, Deserialize)]
struct GitMergeArgs {
    source: String,
}

/// Handle git_merge - Merge a branch into the current branch
async fn handle_git_merge(root: &Path, arguments: Value) -> Result<Value> {
    let args: GitMergeArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let ctx = RepoContext::new(root)?;
    let provider = create_git_provider(&ctx.root, ctx.mode)?;
    let repo = Repository::open(provider.main_worktree().to_native())
        .map_err(repo_git::Error::from)?;

    let current_branch_fn = || provider.current_branch();
    repo_git::merge(&repo, &args.source, current_branch_fn, None)?;

    Ok(json!({
        "success": true,
        "source": args.source,
        "message": format!("Merged '{}' into current branch", args.source),
    }))
}

/// Create a LayoutProvider for git operations based on detected mode.
fn create_git_provider(
    root: &NormalizedPath,
    mode: Mode,
) -> Result<Box<dyn LayoutProvider>> {
    match mode {
        Mode::Standard => {
            let layout = ClassicLayout::new(root.clone())?;
            Ok(Box::new(layout))
        }
        Mode::Worktrees => {
            let layout = ContainerLayout::new(root.clone(), Default::default())?;
            Ok(Box::new(layout))
        }
    }
}

// ============================================================================
// Configuration Management Handlers
// ============================================================================

/// Arguments for tool_add
#[derive(Debug, Deserialize)]
struct ToolAddArgs {
    name: String,
}

/// Handle tool_add - Enable a tool for this repository
async fn handle_tool_add(root: &Path, arguments: Value) -> Result<Value> {
    let args: ToolAddArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let config_path = find_config_path(&normalized_root)?;

    // Read existing config
    let content = fs::read_to_string(config_path.as_ref())?;
    let mut manifest: Manifest = toml::from_str(&content)?;

    // Check if tool already exists
    if manifest.tools.contains(&args.name) {
        return Ok(json!({
            "success": false,
            "message": format!("Tool '{}' is already enabled", args.name),
        }));
    }

    // Add the tool
    manifest.tools.push(args.name.clone());

    // Serialize and write back
    let new_content = serialize_manifest(&manifest)?;
    fs::write(config_path.as_ref(), &new_content)?;

    Ok(json!({
        "success": true,
        "tool": args.name,
        "message": format!("Enabled tool '{}'", args.name),
    }))
}

/// Arguments for tool_remove
#[derive(Debug, Deserialize)]
struct ToolRemoveArgs {
    name: String,
}

/// Handle tool_remove - Disable a tool for this repository
async fn handle_tool_remove(root: &Path, arguments: Value) -> Result<Value> {
    let args: ToolRemoveArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let config_path = find_config_path(&normalized_root)?;

    // Read existing config
    let content = fs::read_to_string(config_path.as_ref())?;
    let mut manifest: Manifest = toml::from_str(&content)?;

    // Check if tool exists
    if !manifest.tools.contains(&args.name) {
        return Ok(json!({
            "success": false,
            "message": format!("Tool '{}' is not enabled", args.name),
        }));
    }

    // Remove the tool
    manifest.tools.retain(|t| t != &args.name);

    // Serialize and write back
    let new_content = serialize_manifest(&manifest)?;
    fs::write(config_path.as_ref(), &new_content)?;

    Ok(json!({
        "success": true,
        "tool": args.name,
        "message": format!("Disabled tool '{}'", args.name),
    }))
}

/// Arguments for rule_add
#[derive(Debug, Deserialize)]
struct RuleAddArgs {
    id: String,
    content: String,
}

/// Handle rule_add - Add a custom rule to the repository
async fn handle_rule_add(root: &Path, arguments: Value) -> Result<Value> {
    let args: RuleAddArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let rules_dir = find_rules_dir(&normalized_root)?;

    // Validate rule ID
    repo_core::validate_rule_id(&args.id)
        .map_err(|e| Error::InvalidArgument(e.to_string()))?;

    // Create the rule file
    let rule_path = rules_dir.join(&format!("{}.md", args.id));

    // Check if rule already exists
    if rule_path.exists() {
        return Ok(json!({
            "success": false,
            "message": format!("Rule '{}' already exists", args.id),
        }));
    }

    // Ensure rules directory exists
    fs::create_dir_all(rules_dir.as_ref())?;

    // Write the rule file
    fs::write(rule_path.as_ref(), &args.content)?;

    Ok(json!({
        "success": true,
        "rule": args.id,
        "path": rule_path.as_str(),
        "message": format!("Created rule '{}'", args.id),
    }))
}

/// Arguments for rule_remove
#[derive(Debug, Deserialize)]
struct RuleRemoveArgs {
    id: String,
}

/// Handle rule_remove - Delete a rule from the repository
async fn handle_rule_remove(root: &Path, arguments: Value) -> Result<Value> {
    let args: RuleRemoveArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    // Validate rule ID
    repo_core::validate_rule_id(&args.id)
        .map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let rules_dir = find_rules_dir(&normalized_root)?;

    // Find the rule file
    let rule_path = rules_dir.join(&format!("{}.md", args.id));

    // Check if rule exists
    if !rule_path.exists() {
        return Ok(json!({
            "success": false,
            "message": format!("Rule '{}' does not exist", args.id),
        }));
    }

    // Remove the rule file
    fs::remove_file(rule_path.as_ref())?;

    Ok(json!({
        "success": true,
        "rule": args.id,
        "message": format!("Removed rule '{}'", args.id),
    }))
}

// ============================================================================
// Preset Management Handlers
// ============================================================================

/// Handle preset_list - List configured presets and available preset types
async fn handle_preset_list(root: &Path) -> Result<Value> {
    let normalized_root = NormalizedPath::new(root);
    let config_path = find_config_path(&normalized_root)?;

    let content = fs::read_to_string(config_path.as_ref())?;
    let manifest: Manifest = toml::from_str(&content)?;

    let configured: Vec<Value> = manifest
        .presets
        .iter()
        .map(|(name, config)| {
            json!({
                "name": name,
                "config": config,
            })
        })
        .collect();

    let registry = Registry::with_builtins();
    let available = registry.list_presets();

    Ok(json!({
        "configured": configured,
        "configured_count": manifest.presets.len(),
        "available": available,
    }))
}

/// Arguments for preset_add
#[derive(Debug, Deserialize)]
struct PresetAddArgs {
    name: String,
}

/// Handle preset_add - Add a preset to the repository configuration
async fn handle_preset_add(root: &Path, arguments: Value) -> Result<Value> {
    let args: PresetAddArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let config_path = find_config_path(&normalized_root)?;

    let content = fs::read_to_string(config_path.as_ref())?;
    let mut manifest: Manifest = toml::from_str(&content)?;

    // Check if preset already exists
    if manifest.presets.contains_key(&args.name) {
        return Ok(json!({
            "success": false,
            "message": format!("Preset '{}' is already configured", args.name),
        }));
    }

    // Add the preset with an empty config
    manifest.presets.insert(args.name.clone(), json!({}));

    let new_content = serialize_manifest(&manifest)?;
    fs::write(config_path.as_ref(), &new_content)?;

    Ok(json!({
        "success": true,
        "preset": args.name,
        "message": format!("Added preset '{}'", args.name),
    }))
}

/// Arguments for preset_remove
#[derive(Debug, Deserialize)]
struct PresetRemoveArgs {
    name: String,
}

/// Handle preset_remove - Remove a preset from the repository configuration
async fn handle_preset_remove(root: &Path, arguments: Value) -> Result<Value> {
    let args: PresetRemoveArgs =
        serde_json::from_value(arguments).map_err(|e| Error::InvalidArgument(e.to_string()))?;

    let normalized_root = NormalizedPath::new(root);
    let config_path = find_config_path(&normalized_root)?;

    let content = fs::read_to_string(config_path.as_ref())?;
    let mut manifest: Manifest = toml::from_str(&content)?;

    if manifest.presets.remove(&args.name).is_none() {
        return Ok(json!({
            "success": false,
            "message": format!("Preset '{}' is not configured", args.name),
        }));
    }

    let new_content = serialize_manifest(&manifest)?;
    fs::write(config_path.as_ref(), &new_content)?;

    Ok(json!({
        "success": true,
        "preset": args.name,
        "message": format!("Removed preset '{}'", args.name),
    }))
}

// ============================================================================
// Extension Management Handlers
// ============================================================================

/// Handle extension_install - Install an extension from a URL or local path
async fn handle_extension_install(_arguments: Value) -> Result<Value> {
    Err(Error::NotImplemented("extension_install".to_string()))
}

/// Handle extension_add - Add a known extension by name from the registry
async fn handle_extension_add(_arguments: Value) -> Result<Value> {
    Err(Error::NotImplemented("extension_add".to_string()))
}

/// Handle extension_init - Initialize a new extension scaffold
async fn handle_extension_init(_arguments: Value) -> Result<Value> {
    Err(Error::NotImplemented("extension_init".to_string()))
}

/// Handle extension_remove - Remove an installed extension
async fn handle_extension_remove(_arguments: Value) -> Result<Value> {
    Err(Error::NotImplemented("extension_remove".to_string()))
}

/// Handle extension_list - List installed and known extensions
async fn handle_extension_list() -> Result<Value> {
    use repo_extensions::ExtensionRegistry;

    let registry = ExtensionRegistry::with_known();
    let known: Vec<Value> = registry
        .known_extensions()
        .iter()
        .filter_map(|name| {
            registry.get(name).map(|entry| {
                json!({
                    "name": entry.name,
                    "description": entry.description,
                    "source": entry.source,
                })
            })
        })
        .collect();

    Ok(json!({
        "known": known,
        "known_count": known.len(),
        "installed": [],
        "installed_count": 0,
        "message": "Extension list (installed extensions not yet tracked)",
    }))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Repository context with mode and normalized root path.
/// This reduces duplication in handlers that need mode detection.
struct RepoContext {
    root: NormalizedPath,
    mode: Mode,
}

impl RepoContext {
    /// Create a new repository context from a path
    fn new(path: &Path) -> Result<Self> {
        let root = NormalizedPath::new(path);
        let mode = detect_mode(&root)?;
        Ok(Self { root, mode })
    }

    /// Create a SyncEngine for this repository
    fn sync_engine(&self) -> Result<SyncEngine> {
        SyncEngine::new(self.root.clone(), self.mode).map_err(Error::Core)
    }

    /// Create a ModeBackend for this repository
    fn backend(&self) -> Result<Box<dyn ModeBackend>> {
        create_backend(&self.root, self.mode)
    }
}

/// Detect the repository mode from filesystem markers and configuration.
///
/// Delegates to [`repo_core::detect_mode`] which checks filesystem markers
/// (`.gt`, `.git`) and falls back to `.repository/config.toml` via ConfigResolver.
/// Defaults to Standard mode when no indicators are found.
fn detect_mode(root: &NormalizedPath) -> Result<Mode> {
    repo_core::detect_mode(root).map_err(Error::Core)
}

/// Create the appropriate backend for the detected mode
fn create_backend(root: &NormalizedPath, mode: Mode) -> Result<Box<dyn ModeBackend>> {
    match mode {
        Mode::Standard => {
            let backend = StandardBackend::new(root.clone()).map_err(Error::Core)?;
            Ok(Box::new(backend))
        }
        Mode::Worktrees => {
            // For worktrees, we need to find the container
            let container = find_container(root)?;
            let backend = WorktreeBackend::new(container).map_err(Error::Core)?;
            Ok(Box::new(backend))
        }
    }
}

/// Find the container directory for a worktree setup
fn find_container(root: &NormalizedPath) -> Result<NormalizedPath> {
    // If root contains .gt, it's the container
    if root.join(".gt").exists() {
        return Ok(root.clone());
    }

    // Otherwise, check parent directory
    if let Some(parent) = root.as_ref().parent() {
        let parent_path = NormalizedPath::new(parent);
        if parent_path.join(".gt").exists() {
            return Ok(parent_path);
        }
    }

    // Not a worktree container
    Err(Error::InvalidArgument(
        "Not a worktree container: .gt not found".to_string(),
    ))
}

/// Find the config.toml path
fn find_config_path(root: &NormalizedPath) -> Result<NormalizedPath> {
    // Check .repository/config.toml in root
    let config_path = root.join(".repository/config.toml");
    if config_path.exists() {
        return Ok(config_path);
    }

    // Check parent for worktree mode
    if let Some(parent) = root.as_ref().parent() {
        let parent_path = NormalizedPath::new(parent);
        let config_path = parent_path.join(".repository/config.toml");
        if config_path.exists() {
            return Ok(config_path);
        }
    }

    Err(Error::InvalidArgument(
        "Config file not found (.repository/config.toml)".to_string(),
    ))
}

/// Find the rules directory
fn find_rules_dir(root: &NormalizedPath) -> Result<NormalizedPath> {
    // Check .repository/rules in root
    let rules_dir = root.join(".repository/rules");
    if rules_dir.exists() || root.join(".repository").exists() {
        return Ok(rules_dir);
    }

    // Check parent for worktree mode
    if let Some(parent) = root.as_ref().parent() {
        let parent_path = NormalizedPath::new(parent);
        let rules_dir = parent_path.join(".repository/rules");
        if rules_dir.exists() || parent_path.join(".repository").exists() {
            return Ok(rules_dir);
        }
    }

    Err(Error::InvalidArgument(
        "Repository not initialized (.repository not found)".to_string(),
    ))
}

/// Serialize a manifest back to TOML format
fn serialize_manifest(manifest: &Manifest) -> Result<String> {
    Ok(manifest.to_toml())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo(dir: &std::path::Path) {
        fs::create_dir_all(dir.join(".git")).unwrap();
        fs::create_dir_all(dir.join(".repository")).unwrap();
        fs::write(
            dir.join(".repository/config.toml"),
            "tools = []\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_handle_repo_check() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(temp.path(), "repo_check", json!({})).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert!(value.get("healthy").is_some());
    }

    #[tokio::test]
    async fn test_handle_repo_sync() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(temp.path(), "repo_sync", json!({"dry_run": true})).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("dry_run"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn test_handle_repo_fix() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(temp.path(), "repo_fix", json!({"dry_run": true})).await;
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.get("dry_run"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn test_handle_repo_init() {
        let temp = TempDir::new().unwrap();
        // Create a minimal git repo without .repository
        fs::create_dir_all(temp.path().join(".git")).unwrap();

        let result = handle_tool_call(
            temp.path(),
            "repo_init",
            json!({
                "name": "test-project",
                "mode": "standard",
                "tools": ["claude", "vscode"]
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify config was created
        assert!(temp.path().join(".repository/config.toml").exists());
    }

    #[tokio::test]
    async fn test_handle_repo_init_with_extensions() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git")).unwrap();

        let result = handle_tool_call(
            temp.path(),
            "repo_init",
            json!({
                "name": "ext-project",
                "mode": "standard",
                "extensions": ["vaultspec"]
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify extensions section was written to config
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        assert!(content.contains("[extensions.\"vaultspec\"]"));
        assert!(content.contains("source = \"https://github.com/vaultspec/vaultspec.git\""));
        assert!(content.contains("ref = \"main\""));
    }

    #[tokio::test]
    async fn test_handle_repo_init_already_initialized() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(
            temp.path(),
            "repo_init",
            json!({
                "name": "test-project"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(false)));
    }

    #[tokio::test]
    async fn test_handle_unknown_tool() {
        let temp = TempDir::new().unwrap();
        let result = handle_tool_call(temp.path(), "unknown_tool", json!({})).await;
        assert!(result.is_err());
        match result {
            Err(Error::UnknownTool(name)) => assert_eq!(name, "unknown_tool"),
            _ => panic!("Expected UnknownTool error"),
        }
    }

    #[tokio::test]
    async fn test_git_handlers_no_longer_return_not_implemented() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        // Initialize a real git repo so the handlers can proceed past mode detection
        Repository::init(temp.path()).unwrap();

        for tool in &["git_push", "git_pull", "git_merge"] {
            let args = if *tool == "git_merge" {
                json!({"source": "nonexistent-branch"})
            } else {
                json!({})
            };
            let result = handle_tool_call(temp.path(), tool, args).await;
            // The handlers should NOT return NotImplemented anymore.
            // They may return other errors (e.g., no remote, no branch) but
            // the key assertion is that NotImplemented is gone.
            match &result {
                Err(Error::NotImplemented(name)) => {
                    panic!("{} still returns NotImplemented - it should be implemented now", name);
                }
                _ => {} // Any other result (Ok or non-NotImplemented Err) is acceptable
            }
        }
    }

    #[tokio::test]
    async fn test_handle_tool_add() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(
            temp.path(),
            "tool_add",
            json!({
                "name": "vscode"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify tool was added
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        assert!(content.contains("vscode"));
    }

    #[tokio::test]
    async fn test_handle_tool_add_duplicate() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git")).unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        fs::write(
            temp.path().join(".repository/config.toml"),
            "tools = [\"vscode\"]\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        let result = handle_tool_call(
            temp.path(),
            "tool_add",
            json!({
                "name": "vscode"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(false)));
    }

    #[tokio::test]
    async fn test_handle_tool_remove() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git")).unwrap();
        fs::create_dir_all(temp.path().join(".repository")).unwrap();
        fs::write(
            temp.path().join(".repository/config.toml"),
            "tools = [\"vscode\", \"cursor\"]\n\n[core]\nmode = \"standard\"\n",
        )
        .unwrap();

        let result = handle_tool_call(
            temp.path(),
            "tool_remove",
            json!({
                "name": "vscode"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify tool was removed
        let content = fs::read_to_string(temp.path().join(".repository/config.toml")).unwrap();
        assert!(!content.contains("vscode"));
        assert!(content.contains("cursor"));
    }

    #[tokio::test]
    async fn test_handle_rule_add() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(
            temp.path(),
            "rule_add",
            json!({
                "id": "no-unsafe",
                "content": "Do not use unsafe code blocks."
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify rule file was created
        let rule_path = temp.path().join(".repository/rules/no-unsafe.md");
        assert!(rule_path.exists());
        let content = fs::read_to_string(&rule_path).unwrap();
        assert_eq!(content, "Do not use unsafe code blocks.");
    }

    #[tokio::test]
    async fn test_handle_rule_add_invalid_id() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        let result = handle_tool_call(
            temp.path(),
            "rule_add",
            json!({
                "id": "invalid/rule",
                "content": "This should fail."
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_rule_remove() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());

        // First create the rule
        fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();
        fs::write(
            temp.path().join(".repository/rules/test-rule.md"),
            "Test rule content",
        )
        .unwrap();

        let result = handle_tool_call(
            temp.path(),
            "rule_remove",
            json!({
                "id": "test-rule"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(true)));

        // Verify rule was removed
        assert!(!temp.path().join(".repository/rules/test-rule.md").exists());
    }

    #[tokio::test]
    async fn test_handle_rule_remove_not_found() {
        let temp = TempDir::new().unwrap();
        create_test_repo(temp.path());
        fs::create_dir_all(temp.path().join(".repository/rules")).unwrap();

        let result = handle_tool_call(
            temp.path(),
            "rule_remove",
            json!({
                "id": "nonexistent"
            }),
        )
        .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.get("success"), Some(&json!(false)));
    }

    #[test]
    fn test_detect_mode_standard() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".git")).unwrap();

        let root = NormalizedPath::new(temp.path());
        let mode = detect_mode(&root).unwrap();
        assert_eq!(mode, Mode::Standard);
    }

    #[test]
    fn test_detect_mode_worktrees() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".gt")).unwrap();

        let root = NormalizedPath::new(temp.path());
        let mode = detect_mode(&root).unwrap();
        assert_eq!(mode, Mode::Worktrees);
    }

    #[test]
    fn test_serialize_manifest() {
        // Parse a manifest from TOML to get proper CoreSection
        let manifest: Manifest = toml::from_str(
            r#"
            tools = ["vscode", "cursor"]

            [core]
            mode = "standard"
            "#,
        )
        .unwrap();

        let result = serialize_manifest(&manifest).unwrap();
        // toml::to_string_pretty may format arrays multi-line
        assert!(result.contains("vscode"));
        assert!(result.contains("cursor"));
        assert!(result.contains("[core]"));
        assert!(result.contains("mode = \"standard\""));
    }

    #[test]
    fn test_json_to_toml_value() {
        use repo_core::json_to_toml_value;
        assert_eq!(json_to_toml_value(&json!("hello")), "\"hello\"");
        assert_eq!(json_to_toml_value(&json!(42)), "42");
        assert_eq!(json_to_toml_value(&json!(true)), "true");
        assert_eq!(json_to_toml_value(&json!([1, 2, 3])), "[1, 2, 3]");
    }

    #[tokio::test]
    async fn test_handle_extension_install_returns_not_implemented() {
        let temp = TempDir::new().unwrap();
        let result = handle_tool_call(
            temp.path(),
            "extension_install",
            json!({ "source": "https://github.com/example/ext.git" }),
        )
        .await;

        assert!(result.is_err(), "extension_install must return an error");
        match result {
            Err(Error::NotImplemented(name)) => {
                assert_eq!(name, "extension_install");
            }
            other => panic!(
                "Expected NotImplemented error for extension_install, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_handle_extension_add_returns_not_implemented() {
        let temp = TempDir::new().unwrap();
        let result =
            handle_tool_call(temp.path(), "extension_add", json!({ "name": "vaultspec" })).await;

        assert!(result.is_err(), "extension_add must return an error");
        match result {
            Err(Error::NotImplemented(name)) => {
                assert_eq!(name, "extension_add");
            }
            other => panic!(
                "Expected NotImplemented error for extension_add, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_handle_extension_init_returns_not_implemented() {
        let temp = TempDir::new().unwrap();
        let result =
            handle_tool_call(temp.path(), "extension_init", json!({ "name": "my-ext" })).await;

        assert!(result.is_err(), "extension_init must return an error");
        match result {
            Err(Error::NotImplemented(name)) => {
                assert_eq!(name, "extension_init");
            }
            other => panic!(
                "Expected NotImplemented error for extension_init, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_handle_extension_remove_returns_not_implemented() {
        let temp = TempDir::new().unwrap();
        let result =
            handle_tool_call(temp.path(), "extension_remove", json!({ "name": "my-ext" })).await;

        assert!(result.is_err(), "extension_remove must return an error");
        match result {
            Err(Error::NotImplemented(name)) => {
                assert_eq!(name, "extension_remove");
            }
            other => panic!(
                "Expected NotImplemented error for extension_remove, got: {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn test_extension_handlers_return_not_implemented() {
        // Extension handlers should still return NotImplemented
        let temp = TempDir::new().unwrap();

        let extension_tools = ["extension_install", "extension_add", "extension_init", "extension_remove"];

        for tool in extension_tools.iter() {
            let result = handle_tool_call(temp.path(), tool, json!({})).await;
            assert!(result.is_err(), "{} must return an error", tool);
            match result {
                Err(Error::NotImplemented(_)) => {}
                other => panic!(
                    "Expected NotImplemented error for {}, got: {:?}",
                    tool, other
                ),
            }
        }
    }

    #[tokio::test]
    async fn test_handle_extension_list() {
        let temp = TempDir::new().unwrap();
        let result = handle_tool_call(temp.path(), "extension_list", json!({})).await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.get("known").is_some());
        assert!(value.get("known_count").is_some());
        // Should contain at least the "vaultspec" known extension
        let known = value.get("known").unwrap().as_array().unwrap();
        assert!(
            known
                .iter()
                .any(|e| e.get("name") == Some(&json!("vaultspec")))
        );
    }
}

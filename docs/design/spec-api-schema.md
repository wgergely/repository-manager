# Agentic Repository Orchestrator API Schema

*Design Document: Unified Tool Configuration Management*
*Date: 2026-01-23*

## Executive Summary

API schema for `repo`, a CLI orchestrating configuration across agentic coding tools (Claude, Cursor, Windsurf, Gemini, Copilot).

**Design Options Under Consideration:**

1. **Configuration format** - TOML, YAML, and JSON each have trade-offs (see Section 10.1)
2. **Rules/skills format** - Markdown aligns with current industry patterns
3. **Validation approach** - JSON Schema is one option for CI/CD validation and editor support
4. **Provider abstraction** - One approach separates central config from tool-specific translation
5. **Plugin architecture** - Could be extensible through hooks and manifest format

---

## 1. Core Commands API

Core commands: init, manage rules/skills, sync to providers. Design uses provider-agnostic abstractions translated to tool-specific formats during sync.

---

## 2. Configuration Schema

Central config handles: project metadata, provider settings, rules/skills, plugins, permissions. TOML/YAML enables manual editing and complex structures.

---

## 3. Rule Definitions

### 3.1 Rule File Format (Markdown)

Rules are stored as Markdown files with optional YAML frontmatter for metadata.

```markdown
---
# .repository/rules/coding-standards.md
id: coding-standards
name: Coding Standards
version: 1.0.0
priority: 50
scope: all
providers: []  # Empty = all providers
tags:
  - style
  - typescript
  - python
conditions:
  file_patterns:
    - "**/*.ts"
    - "**/*.tsx"
    - "**/*.py"
---

# Coding Standards

## TypeScript Guidelines

### Naming Conventions
- Use `camelCase` for variables and functions
- Use `PascalCase` for types, interfaces, and classes
- Use `UPPER_SNAKE_CASE` for constants
- Prefix interfaces with `I` only when necessary for clarity

### Type Safety
- Always use strict TypeScript (`"strict": true`)
- Avoid `any` type - use `unknown` with type guards instead
- Prefer type inference where the type is obvious
- Use explicit return types for exported functions

### Code Organization
- One component/class per file
- Group imports: external, internal, relative
- Use barrel exports (`index.ts`) for module boundaries

## Python Guidelines

### Style
- Follow PEP 8
- Use type hints for all function signatures
- Maximum line length: 88 characters (Black default)

### Structure
- Use dataclasses for data containers
- Prefer composition over inheritance
- Document public APIs with docstrings

## Universal Guidelines

### Comments
- Write self-documenting code
- Comment "why", not "what"
- Keep comments up to date or remove them

### Testing
- Write tests for all new features
- Aim for 80% code coverage minimum
- Use descriptive test names
```

### 3.2 Rule Entity Data Model

```typescript
interface Rule {
  // Identity
  id: string;                           // Unique identifier (derived from filename)
  name: string;                         // Human-readable name
  version: string;                      // Semantic version

  // Content
  content: string;                      // Markdown content (after frontmatter)
  checksum: string;                     // SHA-256 of content for change detection

  // Scope & Targeting
  scope: 'global' | 'project' | 'local';
  providers: ProviderType[];            // Empty array = all providers
  conditions: RuleConditions;

  // Behavior
  priority: number;                     // 1-100, higher = applied later
  enabled: boolean;

  // Metadata
  tags: string[];
  created_at: string;                   // ISO 8601
  updated_at: string;
  source_path: string;                  // Relative path to .md file
}

interface RuleConditions {
  file_patterns?: string[];             // Glob patterns for file-specific rules
  branch_patterns?: string[];           // Git branch patterns
  environment?: string[];               // e.g., ['development', 'production']
}

type ProviderType = 'claude' | 'cursor' | 'copilot' | 'gemini' | 'windsurf';
```

---

## 4. Skill Definitions

### 4.1 Skill File Format

```markdown
---
# .repository/skills/commit.md
id: commit
name: Git Commit
version: 1.0.0
description: Create well-formatted git commits with conventional commit messages
trigger: /commit
aliases:
  - /gc
  - /git-commit
providers:
  - claude
  - cursor
permissions:
  - Bash(git:*)
inputs:
  - name: message
    type: string
    description: Optional commit message override
    required: false
  - name: scope
    type: string
    description: Commit scope for conventional commits
    required: false
---

# Git Commit Skill

## Description
Create a well-formatted git commit following conventional commit standards.

## Instructions

1. **Analyze Changes**
   - Run `git status` to see all modified files
   - Run `git diff --staged` to see staged changes
   - If nothing is staged, ask user what to stage

2. **Generate Commit Message**
   - Follow conventional commit format: `<type>(<scope>): <description>`
   - Types: feat, fix, docs, style, refactor, test, chore, perf, ci
   - Keep description under 72 characters
   - Add body if changes are complex

3. **Create Commit**
   - Stage files if needed
   - Create commit with generated message
   - Include co-author attribution

## Conventional Commit Format

```text

<type>(<scope>): <short description>

[optional body]

[optional footer]

Co-Authored-By: AI Assistant <ai@example.com>

```

## Examples

```bash
# Feature commit
feat(auth): add OAuth2 login support

# Bug fix
fix(api): handle null response in user endpoint

# Documentation
docs(readme): update installation instructions
```

### 4.2 Skill Entity Data Model

```typescript
interface Skill {
  // Identity
  id: string;
  name: string;
  version: string;
  description: string;

  // Invocation
  trigger: string;                      // Primary trigger command
  aliases: string[];                    // Alternative triggers

  // Content
  content: string;                      // Markdown instructions
  checksum: string;

  // Targeting
  providers: ProviderType[];
  scope: 'global' | 'project';

  // Requirements
  permissions: string[];                // Required permissions
  inputs: SkillInput[];

  // Metadata
  enabled: boolean;
  tags: string[];
  source_path: string;
  created_at: string;
  updated_at: string;
}

interface SkillInput {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'array' | 'object';
  description: string;
  required: boolean;
  default?: unknown;
  enum?: unknown[];
}
```

---

## 5. Provider Abstraction Layer

Provider abstraction translates central config to tool-specific formats:

### 5.1 Possible Provider Interface

```typescript
interface Provider {
  // Identity
  readonly name: ProviderType;
  readonly displayName: string;
  readonly version: string;

  // Capabilities
  readonly capabilities: ProviderCapabilities;

  // Lifecycle
  detect(): Promise<ProviderDetectionResult>;
  initialize(config: ProviderConfig): Promise<void>;

  // Translation
  translateRules(rules: Rule[]): Promise<ProviderOutput>;
  translateSkills(skills: Skill[]): Promise<ProviderOutput>;
  translateSettings(settings: RepoSettings): Promise<ProviderOutput>;
  translateMcpConfig(servers: McpServer[]): Promise<ProviderOutput>;

  // Sync
  sync(output: ProviderOutput): Promise<SyncResult>;
  validateSync(): Promise<ValidationResult>;

  // Status
  getStatus(): Promise<ProviderStatus>;
  getDrift(): Promise<DriftReport>;
}

interface ProviderCapabilities {
  // Feature support
  supportsRules: boolean;
  supportsSkills: boolean;
  supportsMcp: boolean;
  supportsMemory: boolean;
  supportsPermissions: boolean;

  // Rule features
  supportsHierarchicalRules: boolean;
  supportsConditionalRules: boolean;
  supportsRulePriority: boolean;

  // Configuration features
  configFormat: 'markdown' | 'json' | 'yaml' | 'toml';
  multiFileConfig: boolean;
  watchesConfigChanges: boolean;

  // Limitations
  maxRuleSize?: number;
  maxRuleCount?: number;
  maxContextTokens?: number;
}

interface ProviderDetectionResult {
  detected: boolean;
  installed: boolean;
  version?: string;
  configPath?: string;
  existingConfig: boolean;
}

interface ProviderOutput {
  files: OutputFile[];
  warnings: string[];
  info: string[];
}

interface OutputFile {
  path: string;
  content: string;
  encoding: 'utf-8' | 'binary';
  overwrite: boolean;
}

interface SyncResult {
  success: boolean;
  filesWritten: string[];
  filesSkipped: string[];
  errors: SyncError[];
  warnings: string[];
}

interface DriftReport {
  hasDrift: boolean;
  drifts: DriftItem[];
}

interface DriftItem {
  file: string;
  type: 'modified' | 'deleted' | 'added';
  repoChecksum?: string;
  currentChecksum?: string;
}
```

### 5.2 Provider Implementation Example: Claude

```typescript
class ClaudeProvider implements Provider {
  readonly name = 'claude' as const;
  readonly displayName = 'Claude Code';
  readonly version = '1.0.0';

  readonly capabilities: ProviderCapabilities = {
    supportsRules: true,
    supportsSkills: true,
    supportsMcp: true,
    supportsMemory: true,
    supportsPermissions: true,
    supportsHierarchicalRules: true,
    supportsConditionalRules: true,
    supportsRulePriority: true,
    configFormat: 'markdown',
    multiFileConfig: true,
    watchesConfigChanges: true,
  };

  async translateRules(rules: Rule[]): Promise<ProviderOutput> {
    const files: OutputFile[] = [];

    // Generate CLAUDE.md for high-priority rules
    const mainRules = rules
      .filter(r => r.priority >= 50)
      .sort((a, b) => b.priority - a.priority);

    files.push({
      path: 'CLAUDE.md',
      content: this.generateClaudeMd(mainRules),
      encoding: 'utf-8',
      overwrite: true,
    });

    // Generate individual rule files in .claude/rules/
    for (const rule of rules) {
      files.push({
        path: `.claude/rules/${rule.id}.md`,
        content: rule.content,
        encoding: 'utf-8',
        overwrite: true,
      });
    }

    return { files, warnings: [], info: [] };
  }

  private generateClaudeMd(rules: Rule[]): string {
    let content = '# Project Instructions\n\n';
    content += '<!-- Generated by repo - do not edit directly -->\n\n';

    for (const rule of rules) {
      content += `## ${rule.name}\n\n`;
      content += rule.content + '\n\n';
    }

    return content;
  }

  // ... other methods
}
```

### 5.3 Provider Registry

```typescript
interface ProviderRegistry {
  // Registration
  register(provider: Provider): void;
  unregister(name: ProviderType): void;

  // Discovery
  get(name: ProviderType): Provider | undefined;
  getAll(): Provider[];
  getEnabled(config: RepoConfig): Provider[];

  // Detection
  detectAll(): Promise<Map<ProviderType, ProviderDetectionResult>>;
}

// Built-in providers
const builtInProviders: ProviderType[] = [
  'claude',
  'cursor',
  'copilot',
  'gemini',
  'windsurf',
];
```

---

## 6. Plugin System

### 6.1 Plugin Manifest Format

```toml
# plugin.toml - Plugin manifest file

[plugin]
name = "@repo/security-scanner"
version = "2.0.0"
description = "Security scanning integration for repo configurations"
author = "Repo Team <team@repo.dev>"
license = "MIT"
repository = "https://github.com/repo/security-scanner"

# Minimum repo version required
repo_version = ">=1.0.0"

# Plugin entry points
[plugin.entry]
main = "dist/index.js"
cli = "dist/cli.js"

# Capabilities this plugin provides
[plugin.capabilities]
provides_provider = false
provides_hooks = true
provides_commands = true
provides_rules = true

# CLI commands this plugin adds
[[plugin.commands]]
name = "scan"
description = "Run security scan on configuration"
handler = "commands/scan"

[[plugin.commands]]
name = "audit"
description = "Audit dependencies for vulnerabilities"
handler = "commands/audit"

# Hooks this plugin implements
[plugin.hooks]
pre_sync = "hooks/preSyncScan"
post_sync = "hooks/postSyncReport"

# Rules this plugin provides
[[plugin.rules]]
name = "security-defaults"
path = "rules/security-defaults.md"
description = "Default security rules"

# Configuration schema for this plugin
[plugin.config_schema]
type = "object"
properties.scan_depth = { type = "integer", default = 3 }
properties.ignore_patterns = { type = "array", items = { type = "string" } }
properties.fail_on_warning = { type = "boolean", default = false }

# Dependencies on other plugins
[plugin.dependencies]
"@repo/core" = ">=1.0.0"

# Optional peer dependencies
[plugin.peer_dependencies]
"@repo/mcp-filesystem" = ">=1.0.0"
```

### 6.2 Plugin Entity Data Model

```typescript
interface Plugin {
  // Identity
  name: string;                         // Scoped package name
  version: string;
  description: string;
  author: string;
  license: string;
  repository?: string;

  // Requirements
  repoVersion: string;                  // Semver range
  dependencies: Record<string, string>;
  peerDependencies?: Record<string, string>;

  // Entry points
  entry: {
    main: string;
    cli?: string;
  };

  // Capabilities
  capabilities: PluginCapabilities;

  // Provided features
  commands: PluginCommand[];
  hooks: PluginHooks;
  rules: PluginRule[];

  // Configuration
  configSchema?: JSONSchema;
  config?: Record<string, unknown>;

  // Runtime state
  enabled: boolean;
  loaded: boolean;
  error?: string;
}

interface PluginCapabilities {
  providesProvider: boolean;
  providesHooks: boolean;
  providesCommands: boolean;
  providesRules: boolean;
  providesSkills: boolean;
}

interface PluginCommand {
  name: string;
  description: string;
  handler: string;                      // Path to handler module
  options?: PluginCommandOption[];
}

interface PluginHooks {
  pre_sync?: string;
  post_sync?: string;
  pre_commit?: string;
  post_commit?: string;
  on_rule_change?: string;
  on_skill_change?: string;
  on_config_change?: string;
}

interface PluginRule {
  name: string;
  path: string;
  description: string;
}
```

### 6.3 Plugin Lifecycle

```typescript
interface PluginLifecycle {
  // Installation
  install(source: string, options: InstallOptions): Promise<InstallResult>;
  uninstall(name: string): Promise<void>;
  update(name: string, version?: string): Promise<UpdateResult>;

  // Loading
  load(name: string): Promise<LoadedPlugin>;
  unload(name: string): Promise<void>;
  reload(name: string): Promise<LoadedPlugin>;

  // Execution
  executeCommand(plugin: string, command: string, args: string[]): Promise<void>;
  executeHook(hookName: keyof PluginHooks, context: HookContext): Promise<HookResult>;
}

interface HookContext {
  config: RepoConfig;
  rules: Rule[];
  skills: Skill[];
  providers: Provider[];
  event: HookEvent;
}

interface HookResult {
  success: boolean;
  modified: boolean;
  data?: unknown;
  errors: Error[];
  warnings: string[];
}
```

### 6.4 Hook Points

| Hook               | Trigger                                | Context                | Can Modify    |
| :----------------- | :------------------------------------- | :--------------------- | :------------ |
| `pre_sync`         | Before syncing to providers            | Full config, providers | Config, rules |
| `post_sync`        | After syncing completes                | Sync results           | No            |
| `pre_commit`       | Before git commit (via hook)           | Staged files           | Files         |
| `post_commit`      | After git commit                       | Commit info            | No            |
| `on_rule_change`   | When rules are added/modified/removed  | Affected rules         | Rules         |
| `on_skill_change`  | When skills are added/modified/removed | Affected skills        | Skills        |
| `on_config_change` | When config.toml changes               | Old + new config       | Config        |
| `on_provider_sync` | Before each provider sync              | Provider, output       | Output        |
| `on_validation`    | During config validation               | Validation context     | Errors        |

---

## 7. Data Model Summary

### 7.1 Repository Configuration Entity

```typescript
interface RepositoryConfig {
  // Location
  rootPath: string;                     // Absolute path to repository root
  repoPath: string;                     // Path to .repository directory

  // Configuration
  config: RepoConfig;                // Parsed config.toml
  configPath: string;                   // Path to config.toml

  // Entities
  rules: Rule[];
  skills: Skill[];
  plugins: Plugin[];

  // Providers
  providers: Map<ProviderType, Provider>;
  enabledProviders: ProviderType[];

  // State
  lastSync: Date | null;
  syncStatus: SyncStatus;

  // Worktree support
  isWorktree: boolean;
  worktreeContainer?: string;
  sharedConfigPath?: string;
}

interface RepoConfig {
  repo: {
    version: string;
    schema_version: string;
  };
  project: ProjectConfig;
  providers: ProvidersConfig;
  rules: RulesConfig;
  skills: SkillsConfig;
  plugins: PluginsConfig;
  permissions: PermissionsConfig;
  context: ContextConfig;
  memory: MemoryConfig;
  mcp: McpConfig;
  hooks: HooksConfig;
  worktrees: WorktreesConfig;
}

type SyncStatus =
  | { status: 'synced'; timestamp: Date }
  | { status: 'pending'; changes: string[] }
  | { status: 'error'; errors: string[] }
  | { status: 'drift'; drifts: DriftItem[] };
```

### 7.2 Entity Relationships

```text
RepositoryConfig
├── config.toml (RepoConfig)
│   ├── Providers Configuration
│   ├── Rules Configuration
│   ├── Skills Configuration
│   ├── Plugins Configuration
│   └── MCP Configuration
│
├── Rules[]
│   ├── rule-1.md
│   ├── rule-2.md
│   └── ...
│
├── Skills[]
│   ├── skill-1.md
│   ├── skill-2.md
│   └── ...
│
├── Plugins[]
│   ├── @repo/plugin-a
│   └── @repo/plugin-b
│
└── Providers[]
    ├── ClaudeProvider
    │   └── Output: CLAUDE.md, .claude/*
    ├── CursorProvider
    │   └── Output: .cursorrules, .cursor/*
    ├── CopilotProvider
    │   └── Output: .github/copilot-instructions.md
    ├── GeminiProvider
    │   └── Output: .gemini/*
    └── WindsurfProvider
        └── Output: .windsurf/*
```

---

## 8. Sync Operation Flow

### 8.1 Sync Algorithm

```text
repo sync
│
├── 1. Load Configuration
│   ├── Parse config.toml
│   ├── Validate against JSON Schema
│   └── Resolve all paths
│
├── 2. Load Entities
│   ├── Load rules from rules/
│   ├── Load skills from skills/
│   ├── Merge global rules (if inherit_global)
│   └── Apply priority ordering
│
├── 3. Execute Pre-Sync Hooks
│   ├── Built-in validation
│   └── Plugin pre_sync hooks
│
├── 4. For Each Enabled Provider:
│   │
│   ├── 4a. Filter entities for provider
│   │   ├── Apply scope filters
│   │   ├── Apply provider filters
│   │   └── Apply condition filters
│   │
│   ├── 4b. Translate to provider format
│   │   ├── translateRules()
│   │   ├── translateSkills()
│   │   ├── translateSettings()
│   │   └── translateMcpConfig()
│   │
│   ├── 4c. Execute on_provider_sync hooks
│   │
│   ├── 4d. Check for drift
│   │   ├── Compare checksums
│   │   └── Report modifications
│   │
│   └── 4e. Write output files
│       ├── Create directories
│       ├── Write files (if changed)
│       └── Record sync metadata
│
├── 5. Execute Post-Sync Hooks
│   └── Plugin post_sync hooks
│
└── 6. Report Results
    ├── Files written
    ├── Files skipped
    ├── Warnings
    └── Errors
```

### 8.2 Drift Detection

```typescript
interface DriftDetection {
  // Check if provider configs have been modified externally
  detectDrift(provider: Provider): Promise<DriftReport>;

  // Store checksums after successful sync
  recordSync(provider: Provider, files: OutputFile[]): Promise<void>;

  // Get last sync state
  getLastSync(provider: Provider): Promise<SyncMetadata | null>;
}

interface SyncMetadata {
  timestamp: Date;
  files: Record<string, string>;        // path -> checksum
  repoVersion: string;
  configChecksum: string;
}
```

---

## 9. Directory Structure

### 9.1 Standard Project Layout

```text
project/
├── .repository/                        # Repository Manager configuration root
│   ├── config.toml                     # Central configuration
│   ├── config.local.toml               # Local overrides (gitignored)
│   ├── rules/                          # Rule definitions
│   │   ├── coding-standards.md
│   │   ├── security.md
│   │   └── testing.md
│   ├── skills/                         # Skill definitions
│   │   ├── commit.md
│   │   ├── review.md
│   │   └── deploy.md
│   ├── plugins/                        # Installed plugins
│   │   └── @repo/
│   │       └── security-scanner/
│   ├── providers/                      # Custom provider configs
│   │   └── custom-templates/
│   ├── cache/                          # Sync metadata cache
│   │   └── sync-state.json
│   └── schemas/                        # JSON schemas for validation
│       └── config.schema.json
│
├── .claude/                            # Generated: Claude Code config
│   ├── settings.json
│   ├── settings.local.json
│   └── rules/
│       └── *.md
│
├── .cursor/                            # Generated: Cursor config
│   └── rules
│
├── .cursorrules                        # Generated: Cursor rules
│
├── .github/                            # Generated: GitHub/Copilot
│   └── copilot-instructions.md
│
├── .gemini/                            # Generated: Gemini config
│   └── config.yaml
│
├── .windsurf/                          # Generated: Windsurf config
│   └── rules/
│
├── CLAUDE.md                           # Generated: Claude instructions
│
└── (project source code)
```

### 9.2 Global Configuration

```text
~/.repository/                          # User-level configuration
├── config.toml                         # Global defaults
├── rules/                              # Global rules
│   └── personal-preferences.md
├── skills/                             # Global skills
├── plugins/                            # Globally installed plugins
├── providers/                          # Custom provider implementations
├── templates/                          # Project templates
│   ├── minimal/
│   ├── standard/
│   └── enterprise/
└── cache/                              # Global cache
    └── registry-cache.json
```

### 9.3 Worktree Container Layout

```text
container/                              # Worktree container root
├── .git/                               # Centralized git database
├── .repository/                        # Shared repository configuration
│   ├── config.toml
│   ├── rules/
│   └── skills/
├── main/                               # Main branch worktree
│   ├── .git                            # File pointing to container
│   ├── .claude -> ../.repository/.claude  # Symlink to shared config
│   └── (source)
├── feature-a/                          # Feature branch worktree
│   ├── .git
│   ├── .claude -> ../.repository/.claude
│   └── (source)
└── feature-b/
    └── ...
```

---

## 10. Key Design Decisions

### 10.1 Why TOML for Configuration

| Consideration         | TOML      | YAML      | JSON      |
| :-------------------- | :-------- | :-------- | :-------- |
| Human readability     | Excellent | Good      | Poor      |
| Comments              | Native    | Native    | None      |
| Type safety           | Strong    | Weak      | Strong    |
| Nested structures     | Good      | Excellent | Excellent |
| Parsing ambiguity     | None      | High      | None      |
| Developer familiarity | Growing   | High      | Very High |
| Tooling support       | Good      | Excellent | Excellent |

**Decision**: TOML provides the best balance of readability, type safety, and unambiguous parsing. YAML's implicit typing issues (the "Norway problem" where `NO` becomes boolean `false`) make it unsuitable for configuration where strings may include reserved words.

### 10.2 Why Markdown for Rules/Skills

- **Industry alignment**: All major tools (Claude, Cursor, Copilot, Windsurf) use Markdown
- **Human readable**: Non-technical stakeholders can review/edit
- **AI friendly**: LLMs parse Markdown naturally
- **Version control**: Clean diffs for code review
- **Extensible**: YAML frontmatter for metadata without breaking readability

### 10.3 Provider Translation vs. Abstraction

**Approach**: Translation layer that generates tool-native formats

**Rejected alternative**: Force all tools to read a universal format

**Rationale**:

- Tools don't support reading arbitrary formats
- Native formats allow tool-specific optimizations
- Users can still manually edit generated files (with drift detection)
- Easier adoption - existing projects can add repo alongside current configs

### 10.4 Plugin Architecture

**Approach**: npm-style registry with manifest-based discovery

**Rationale**:

- Familiar model for JavaScript/TypeScript developers
- Supports both registry and local/git installations
- Manifest allows capability detection without loading
- Hook-based extension points enable non-invasive customization

### 10.5 Worktree Support Strategy

**Approach**: Symlink-based with optional copy fallback

**Rationale**:

- Symlinks work on all Unix systems and Windows (with dev mode)
- Copy fallback for environments without symlink support
- Aligns with research findings from `pattern-git-worktrees.md`
- Container-level shared config reduces duplication

---

## 11. Future Considerations

### 11.1 Potential Extensions

1. **Watch Mode**: Real-time sync when rules/config change
2. **Remote Config**: Pull shared config from git/HTTP
3. **Team Sync**: Share configurations via team registry
4. **AI-Assisted Config**: Generate rules from codebase analysis
5. **Validation Testing**: Test rule effectiveness across providers
6. **Migration Tools**: Convert existing tool configs to repo format

### 11.2 MCP Integration

The orchestrator should eventually become an MCP server itself:

```json
{
  "mcpServers": {
    "repo": {
      "command": "repo",
      "args": ["mcp-server"],
      "capabilities": {
        "tools": ["sync", "status", "add-rule"],
        "resources": ["config", "rules", "skills"]
      }
    }
  }
}
```

### 11.3 Standards Participation

As noted in the research, no formal standards exist for agentic tool configuration. This orchestrator could serve as a reference implementation for a future standard, potentially proposed to:

- W3C Community Group
- OpenJS Foundation
- Independent specification body

---

## Appendix A: Command Exit Codes

| Code | Meaning                                     |
| :--- | :------------------------------------------ |
| 0    | Success                                     |
| 1    | General error                               |
| 2    | Configuration error                         |
| 3    | Validation error                            |
| 4    | Provider error                              |
| 5    | Plugin error                                |
| 6    | Sync conflict (drift detected with --check) |
| 7    | Permission denied                           |
| 10   | Network error (plugin registry)             |

## Appendix B: Environment Variables

| Variable              | Description                     | Default                        |
| :-------------------- | :------------------------------ | :----------------------------- |
| `REPO_CONFIG_PATH`    | Override config file location   | `.repository/config.toml`      |
| `REPO_GLOBAL_PATH`    | Override global config location | `~/.repository`                |
| `REPO_REGISTRY`       | Override plugin registry URL    | `https://registry.repo.dev`    |
| `REPO_NO_COLOR`       | Disable colored output          | `false`                        |
| `REPO_VERBOSE`        | Enable verbose logging          | `false`                        |
| `REPO_DRY_RUN`        | Global dry-run mode             | `false`                        |

## Appendix C: Related Documents

- `../research/_index.md` - Research index and overview
- `../research/tool-*.md` - Individual tool deep-dives
- `../research/pattern-interoperability.md` - Interoperability analysis
- `../research/pattern-git-worktrees.md` - Git worktree integration patterns
- `../research/standard-*.md` - Standards landscape analysis (AGENTS.md, MCP)
- `architecture-presets.md` - Core architecture specification

---

*Document created: 2026-01-23*
*Last updated: 2026-01-23*
*Location: docs/design/spec-api-schema.md*
*Status: Design specification - ready for implementation*

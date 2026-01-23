# Emerging Standards and Schemas for Agentic Coding Tool Configuration (2026)

## Executive Summary

The agentic coding tool ecosystem in 2026 is characterized by **rapid innovation but limited formal standardization**. While several promising initiatives exist - most notably the Model Context Protocol (MCP) - the industry lacks comprehensive standards for configuration portability, behavior specification, and cross-platform interoperability.

This document analyzes the current state of standardization efforts, schema patterns, plugin systems, and identifies critical gaps that need addressing.

---

## 1. Standardization Efforts

### 1.1 Formal Standards Bodies

#### W3C / IETF Status
As of January 2026, there are **no formal W3C or IETF working groups** specifically focused on agentic coding tool configuration. However, relevant adjacent standards exist:

| Standard | Body | Relevance to Agentic Tools |
|----------|------|---------------------------|
| JSON Schema (Draft 2020-12) | JSON Schema Org | Foundation for config validation |
| OAuth 2.0 / 2.1 | IETF RFC 6749/9728 | Authentication for tool integrations |
| OpenAPI 3.1 | OpenAPI Initiative | API specification format |
| YAML 1.2 | yaml.org | Configuration file format |

**Gap:** No dedicated working group for AI agent configuration standards.

### 1.2 AGENTS.md - Universal Rules Standard

**Major Development (Mid-2025)**: Google, OpenAI, Factory, Sourcegraph, and Cursor jointly launched **AGENTS.md** - a universal standard for providing agent-specific instructions.

#### Key Facts
| Attribute | Value |
|-----------|-------|
| Launch | July 2025 |
| Adoption | 20,000+ repositories on GitHub |
| Governance | Agentic AI Foundation (Linux Foundation) |
| Website | https://agents.md/ |
| Spec Repo | https://github.com/agentsmd/agents.md |

#### Tool Support
| Tool | Support Level |
|------|---------------|
| OpenAI Codex | Native |
| Google Jules | Native |
| Cursor | Native |
| GitHub Copilot | Native |
| Aider | Native |
| RooCode | Native |
| Zed | Native |
| Factory AI | Native |
| Claude Code | Compatible |
| Gemini CLI | Native |

#### Structure
AGENTS.md is standard Markdown. Best practices cover six areas:
1. **Commands** - Build, test, lint commands
2. **Testing** - How to run and write tests
3. **Project Structure** - Directory layout and conventions
4. **Code Style** - Formatting, naming, patterns
5. **Git Workflow** - Branching, commits, PRs
6. **Boundaries** - What the agent should NOT do

#### Significance
AGENTS.md represents the first successful cross-vendor standardization of AI coding agent configuration. Its backing by major vendors (Google, OpenAI, Microsoft/GitHub, Cursor) and governance under the Linux Foundation gives it strong legitimacy.

### 1.3 Model Context Protocol (MCP)

MCP represents the **most significant tool integration standard** in the agentic tool space. Developed by Anthropic and released as open source, it provides a standardized protocol for connecting AI applications to external systems.

#### Specification Versions
| Version | Release Date | Key Changes |
|---------|--------------|-------------|
| 2024-11-05 | November 2024 | Initial release |
| 2025-03-26 | March 2025 | Transport layer improvements |
| 2025-06-18 | June 2025 | Authorization enhancements (OAuth 2.0) |
| 2025-11-25 | November 2025 | **Current stable** - Tasks, improved OAuth, extensions |

#### MCP 2025-11-25 Key Features
The November 2025 release added major capabilities:

1. **Tasks Primitive (Async Execution)**
   - Any request can return a task handle for "call-now, fetch-later"
   - Task states: `working`, `input_required`, `completed`, `failed`, `cancelled`
   - Enables multi-step operations and long-running processes

2. **Improved OAuth & Authorization**
   - Client ID Metadata Documents (CIMD) as default registration method
   - PKCE now mandatory (must use S256 code challenge)
   - Removes need for Dynamic Client Registration complexity

3. **Server Discovery**
   - Servers can publish identity documents at `.well-known` URLs
   - Enables discovery without connecting first

4. **Standardized Tool Names (SEP-986)**
   - Single canonical format for tool naming
   - Enables consistent display, sorting, referencing across SDKs

5. **Protocol Extensions**
   - Official support for industry-specific extensions
   - Curated patterns for healthcare, finance, education domains

#### MCP Architecture
```
AI Application (Client)
         |
    MCP Protocol Layer
    - Base Protocol (lifecycle, transports, authorization)
    - Client Features (roots, sampling, elicitation)
    - Server Features (prompts, resources, tools, utilities)
    - Tasks (async execution) [NEW 2025-11-25]
         |
External Systems (MCP Servers)
```

#### Adoption Momentum
- **OpenAI**: Adopted MCP in March 2025
- **Google DeepMind**: Confirmed adoption
- **Ecosystem**: 100+ official/community MCP servers
- **SDKs**: Python, TypeScript, Rust, Go

#### Governance Structure
MCP follows a formal enhancement proposal process:

| SEP Number | Focus Area |
|------------|------------|
| SEP-932 | MCP Governance framework |
| SEP-973 | Metadata standards for Resources, Tools, Prompts |
| SEP-986 | Tool naming format specifications |
| SEP-985 | OAuth 2.0 Protected Resource Metadata (RFC 9728 alignment) |
| SEP-990 | Enterprise IdP policy controls |
| SEP-1046 | OAuth client credentials flow |
| SEP-1319 | Decouple request payloads from RPC methods |
| SEP-1330 | Elicitation enum schema standards |

### 1.3 Industry-Led Initiatives

#### OpenAI Function Calling / Tools API
OpenAI has established de facto standards through their function calling API, now widely adopted:
- JSON Schema for function definitions
- Structured output specifications
- Tool use patterns

#### Google Gemini Tool Use
Google's approach closely mirrors OpenAI's function calling but with extensions for:
- Multi-modal tool inputs
- Grounding with Google Search
- Code execution environments

#### Microsoft Semantic Kernel
Microsoft's open-source SDK provides:
- Plugin abstraction layer
- Cross-model compatibility
- Configuration patterns for agent orchestration

### 1.4 Open Source Unification Attempts

| Project | Focus | Status (Jan 2026) |
|---------|-------|-------------------|
| LangChain | Agent framework | Dominant but fragmented |
| LlamaIndex | Data framework | Growing standardization |
| AutoGen | Multi-agent | Microsoft-backed |
| CrewAI | Agent orchestration | Community-driven |
| Haystack | AI pipelines | Enterprise focus |

**Challenge:** Each framework has its own configuration format, creating fragmentation rather than unification.

---

## 2. Schema Analysis

### 2.1 Configuration Schema Comparison

#### Claude Code (CLAUDE.md / .claude/)
```yaml
# Configuration approach
format: Markdown (CLAUDE.md) + JSON/YAML (.claude/)
location: Repository root, user home, or .claude directory
inheritance: Hierarchical (project > user > system)
schema_validation: Informal

# Key components
- context_rules: Natural language instructions
- skills: Modular capability definitions
- memory: Persistent context across sessions
- settings: Tool behavior configuration
```

#### Cursor (.cursorrules / .cursor/)
```yaml
# Configuration approach
format: Markdown (.cursorrules) + JSON (.cursor/settings.json)
location: Repository root or .cursor directory
inheritance: Project-level only
schema_validation: Partial

# Key components
- rules: Natural language coding guidelines
- context: File inclusion/exclusion patterns
- composer: AI composer settings
- mcp_servers: MCP server configuration
```

#### GitHub Copilot
```yaml
# Configuration approach
format: YAML (.github/copilot-instructions.md) + Settings
location: Repository .github/ or user settings
inheritance: Organization > Repository > User
schema_validation: Limited

# Key components
- instructions: Natural language guidance
- content_exclusions: Privacy patterns
- policies: Enterprise controls
```

#### Windsurf (Codeium)
```yaml
# Configuration approach
format: Markdown (rules files) + JSON (workspace settings)
location: Workspace root or .windsurf/
inheritance: Workspace > User
schema_validation: Minimal

# Key components
- cascade_rules: AI behavior guidelines
- memory: Persistent context
- context_providers: Data source configuration
```

### 2.2 Common Schema Patterns

Despite format differences, several patterns emerge across all major tools:

#### Universal Elements
1. **Natural Language Rules** - Human-readable behavioral guidelines
2. **File/Directory Patterns** - Glob patterns for context inclusion/exclusion
3. **Hierarchical Inheritance** - Project < User < System precedence
4. **Metadata Headers** - Name, version, description
5. **Context Windows** - Token/character limits

#### Shared Configuration Categories
```
root_config/
├── rules/              # Behavioral guidelines
│   ├── coding_style/
│   ├── naming/
│   └── patterns/
├── context/            # What the AI can see
│   ├── include/
│   └── exclude/
├── memory/             # Persistent state
│   ├── project_facts/
│   └── learned_preferences/
├── tools/              # MCP or native tools
│   ├── enabled/
│   └── configuration/
└── settings/           # Tool-specific settings
    ├── model/
    ├── tokens/
    └── behavior/
```

### 2.3 Schema Divergences

| Aspect | Claude Code | Cursor | Copilot | Windsurf |
|--------|-------------|--------|---------|----------|
| Primary Format | Markdown | Markdown | Markdown | Markdown |
| Config Location | .claude/, root | .cursor/, root | .github/ | .windsurf/ |
| JSON Schema | Partial | Partial | No | No |
| MCP Support | Native | Native | Limited | Native |
| Skill System | Yes | No | Extensions | No |
| Memory Persistence | Yes | Limited | No | Yes |
| Multi-file Configs | Yes | Limited | Limited | Yes |

### 2.4 Schema Evolution Timeline

```
2023 Q4: Copilot introduces .github/copilot-instructions.md
2024 Q1: Cursor adopts .cursorrules markdown format
2024 Q3: Claude introduces CLAUDE.md convention
2024 Q4: MCP v1.0 released (2024-11-05)
2025 Q1: Windsurf adds cascade rules system
2025 Q2: MCP gains OAuth 2.0 support
2025 Q3: Cross-tool rule experiments begin
2025 Q4: MCP 2025-11-25 with enterprise features
2026 Q1: Fragmentation acknowledged as industry problem
```

---

## 3. Plugin/Extension Systems

### 3.1 Extension Architectures Comparison

#### MCP (Model Context Protocol)
**Status: Leading standard for tool/resource integration**

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path"],
      "env": {}
    }
  }
}
```

**Capabilities:**
- Resources: Read-only data exposure
- Tools: Function-like capabilities
- Prompts: Reusable prompt templates
- Sampling: Model interaction delegation

**Transport Options:**
- stdio (local processes)
- HTTP with SSE
- WebSocket (emerging)

#### Language Server Protocol (LSP) Influence
While LSP doesn't directly apply to AI tools, its architecture influences emerging patterns:

```
LSP Pattern                    AI Tool Equivalent
──────────────────────────────────────────────────
TextDocumentService           → Context Provider
CodeActionProvider            → AI Suggestion System
CompletionProvider            → AI Completion
DiagnosticsProvider           → Code Review Features
```

**Observation:** No formal "AI Agent Protocol" (AAP) exists that mirrors LSP's success.

### 3.2 MCP Adoption Status (January 2026)

| Tool | MCP Support | Implementation Status |
|------|-------------|----------------------|
| Claude Code | Native | Full client support |
| Claude Desktop | Native | Full client support |
| Cursor | Native | Full client support (40 tool limit, one-click install) |
| Windsurf | Native | Full client support |
| Zed | Native | Full client support |
| Amazon Q | Native | Full client support (IDE integration) |
| VS Code (Copilot) | Limited | Experimental |
| JetBrains AI | Partial | Server support |
| OpenAI (Codex) | Native | Adopted March 2025 |
| Google (Gemini) | Native | DeepMind adoption confirmed |

**Ecosystem Growth:**
- 100+ official/community MCP servers available
- SDKs for Python, TypeScript, Rust, Go
- MCP Inspector for debugging
- One-click server installation in Cursor
- OAuth integration for enterprise servers

### 3.3 Other Interop Protocols

#### OpenAI Plugins (Deprecated Path)
OpenAI's plugin system (2023) is being superseded by:
- GPTs with Actions (JSON Schema-based)
- Assistants API with tools
- Custom tool definitions

#### Semantic Kernel Plugins
Microsoft's approach:
```csharp
[KernelFunction, Description("Get weather for location")]
public async Task<string> GetWeather(string location) { }
```
- Native function annotations
- Cross-language support
- Configuration via dependency injection

#### LangChain Tools
```python
@tool
def search_database(query: str) -> str:
    """Search the database for relevant information."""
    pass
```
- Decorator-based definition
- LCEL (LangChain Expression Language) integration
- Provider-agnostic design

### 3.4 Universal Plugin Format Assessment

**Current State: No universal format exists**

Barriers to universality:
1. Vendor differentiation incentives
2. Different capability requirements
3. Security model variations
4. Performance optimization needs
5. Licensing/commercial concerns

**Most Promising Path:** MCP adoption as de facto standard, potentially with formal standardization in 2026-2027.

---

## 4. Configuration as Code

### 4.1 Version Control Practices

#### Current Best Practices
```
repository/
├── .github/
│   └── copilot-instructions.md    # Copilot rules
├── .cursor/
│   └── rules/                      # Cursor rules
├── .claude/
│   ├── CLAUDE.md                   # Claude rules
│   ├── skills/                     # Claude skills
│   └── settings.json               # Tool settings
├── .windsurf/
│   └── rules/                      # Windsurf rules
└── .agentic/                       # Universal (proposed)
    ├── rules.md
    ├── context.yaml
    └── tools.json
```

#### Versioning Patterns
1. **Semantic versioning** in config headers (rare in practice)
2. **Git-based versioning** (most common)
3. **Date-based versioning** (MCP specification approach)
4. **No versioning** (unfortunately common)

### 4.2 CI/CD Integration

#### Rule Validation Pipelines
```yaml
# Example: GitHub Actions for AI config validation
name: Validate AI Configurations
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Validate JSON/YAML syntax
      - name: Lint configurations
        run: |
          yamllint .claude/settings.yaml
          jsonlint .cursor/settings.json

      # Schema validation (if schemas exist)
      - name: Validate against schema
        run: |
          ajv validate -s schemas/claude-config.json -d .claude/

      # Custom rule validation
      - name: Check rule conflicts
        run: python scripts/validate_rules.py
```

#### Emerging Tools
- **Claude Code Rules Linter** (community project)
- **CursorLint** (third-party validation)
- **AI Config Validator** (multi-tool support, beta)

### 4.3 Testing Frameworks for AI Behavior

#### Current State: Immature

| Framework | Purpose | Maturity |
|-----------|---------|----------|
| Promptfoo | Prompt testing | Production-ready |
| DeepEval | LLM evaluation | Growing |
| Ragas | RAG evaluation | Specialized |
| Langsmith | Tracing/debugging | Comprehensive |
| Arize Phoenix | Observability | Enterprise |

#### Testing Patterns
```python
# Example: Testing rule effectiveness
def test_coding_style_rule():
    """Test that coding style rules are followed."""
    prompt = "Write a function to calculate factorial"

    # With rules applied
    response_with_rules = agent.complete(prompt, rules=coding_rules)

    # Assertions
    assert uses_type_hints(response_with_rules)
    assert follows_naming_convention(response_with_rules)
    assert has_docstring(response_with_rules)
```

#### Gap: No Standard Testing Framework
There is no widely-adopted framework specifically for testing:
- Rule effectiveness across different prompts
- Behavioral consistency across model versions
- Cross-tool behavior equivalence

---

## 5. Gap Analysis

### 5.1 Missing Standards

| Gap | Impact | Urgency |
|-----|--------|---------|
| Universal rule format | Config duplication across tools | High |
| Behavior specification language | Inconsistent AI behavior | High |
| Memory/context portability | Vendor lock-in | Medium |
| Skill/plugin interoperability | Duplicate development effort | High |
| Testing/validation schemas | Quality assurance challenges | Medium |
| Security/permissions model | Enterprise adoption barriers | High |
| Observability standards | Debugging difficulty | Medium |

### 5.2 Universal Schema Requirements

A comprehensive universal schema would need to address:

#### Core Configuration
```yaml
# Proposed universal schema structure
agentic_config:
  version: "1.0.0"
  schema: "https://agentic-standards.org/schema/v1"

  metadata:
    name: "Project Configuration"
    description: "AI coding assistant configuration"
    author: "team@example.com"

  rules:
    format: "markdown"  # or "structured"
    inheritance: "merge"  # or "override"
    files:
      - path: "rules/*.md"
        priority: 100

  context:
    include:
      - "src/**/*.ts"
      - "docs/**/*.md"
    exclude:
      - "node_modules/**"
      - "**/*.min.js"
    max_tokens: 100000

  memory:
    enabled: true
    scope: "project"  # or "user", "session"
    persistence: "file"  # or "service"

  tools:
    mcp_servers:
      - name: "filesystem"
        uri: "npx://@mcp/server-filesystem"
        config:
          allowed_paths: ["./src"]
    native_tools:
      - shell: true
      - file_edit: true

  security:
    allowed_operations:
      - read_files
      - write_files
      - execute_commands
    blocked_patterns:
      - "**/secrets/**"
      - "**/.env*"
    require_confirmation:
      - delete_files
      - git_push
```

#### Behavioral Specification
```yaml
  behavior:
    coding_style:
      language_defaults:
        typescript:
          use_strict: true
          prefer_const: true
          naming_convention: "camelCase"

    response_format:
      verbosity: "concise"  # or "detailed", "minimal"
      include_explanations: true
      code_comments: "minimal"

    safety:
      confirm_destructive: true
      sandbox_execution: true
      audit_logging: true
```

### 5.3 Barriers to Standardization

#### Technical Barriers
1. **Rapid Innovation** - Tools evolving faster than standards can form
2. **Capability Differences** - Different models have different strengths
3. **Context Handling Variations** - Each tool optimizes differently
4. **Performance Trade-offs** - Universal format may sacrifice speed

#### Commercial Barriers
1. **Competitive Differentiation** - Vendors want unique features
2. **Lock-in Incentives** - Switching costs benefit incumbents
3. **IP Concerns** - Proprietary optimizations
4. **Support Burden** - Cross-tool compatibility is expensive

#### Organizational Barriers
1. **No Central Authority** - Who leads standardization?
2. **Resource Allocation** - Standards work is unfunded
3. **Governance Complexity** - Multiple stakeholders
4. **Adoption Inertia** - Existing tools won't easily change

### 5.4 Path Forward Recommendations

#### Short-term (2026)
1. **Adopt MCP** as the de facto tool integration standard
2. **Document tool-specific schemas** in JSON Schema format
3. **Create conversion utilities** between major config formats
4. **Establish community working group** for rule portability

#### Medium-term (2026-2027)
1. **Propose formal standardization** to appropriate body (possibly W3C Community Group)
2. **Develop universal rule format** specification
3. **Create compliance test suites** for implementations
4. **Build reference implementations** in major languages

#### Long-term (2027+)
1. **Formal IETF/W3C standardization** of core protocols
2. **Certification programs** for compliant tools
3. **Enterprise governance frameworks** adoption
4. **Cross-vendor testing and validation** infrastructure

---

## 6. Conclusions

### Current State Assessment

| Area | Maturity | Trend |
|------|----------|-------|
| **Rules Format (AGENTS.md)** | Maturing | **Rapid convergence** |
| **Tool Integration (MCP)** | Mature | **Strong adoption** |
| Memory/Context | Proprietary | No convergence |
| Testing/Validation | Immature | Growing interest |
| Formal Standards | Emerging | Linux Foundation involvement |

### Key Findings

1. **Two complementary standards have emerged** with real traction:
   - **AGENTS.md**: Universal rules format with 20,000+ repo adoption and Linux Foundation governance
   - **MCP**: Tool integration protocol with OpenAI, Google, and major IDE adoption

2. **Configuration format convergence is happening** - AGENTS.md provides the universal standard that was missing. Major vendors (Google, OpenAI, Microsoft/GitHub, Cursor) are aligned.

3. **Formal governance exists** - Agentic AI Foundation (Linux Foundation) now stewards AGENTS.md; MCP has its own SEP process.

4. **Memory portability remains the biggest gap** - No standard for shared context/memory across tools.

5. **Testing and validation** frameworks for AI behavior remain immature and non-standardized.

### Strategic Recommendations

1. **Adopt AGENTS.md** as the primary rules format - it has vendor backing and formal governance.

2. **Bet on MCP** for tool integration - it has the best momentum with OpenAI and Google adoption.

3. **Design for dual-standard architecture** - AGENTS.md for rules, MCP for tools/resources.

4. **Invest in validation tooling** - this is an underserved area with high need.

5. **Participate in Linux Foundation efforts** to shape AGENTS.md evolution.

6. **Design for portability** even when using tool-specific features.

---

## Appendix A: MCP Schema Reference

### Tool Definition Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "name": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9_-]*$"
    },
    "description": {
      "type": "string"
    },
    "inputSchema": {
      "$ref": "http://json-schema.org/draft-07/schema#"
    }
  },
  "required": ["name", "description", "inputSchema"]
}
```

### Resource Definition Schema
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "uri": {
      "type": "string",
      "format": "uri"
    },
    "name": {
      "type": "string"
    },
    "mimeType": {
      "type": "string"
    },
    "description": {
      "type": "string"
    }
  },
  "required": ["uri", "name"]
}
```

## Appendix B: Tool Configuration Locations

| Tool | Primary Config | Universal (AGENTS.md) | MCP Config |
|------|---------------|----------------------|------------|
| Claude Code | CLAUDE.md, .claude/rules/ | Supported | .claude/settings.json |
| Cursor | .cursorrules, .cursor/rules/ | Native | .cursor/mcp.json |
| Copilot | .github/copilot-instructions.md | Native | N/A |
| Windsurf | .windsurf/rules/, Rulebooks | Supported | Native support |
| Zed | .zed/settings.json | Native | Native support |
| Gemini | .gemini/, GEMINI.md | Native | N/A |
| Amazon Q | .amazonq/default.json | N/A | Native support |
| OpenAI Codex | AGENTS.md | Native | Native support |
| Google Jules | AGENTS.md | Native | N/A |

## Appendix C: Related Specifications

- [AGENTS.md Specification](https://agents.md/) - Universal AI coding agent configuration
- [AGENTS.md GitHub](https://github.com/agentsmd/agents.md) - Official repository and tooling
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-11-25) - Model Context Protocol (current)
- [MCP Blog - 2025-11-25 Release](https://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/) - Anniversary release notes
- [JSON Schema](https://json-schema.org/specification) - Configuration validation
- [OpenAPI 3.1](https://spec.openapis.org/oas/v3.1.0) - API specifications
- [OAuth 2.1](https://oauth.net/2.1/) - Authentication standard
- [LSP Specification](https://microsoft.github.io/language-server-protocol/) - Language Server Protocol

## Appendix D: Sources Consulted

- [GitHub Blog - AGENTS.md Lessons](https://github.blog/ai-and-ml/github-copilot/how-to-write-a-great-agents-md-lessons-from-over-2500-repositories/)
- [OpenAI Codex - AGENTS.md Guide](https://developers.openai.com/codex/guides/agents-md)
- [WorkOS - MCP 2025-11-25 Analysis](https://workos.com/blog/mcp-2025-11-25-spec-update)
- [Cursor MCP Documentation](https://cursor.com/docs/context/mcp)
- [Windsurf Cascade Documentation](https://docs.windsurf.com/windsurf/cascade/cascade)
- [Amazon Q MCP Documentation](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/mcp-ide.html)
- [GitHub Copilot Extensions](https://github.com/features/copilot/extensions)

---

*Research conducted: January 2026*
*Last updated: 2026-01-23*
*Status: Complete*
*Branch: research-docs*

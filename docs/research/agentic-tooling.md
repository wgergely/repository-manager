# Agentic Tooling & Ecosystem Research

Consolidated research into the state of agentic coding tools and the Git hooks ecosystem (2025/2026).

## Agentic IDE Support

### Gemini Code Assist

- **Configuration**: JSON-based settings (e.g., `.gemini/settings.json`), NOT YAML.
- **Hierarchy**: Follows a standard project -> global search hierarchy for configuration.

### GitHub Copilot Extensions

- **Ecosystem**: Moving towards an "Agentic" SDK to allow tools to perform actions on behalf of the user.
- **Integration**: Strong focus on IDE "Agent Mode" (e.g., JetBrains, VSCode) for autonomous task execution.

## Git Hooks Ecosystem

### Established Tools

- **pre-commit**: Python-based, massive hook library, industry standard.
- **lefthook**: Go-based, extremely fast, concurrent execution, simple YAML config.

### Emerging Candidates

- Research identifies a shift towards more language-agnostic, single-binary tools that focus on developer experience and CI integration stability.

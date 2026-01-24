# Other Protocols and Standards

Additional standardization efforts beyond AGENTS.md and MCP.

## OpenAI Function Calling / Tools API

De facto standard through OpenAI's API:

- JSON Schema for function definitions
- Structured output specifications
- Tool use patterns

```json
{
  "name": "get_weather",
  "description": "Get weather for a location",
  "parameters": {
    "type": "object",
    "properties": {
      "location": { "type": "string" }
    },
    "required": ["location"]
  }
}
```

**Status**: Widely adopted, but being superseded by:
- GPTs with Actions (JSON Schema-based)
- Assistants API with tools

## Google Gemini Tool Use

Mirrors OpenAI's function calling with extensions:

- Multi-modal tool inputs
- Grounding with Google Search
- Code execution environments

## Microsoft Semantic Kernel & LangChain

| Technology | Pattern | Config |
|------------|---------|--------|
| Semantic Kernel | C# attributes | DI injection |
| LangChain | Python decorators | LCEL |

## LSP (Language Server Protocol) Influence

While not directly for AI tools, LSP patterns influence design:

| LSP Pattern | AI Tool Equivalent |
|-------------|-------------------|
| TextDocumentService | Context Provider |
| CodeActionProvider | AI Suggestion System |
| CompletionProvider | AI Completion |
| DiagnosticsProvider | Code Review Features |

**Note**: No formal "AI Agent Protocol" (AAP) exists that mirrors LSP's success.

## Open Source Frameworks

| Project | Focus | Status (Jan 2026) |
|---------|-------|-------------------|
| LangChain | Agent framework | Dominant but fragmented |
| LlamaIndex | Data framework | Growing |
| AutoGen | Multi-agent | Microsoft-backed |
| CrewAI | Agent orchestration | Community-driven |
| Haystack | AI pipelines | Enterprise focus |

**Challenge**: Each framework has its own configuration format.

## Formal Standards Status

| Standard | Body | Relevance |
|----------|------|-----------|
| JSON Schema | JSON Schema Org | Config validation |
| OAuth 2.0/2.1 | IETF | Tool authentication |
| OpenAPI 3.1 | OpenAPI Initiative | API specification |
| YAML 1.2 | yaml.org | Config format |

**Gap**: No W3C/IETF working group for AI agent configuration.

## Standardization Path

### Current Reality
- AGENTS.md emerging as rules standard
- MCP emerging as tool integration standard
- Memory/context: no standards
- Skills: fragmented

### Future Outlook
- AGENTS.md has Linux Foundation governance
- MCP has formal SEP process
- Other areas may follow similar paths

---

*Last updated: 2026-01-23*
*Status: Complete*

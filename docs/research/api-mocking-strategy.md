# API Mocking Strategy Research

> **Purpose:** Define strategy for mocking LLM APIs during testing vs using real APIs.
> **Last Updated:** 2026-01-29
> **Status:** Research Draft

## Overview

The tools we support require various API credentials:

| Provider | Tools Using | API Type |
|----------|------------|----------|
| Anthropic | Claude, Cursor, Aider, Cline, Roo | Messages API |
| OpenAI | Aider, Copilot (backend), Cline, Roo | Chat Completions |
| Google | Gemini CLI | Vertex AI / Gemini API |
| GitHub | GitHub Copilot | Copilot API |
| AWS | Amazon Q | Bedrock / Q API |

This document defines when to mock vs use real APIs, and how to implement mocking.

---

## Testing Modes

### Mode 1: Mock (CI/PR Checks)

**When:** Every PR, every push, local development

**Behavior:**
- All API calls hit a local mock server
- Responses are deterministic
- Zero API cost
- Fast execution

**Use for:**
- Config file generation testing
- Tool detection testing
- Basic integration flows
- Regression testing

### Mode 2: Real (Certification)

**When:** Weekly certification runs, pre-release validation

**Behavior:**
- Real API calls to actual providers
- Uses test API keys from `.env`
- Has real cost implications
- Tests actual integration

**Use for:**
- End-to-end validation
- Compatibility verification
- Catching real-world issues

### Mode 3: Hybrid (Development)

**When:** Local development with selective real calls

**Behavior:**
- Mock by default
- Real APIs for specifically marked tests
- Developer controls via env vars

**Use for:**
- Developing new tool integrations
- Debugging specific issues

---

## Credential Management

### File Structure

```
project-root/
├── .env.example          # Committed template
├── .env                  # Gitignored - real credentials
├── .gitignore            # Contains ".env"
└── docker/
    └── secrets/          # Gitignored - for JSON/complex credentials
        └── gcloud-key.json
```

### .env.example Template

```bash
# ==============================================
# Repository Manager Test Environment
# ==============================================
# Copy this file to .env and fill in values
# NEVER commit .env to version control
# ==============================================

# Test Mode: mock | real | hybrid
TEST_MODE=mock

# Mock API Server (when TEST_MODE=mock or hybrid)
MOCK_API_URL=http://mock-api:8080

# ----------------------------------------------
# LLM Provider API Keys
# ----------------------------------------------

# Anthropic (Claude, Cursor, Aider, Cline, Roo)
ANTHROPIC_API_KEY=sk-ant-api03-xxxx

# OpenAI (Aider, some Cline/Roo configs)
OPENAI_API_KEY=sk-xxxx

# Google Cloud (Gemini CLI)
# Option 1: API Key
GOOGLE_API_KEY=xxxx
# Option 2: Service Account (place JSON at docker/secrets/gcloud-key.json)
GOOGLE_APPLICATION_CREDENTIALS=/workspace/secrets/gcloud-key.json

# ----------------------------------------------
# Platform Credentials
# ----------------------------------------------

# GitHub (Copilot)
# Personal Access Token with copilot scope
GITHUB_TOKEN=ghp_xxxx

# AWS (Amazon Q)
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=xxxx
AWS_DEFAULT_REGION=us-east-1

# ----------------------------------------------
# Optional: Provider-specific overrides
# ----------------------------------------------

# Override base URLs (for proxies, regional endpoints)
# ANTHROPIC_BASE_URL=https://api.anthropic.com
# OPENAI_API_BASE=https://api.openai.com/v1
```

### .gitignore Entries

```gitignore
# Environment files with secrets
.env
.env.local
.env.*.local

# Secret files
docker/secrets/
*.pem
*.key
*-key.json
```

### Security Considerations

1. **Never commit credentials** - `.env` must always be gitignored
2. **Use restricted API keys** - Create keys with minimal permissions
3. **Rotate regularly** - Especially if accidentally exposed
4. **CI secrets** - Use GitHub Actions secrets, not env files

---

## Mock Server Options

### Option 1: WireMock

**Pros:**
- Industry standard
- Rich matching and templating
- Stateful scenarios
- Good documentation

**Cons:**
- Java-based (requires JVM)
- Larger container footprint

```dockerfile
FROM wiremock/wiremock:3.3.1
COPY stubs/ /home/wiremock/mappings/
```

**Stub example:**
```json
{
  "request": {
    "method": "POST",
    "urlPath": "/v1/messages",
    "headers": {
      "x-api-key": { "matches": ".*" }
    }
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "id": "msg_mock_123",
      "type": "message",
      "role": "assistant",
      "content": [
        {
          "type": "text",
          "text": "Mock response from Anthropic API"
        }
      ],
      "model": "claude-3-opus-20240229",
      "stop_reason": "end_turn"
    }
  }
}
```

### Option 2: Prism (OpenAPI-based)

**Pros:**
- Uses OpenAPI specs directly
- Auto-generates realistic responses
- Validates requests against spec
- Node.js-based (lighter than Java)

**Cons:**
- Requires OpenAPI specs for each API
- Less flexible for custom scenarios

```dockerfile
FROM stoplight/prism:5
COPY openapi-specs/ /specs/
CMD ["mock", "-h", "0.0.0.0", "/specs/anthropic.yaml"]
```

### Option 3: Custom Rust Mock Server

**Pros:**
- Tailored to our exact needs
- Rust ecosystem consistency
- Minimal footprint
- Full control

**Cons:**
- Development effort
- Maintenance burden

```rust
// Conceptual structure
use axum::{Router, routing::post, Json};

async fn anthropic_messages(
    Json(request): Json<MessagesRequest>
) -> Json<MessagesResponse> {
    Json(MessagesResponse {
        id: "msg_mock".into(),
        content: vec![Content::Text {
            text: format!("Mock response for: {:?}", request.messages)
        }],
        // ...
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/v1/messages", post(anthropic_messages))
        .route("/v1/chat/completions", post(openai_completions));

    axum::serve(listener, app).await.unwrap();
}
```

### Option 4: mockserver (Node.js)

**Pros:**
- JavaScript-based
- Easy to set up
- Programmatic API

**Cons:**
- Less feature-rich than WireMock

```javascript
const mockServer = require('mockserver-node');

mockServer.start_mockserver({
    serverPort: 8080,
    verbose: true
});

// Set expectations via REST API
```

### Recommendation

**WireMock** for initial implementation because:
1. Battle-tested in enterprise environments
2. Rich feature set (proxying, recording, templating)
3. JSON-based configuration (easy to version control)
4. Can record real API responses for replay

Consider custom Rust mock server later if:
- Container size becomes an issue
- Need tighter Rust test integration
- Specific behaviors not supported by WireMock

---

## API Response Stubs

### Anthropic Messages API

**Endpoint:** `POST /v1/messages`

**Request example:**
```json
{
  "model": "claude-3-opus-20240229",
  "max_tokens": 1024,
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

**Mock response:**
```json
{
  "id": "msg_mock_12345",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Hello! This is a mock response from the test API server."
    }
  ],
  "model": "claude-3-opus-20240229",
  "stop_reason": "end_turn",
  "stop_sequence": null,
  "usage": {
    "input_tokens": 10,
    "output_tokens": 15
  }
}
```

### OpenAI Chat Completions

**Endpoint:** `POST /v1/chat/completions`

**Mock response:**
```json
{
  "id": "chatcmpl-mock123",
  "object": "chat.completion",
  "created": 1700000000,
  "model": "gpt-4",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! This is a mock response from the test API server."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 15,
    "total_tokens": 25
  }
}
```

### Streaming Responses

Both Anthropic and OpenAI support streaming. Mock implementation:

```json
{
  "request": {
    "method": "POST",
    "urlPath": "/v1/messages",
    "bodyPatterns": [
      { "contains": "\"stream\": true" }
    ]
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "text/event-stream"
    },
    "body": "event: message_start\ndata: {\"type\":\"message_start\"}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Mock \"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"streaming \"}}\n\nevent: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"response\"}}\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"
  }
}
```

---

## Response Recording

WireMock can act as a proxy and record real API responses for later replay:

### Recording Setup

```json
{
  "targetBaseUrl": "https://api.anthropic.com",
  "filters": {
    "headers": {
      "exclude": ["x-api-key", "authorization"]
    }
  },
  "captureHeaders": {
    "Content-Type": {}
  },
  "requestBodyPattern": {
    "matcher": "equalToJson",
    "ignoreArrayOrder": true
  }
}
```

### Recording Workflow

```bash
# Start WireMock in recording mode
docker run -p 8080:8080 wiremock/wiremock --record-mappings

# Point tools at WireMock proxy
ANTHROPIC_BASE_URL=http://localhost:8080 claude "test prompt"

# Recordings saved to mappings/ directory
# Sanitize recordings (remove API keys, personal data)
# Commit sanitized recordings
```

### Benefits of Recording

1. **Realistic responses** - Actual API behavior captured
2. **Regression testing** - Detect when API changes
3. **Offline development** - Work without API access
4. **Reduce API costs** - Record once, replay forever

---

## Environment Variable Routing

Tools typically allow overriding base URLs:

| Tool | Environment Variable | Default |
|------|---------------------|---------|
| Claude CLI | `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` |
| Aider | `OPENAI_API_BASE` | `https://api.openai.com/v1` |
| Cline | Extension settings | Varies |
| Roo | Extension settings | Varies |

### Mock Mode Configuration

```bash
# In .env for mock mode
ANTHROPIC_BASE_URL=http://mock-api:8080
OPENAI_API_BASE=http://mock-api:8080
```

### Docker Compose for Mock Mode

```yaml
# docker-compose.ci.yml
services:
  claude:
    environment:
      ANTHROPIC_API_KEY: "mock-key"
      ANTHROPIC_BASE_URL: "http://mock-api:8080"
    depends_on:
      - mock-api

  mock-api:
    image: wiremock/wiremock:3.3.1
    volumes:
      - ./docker/mock-api/stubs:/home/wiremock/mappings
    ports:
      - "8080:8080"
```

---

## Test Scenarios

### Scenario 1: Basic Functionality

```gherkin
Given the mock API returns a standard response
When the tool sends a message
Then it should receive the mock response
And it should parse the response correctly
```

### Scenario 2: Error Handling

```gherkin
Given the mock API returns a 429 rate limit error
When the tool sends a message
Then it should handle the error gracefully
And it should retry appropriately
```

**Mock error response:**
```json
{
  "request": {
    "method": "POST",
    "urlPath": "/v1/messages",
    "headers": {
      "x-trigger-error": { "equalTo": "rate-limit" }
    }
  },
  "response": {
    "status": 429,
    "headers": {
      "Content-Type": "application/json",
      "retry-after": "60"
    },
    "jsonBody": {
      "type": "error",
      "error": {
        "type": "rate_limit_error",
        "message": "Rate limit exceeded"
      }
    }
  }
}
```

### Scenario 3: Streaming

```gherkin
Given the mock API returns a streaming response
When the tool requests streaming
Then it should receive events incrementally
And it should assemble the complete response
```

### Scenario 4: Tool Use (Function Calling)

```gherkin
Given the mock API returns a tool use response
When the tool sends a message with tools
Then it should receive the tool call
And it should execute the tool correctly
```

---

## CI Pipeline Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  mock-tests:
    runs-on: ubuntu-latest
    services:
      mock-api:
        image: wiremock/wiremock:3.3.1
        ports:
          - 8080:8080
        volumes:
          - ./docker/mock-api/stubs:/home/wiremock/mappings

    steps:
      - uses: actions/checkout@v4

      - name: Run tests with mock API
        env:
          TEST_MODE: mock
          ANTHROPIC_BASE_URL: http://localhost:8080
        run: cargo test --features integration

  certification:
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: certification  # Requires approval

    steps:
      - uses: actions/checkout@v4

      - name: Run certification tests
        env:
          TEST_MODE: real
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: cargo test --features certification
```

---

## Cost Management

### API Cost Estimates

| Provider | Model | Input (per 1M tokens) | Output (per 1M tokens) |
|----------|-------|----------------------|------------------------|
| Anthropic | Claude 3 Opus | $15.00 | $75.00 |
| Anthropic | Claude 3 Sonnet | $3.00 | $15.00 |
| Anthropic | Claude 3 Haiku | $0.25 | $1.25 |
| OpenAI | GPT-4 | $30.00 | $60.00 |
| OpenAI | GPT-3.5 Turbo | $0.50 | $1.50 |

### Cost Control Strategies

1. **Use cheapest model for testing** - Haiku/GPT-3.5 when testing API connectivity
2. **Limit token counts** - Short prompts, low max_tokens
3. **Cache responses** - Don't repeat identical calls
4. **Budget alerts** - Set spending limits on API accounts
5. **Certification frequency** - Weekly, not daily

### Estimated Certification Cost

Assuming 100 API calls per certification run:
- ~1000 input tokens per call
- ~500 output tokens per call
- Using Claude 3 Haiku

**Per run:** ~$0.10
**Weekly:** ~$0.40/month
**Real cost driver:** Opus testing if needed

---

## Research TODOs

- [ ] Set up WireMock container with basic stubs
- [ ] Verify tool base URL override works for each tool
- [ ] Record real API responses for replay
- [ ] Test streaming mock responses
- [ ] Measure mock server latency vs real API
- [ ] Document which tools support base URL override
- [ ] Create error scenario stubs

# Amazon Q Developer (AWS)

AWS's AI-powered developer assistant integrated with AWS services.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Amazon Web Services |
| **Models** | Amazon Bedrock models |
| **Type** | IDE Extension + AWS Console |
| **MCP Support** | Native |
| **AGENTS.md** | Not confirmed |

## Configuration Files

### .amazonq/ Directory

```
.amazonq/
├── default.json               # Default behavior configuration
├── rules/                     # Project rules
│   └── *.md
└── agents/                    # Custom agent definitions
```

### default.json

```json
{
  "behavior": {
    "language": "typescript",
    "framework": "aws-cdk"
  },
  "aws": {
    "region": "us-east-1",
    "profile": "dev"
  },
  "context": {
    "include": ["lib/**/*", "bin/**/*"],
    "exclude": ["cdk.out/**"]
  }
}
```

### Additional Configuration

- AWS Toolkit settings in IDE
- AWS profile configuration
- IAM permissions for AWS service access

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Inline completions | Full | Core feature |
| Chat panel | Full | Q Developer chat |
| Multi-file editing | Partial | IDE-dependent |
| Terminal access | Partial | Via IDE terminal |
| Autonomous coding | Partial | AWS-focused automation |
| Git operations | Limited | Basic support |
| MCP | Native | Configured in IDE settings |

## AWS Service Integration

Deep integration with AWS services:

- **Lambda**: Function generation, deployment
- **CDK/CloudFormation**: Infrastructure as code
- **DynamoDB**: Query assistance
- **S3**: File operations
- **Step Functions**: Workflow generation
- **CodeCommit/CodePipeline**: CI/CD integration

## MCP Configuration

Configured through IDE settings (VS Code, JetBrains):

```json
{
  "amazonQ": {
    "mcpServers": {
      "database": {
        "command": "npx",
        "args": ["-y", "@mcp/server-dynamodb"]
      }
    }
  }
}
```

## Context Management

- AWS account context
- Project configuration files
- IDE workspace settings
- AWS service-specific context

## Memory/Persistence

| Type | Persistence | Format |
|------|-------------|--------|
| Session | Limited | In-memory |
| Project | Via .amazonq/ | JSON |
| AWS Context | Via AWS profile | AWS config |

## Pricing

| Tier | Price | Features |
|------|-------|----------|
| Free Tier | $0 | Limited queries |
| Pro | $19/user/month | Full features |
| Enterprise | Custom | SSO, compliance |

## Configuration Discovery

```
1. IDE extension loads
2. Check AWS Toolkit configuration
3. Load .amazonq/ directory if present
4. Merge with AWS profile settings
5. Initialize with AWS context
```

## Unique Differentiators

1. **AWS Native**: Deep AWS service integration
2. **Security Scanning**: Built-in code security analysis
3. **Infrastructure Focus**: Strong CDK/CloudFormation support
4. **Enterprise Features**: IAM, SSO, compliance

## Limitations

- Heavily AWS-ecosystem focused
- Limited standalone functionality
- Less flexible for non-AWS projects
- Configuration format not standardized

## Quick Reference

```
./.amazonq/
├── default.json               # Default configuration
├── rules/                     # Project rules
│   └── *.md
└── agents/                    # Custom agents
# Plus: AWS Toolkit IDE settings
# Plus: AWS profile configuration
```

---

*Last updated: 2026-01-23*
*Status: Complete*

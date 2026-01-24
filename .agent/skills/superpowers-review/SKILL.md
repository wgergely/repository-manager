---
name: superpowers-review
description: Reviews changes for correctness, edge cases, style, security, and maintainability. Dispatch subagents to perform reviews.
---

# Code Review Skill

## Overview

Reviews changes for correctness, edge cases, style, security, and maintainability.

**Core principle:** Review early, review often.

## When to Use

**Mandatory:**

- After each task in subagent-driven development (Spec compliance first, then Code quality)
- After completing major feature
- Before merge to main

**Optional but valuable:**

- When stuck (fresh perspective)
- Before refactoring (baseline check)
- After fixing complex bug

## Output Format (for Code Quality Reviewer)

When performing a code review, return:

1. **Strengths**: What is done well? (Architecture, tests, clarity)
2. **Issues**: Grouped by severity
    - **Critical**: Wrong behavior, security issue, data loss risk, broken tests/build
    - **Important**: Likely bug, missing edge cases, poor reliability, missing tests
    - **Minor**: Style, clarity, small maintainability issues, magic numbers
3. **Assessment**:
    - "Ready to proceed" (if only minor/nits)
    - "Needs changes" (if Critical/Important issues exist)

## Checklist for Reviewers

1. **Correctness vs requirements**
    - Does it actually solve the problem as described?
    - Are there extra features not requested? (YAGNI)
2. **Edge cases & error handling**
    - What if input is empty/null/huge?
    - Are network failures handled?
    - Are errors swallowed or reported?
3. **Tests**
    - Do tests actually verify behavior? (Not just mocks)
    - Are tests comprehensive?
    - do they follow TDD patterns?
4. **Security**
    - Secrets exposed?
    - Auth checks present?
    - Input validation?
5. **Performance**
    - Obvious hotspots?
    - N+1 queries?
    - Unnecessary loops/allocations?
6. **Readability & maintainability**
    - Functions short and focused?
    - Names clear and accurate?
    - Comments explain WHY not WHAT?

## Integration with Subagents

**Subagent-Driven Development:**

- Spec Compliance Review: Checks if *what* was built matches requirements.
- Code Quality Review: Checks *how* it was built (using this skill).

**Red Flags (Subagent Behavior):**

- Returning "LGTM" without analysis
- Skipping test verification
- Ignoring edge cases
- Focusing only on style (nits) while missing bugs

## How to Review as a Subagent

1. **Read the Context**: What was the plan? What are the requirements?
2. **Read the Code**: Don't trust the PR description. Read the source.
3. **Check Tests**: Run them if possible. Read them carefully.
4. **Formulate Feedback**: Be specific. Link to files/lines.
5. **Summarize**: Give a clear Go/No-Go recommendation.

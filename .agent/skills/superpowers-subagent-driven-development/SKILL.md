---
name: superpowers-subagent-driven-development
description: Use when executing implementation plans with independent tasks in the current session
---

# Subagent-Driven Development

Execute plan by dispatching fresh subagent per task, with two-stage review after each: spec compliance review first, then code quality review.

**Core principle:** Fresh subagent per task + two-stage review (spec then quality) = high quality, fast iteration

## When to Use

```mermaid
graph TD
    A[Have implementation plan?] -->|yes| B[Tasks mostly independent?]
    A -->|no| C[Manual execution or brainstorm first]
    B -->|yes| D[Stay in this session?]
    B -->|no - tightly coupled| C
    D -->|yes| E[subagent-driven-development]
    D -->|no - parallel session| F[executing-plans]
```

**vs. Executing Plans (parallel session):**

- Same session (no context switch)
- Fresh subagent per task (no context pollution)
- Two-stage review after each task: spec compliance first, then code quality
- Faster iteration (no human-in-loop between tasks)

## The Process

```mermaid
graph TB
    subgraph per_task [Per Task]
        A[Dispatch implementer subagent] --> B{Implementer subagent asks questions?}
        B -->|yes| C[Answer questions, provide context]
        C --> A
        B -->|no| D[Implementer subagent implements, tests, commits, self-reviews]
        D --> E[Dispatch spec reviewer subagent]
        E --> F{Spec reviewer confirms?}
        F -->|no| G[Implementer subagent fixes spec gaps]
        G --> E
        F -->|yes| H[Dispatch code quality reviewer subagent]
        H --> I{Code quality reviewer approves?}
        I -->|no| J[Implementer subagent fixes quality issues]
        J --> H
        I -->|yes| K[Mark task complete]
    end

    Start[Read plan, extract tasks] --> A
    K --> More{More tasks?}
    More -->|yes| A
    More -->|no| Finish[Dispatch final code reviewer]
```

## Prompt Templates

- `.agent/skills/superpowers-subagent-driven-development/implementer-prompt.md`
- `.agent/skills/superpowers-subagent-driven-development/spec-reviewer-prompt.md`
- `.agent/skills/superpowers-subagent-driven-development/code-quality-reviewer-prompt.md`

## Example Workflow

```
You: I'm using Subagent-Driven Development to execute this plan.

[Read plan file once: docs/plans/feature-plan.md]
[Extract all 5 tasks with full text and context]
[Create TodoWrite with all tasks]

Task 1: Hook installation script

[Get Task 1 text and context (already extracted)]
[Dispatch implementation subagent with full task text + context]

Implementer: "Before I begin - should the hook be installed at user or system level?"

You: "User level (~/.config/superpowers/hooks/)"

Implementer: "Got it. Implementing now..."
[Later] Implementer:
  - Implemented install-hook command
  - Added tests, 5/5 passing
  - Self-review: Found I missed --force flag, added it
  - Committed

[Dispatch spec compliance reviewer]
Spec reviewer: ✅ Spec compliant - all requirements met, nothing extra

[Get git SHAs, dispatch code quality reviewer]
Code reviewer: Strengths: Good test coverage, clean. Issues: None. Approved.

[Mark Task 1 complete]

Task 2: Recovery modes

[Get Task 2 text and context (already extracted)]
[Dispatch implementation subagent with full task text + context]

Implementer: [No questions, proceeds]
Implementer:
  - Added verify/repair modes
  - 8/8 tests passing
  - Self-review: All good
  - Committed

[Dispatch spec compliance reviewer]
Spec reviewer: ❌ Issues:
  - Missing: Progress reporting (spec says "report every 100 items")
  - Extra: Added --json flag (not requested)

[Implementer fixes issues]
Implementer: Removed --json flag, added progress reporting

[Spec reviewer reviews again]
Spec reviewer: ✅ Spec compliant now

[Dispatch code quality reviewer]
Code reviewer: Strengths: Solid. Issues (Important): Magic number (100)

[Implementer fixes]
Implementer: Extracted PROGRESS_INTERVAL constant

[Code reviewer reviews again]
Code reviewer: ✅ Approved

[Mark Task 2 complete]

...

[After all tasks]
[Dispatch final code-reviewer]
Final reviewer: All requirements met, ready to merge

Done!
```

## Advantages

**vs. Manual execution:**

- Subagents follow TDD naturally
- Fresh context per task
- Parallel-safe
- Subagent can ask questions (before AND during work)

**vs. Executing Plans:**

- Same session (no handoff)
- Continuous progress
- Review checkpoints automatic
- **Quality gates:**
    - Self-review catches issues before handoff
    - Two-stage review: spec compliance preventing over-building, then code quality
    - Review loops ensure fixes actually work

## Red Flags

**Never:**

- Skip reviews (spec compliance OR code quality)
- Proceed with unfixed issues
- Dispatch multiple implementation subagents in parallel (conflicts)
- Make subagent read plan file (provide full text instead)
- Skip scene-setting context (subagent needs to understand where task fits)
- Ignore subagent questions (answer before letting them proceed)
- Accept "close enough" on spec compliance (spec reviewer found issues = not done)
- Skip review loops (reviewer found issues = implementer fixes = review again)
- Let implementer self-review replace actual review (both are needed)
- **Start code quality review before spec compliance is ✅** (wrong order)
- Move to next task while either review has open issues

**If subagent asks questions:**
- Answer clearly and completely
- Provide additional context if needed
- Don't rush them into implementation

**If reviewer finds issues:**
- Implementer (same subagent) fixes them
- Reviewer reviews again
- Repeat until approved
- Don't skip the re-review

**If subagent fails task:**
- Dispatch fix subagent with specific instructions
- Don't try to fix manually (context pollution)

## Integration

**Required workflow skills:**

- **superpowers:superpowers-plan** - Creates the plan this skill executes
- **superpowers:superpowers-review** - Code review template for reviewer subagents
- **superpowers:superpowers-finish** - Complete development after all tasks

**Subagents should use:**

- **superpowers:superpowers-tdd** - Subagents follow TDD for each task

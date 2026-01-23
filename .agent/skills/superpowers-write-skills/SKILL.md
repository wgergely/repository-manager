---
name: superpowers-write-skills
description: Use when creating new skills, editing existing skills, or verifying skills work before deployment
---

# Writing Skills

## Overview

**Writing skills IS Test-Driven Development applied to process documentation.**

## Core Principles

1. **Agent Search Optimization (ASO):** Write descriptions that trigger correctly.
2. **Rationalization Prevention:** Explicitly close loopholes agents use to be lazy.
3. **TDD for Docs:** Red (baseline failure) -> Green (skill fixes it) -> Refactor.

## Directory Structure

```
.agent/skills/
  superpowers-skill-name/
    SKILL.md              # Main reference (required)
    scripts/              # Helper scripts (optional)
```

## SKILL.md Structure & ASO

**Critical:** The `description` field determines *when* the agent loads the skill.

**Format:**
```yaml
description: Use when [specific triggering conditions and symptoms]
```

**ASO Rules:**
- **Start with "Use when..."**
- **Describe the PROBLEM, not the solution.**
- **NEVER summarize the workflow.** (If you summarize it, the agent will just read the summary and hallucinate the rest, skipping your detailed instructions).

**Bad:**
`description: Workflow for TDD that does red-green-refactor.` -> Agent sees "red-green-refactor", thinks "I know that!", and ignores the file.

**Good:**
`description: Use when implementing logic to ensure correctness and prevent regressions.` -> Agent thinks "I am implementing logic, I need this."

## Rationalization Prevention

Agents are smart but lazy. They will find "loophole" reasons to skip hard work (like tests). You must predict and ban them.

**Add a "Rationalization Table" to your skill:**

```markdown
| Excuse                        | Reality                                                             |
| ----------------------------- | ------------------------------------------------------------------- |
| "Too simple to test"          | Simple code breaks. Test takes 30 seconds.                          |
| "I'll test after"             | Tests passing immediately prove nothing.                            |
| "Tests verify the same thing" | Tests-after verify implementation, Tests-first verify requirements. |
```

**Add a "Red Flags" section:**
```markdown
## Red Flags - STOP and Start Over
- Code before test
- "I already manually tested it"
- "This is different because..."
```

## The TDD Cycle for Skills

### 1. RED: Baseline Failure
Run a pressure scenario (e.g., "Implement this fast!") *without* the skill.
- Document: What did the agent skip? (e.g., "Skipped tests because 'simple change'")

### 2. GREEN: Minimal Skill
Write the skill to specifically counter that failure.
- Add rule: "No code without failing test."
- Add rationalization counter: "'Simple change' is not an excuse."
- Run scenario *with* skill. Verify compliance.

### 3. REFACTOR: Bulletproofing
If agent finds a new loophole (e.g., "I wrote the test but didn't run it"), close it.

## Skill Creation Checklist

- [ ] **ASO Check:** Description uses "Use when...", describes triggers, does NOT summarize workflow.
- [ ] **Rationalization Check:** Includes table of excuses and reality.
- [ ] **Red Flags:** Explicit list of "Stop" signals.
- [ ] **TDD Verified:** You watched an agent fail without it, and pass with it.
- [ ] **Naming:** `superpowers-lower-kebab-case`.
- [ ] **Formatting:** Markdown headers, clear sections.

## Deployment

After creation, the skill is available immediately in the `.agent/skills` folder. Use `/superpowers-reload` (if available) or simply start a new turn to pick up changes.

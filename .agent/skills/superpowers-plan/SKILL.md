---
name: superpowers-plan
description: Writes an implementation plan with small steps, exact files to touch, and verification commands. Use before making non-trivial changes.
---

# Planning Skill

## Overview

Write comprehensive implementation plans assuming the engineer has zero context for your codebase. Document everything: which files to touch, code snippets, testing strategy, and how to verify. Give them the whole plan as bite-sized tasks. DRY. YAGNI. TDD. Frequent commits.

**Announce at start:** "I'm using the superpowers-plan skill to create the implementation plan."

**Save plans to:** `artifacts/superpowers/plan.md`

## When to Use This Skill

- Any multi-file change
- Any change that impacts behavior, data, auth, billing, or production workflows
- Any debugging that needs systematic isolation

## Bite-Sized Task Granularity

**Each step is one action (2-5 minutes):**
- "Write the failing test" - step
- "Run it to make sure it fails" - step
- "Implement the minimal code to make the test pass" - step
- "Run the tests and make sure they pass" - step
- "Commit" - step

## Plan Document Header

**Every plan MUST start with this header:**

```markdown
# [Feature Name] Implementation Plan

> **For Antigravity:** Use `/superpowers-execute-plan` to implement this plan task-by-task.

**Goal:** [One sentence describing what this builds]

**Architecture:** [2-3 sentences about approach]

**Tech Stack:** [Key technologies/libraries]

---
```

## Task Structure

```markdown
### Task N: [Component Name]

**Files:**
- Create: `exact/path/to/file.py`
- Modify: `exact/path/to/existing.py:123-145`
- Test: `tests/exact/path/to/test.py`

**Step 1: Write the failing test**

```python
def test_specific_behavior():
    result = function(input)
    assert result == expected
```

**Step 2: Run test to verify it fails**

Run: `pytest tests/path/test.py::test_name -v`
Expected: FAIL with "function not defined"

**Step 3: Write minimal implementation**

```python
def function(input):
    return expected
```

**Step 4: Run test to verify it passes**

Run: `pytest tests/path/test.py::test_name -v`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/path/test.py src/path/file.py
git commit -m "feat: add specific feature"
```
```

## Plan Format Summary

### Goal
### Assumptions
### Plan
1. Step name
   - Files: `path/to/file.ext`, `...`
   - Change: (1-2 bullets)
   - Verify: (exact commands or checks)
2. ...

### Risks & Mitigations
### Rollback Plan

## Remember

- Exact file paths always
- Complete code in plan (not "add validation")
- Exact commands with expected output
- DRY, YAGNI, TDD, frequent commits

## Execution Handoff

After saving the plan, offer execution choice:

**"Plan complete and saved to `artifacts/superpowers/plan.md`. Two execution options:**

**1. Sequential** - Run `/superpowers-execute-plan` for step-by-step execution with verification

**2. Parallel** - Run `/superpowers-execute-plan-parallel` if steps are independent (faster!)

**Which approach?"**

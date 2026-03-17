# Sendit + Cleanit Integration — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a cleanit quality gate (review → simplify → guard) to the /sendit pipeline, running between commit and code review.

**Architecture:** Insert Step 2.5 into the existing agent-prompt.md. The cleanit-reviewer subagent reads the cleanit skill, runs the full pipeline on changed files, optionally commits fixes, and returns a PASS/WARN/FAIL verdict. FAIL blocks the pipeline.

**Tech Stack:** Markdown skill files, bash, Agent tool (cleanit-reviewer subagent type)

**Spec:** `docs/superpowers/specs/2026-03-17-sendit-cleanit-integration-design.md`

---

## Chunk 1: Update agent-prompt.md

### Task 1: Insert Step 2.5 into agent-prompt.md

**Files:**
- Modify: `.claude/skills/sendit/references/agent-prompt.md` (between line 134 "---" after Step 2 and line 136 "## Step 3")

- [ ] **Step 1: Read the current file to confirm insertion point**

Read `.claude/skills/sendit/references/agent-prompt.md` and locate the `---` separator between Step 2 (ends ~line 134) and Step 3 (starts ~line 136). The new section goes between them.

- [ ] **Step 2: Insert Step 2.5 section**

Use the Edit tool to insert the following after the `---` that ends Step 2 and before `## Step 3: Full Code Review`:

````markdown

## Step 2.5: Code Quality (cleanit)

**Skip if `--skip-review` was passed.**
**In DRY_RUN mode: print `[DRY-RUN]` and skip.**

Run the full cleanit pipeline (review → simplify → guard) on changed files before code review.

```bash
echo ""
echo "=== Code Quality (cleanit) ==="

if [ "$SKIP_REVIEW" = "true" ]; then
  echo "(Skipped — --skip-review)"
elif [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would spawn cleanit-reviewer subagent against changed files"
else
  CLEANIT_FILES=$(git diff HEAD~1 HEAD --name-only 2>/dev/null)

  if [ -z "$CLEANIT_FILES" ]; then
    echo "(No changed files — skipping)"
  else
    echo "Changed files for cleanit:"
    echo "$CLEANIT_FILES" | sed 's/^/  /'

    # Write file list to temp file for subagent
    CLEANIT_FILES_PATH=$(mktemp /tmp/sendit-cleanit-XXXXXX.txt)
    echo "$CLEANIT_FILES" > "$CLEANIT_FILES_PATH"

    echo "Spawning cleanit reviewer..."
    # Subagent is spawned inline via the Agent tool — see below
  fi
fi
```

**Spawn the cleanit subagent now** (only when not in DRY_RUN, not skipped, and files exist):

```
Agent tool:
  subagent_type: "cleanit-reviewer"
  model: "sonnet"
  run_in_background: false
  description: "Cleanit quality gate for sendit"
  prompt: |
    You are running a code quality gate as part of the /sendit shipping pipeline.
    Project root: /Volumes/base/dev/app/e-fees

    Read the cleanit skill at: ~/.claude/skills/cleanit/SKILL.md
    Read its resource files from: ~/.claude/skills/cleanit/resources/

    Execute the full cleanit pipeline (review → simplify → guard) on the files listed in:
    <CLEANIT_FILES_PATH>

    IMPORTANT — commit ownership: If simplify modifies files and tests pass,
    YOU (the subagent) stage and commit them:

      Scope detection from changed file paths:
        e-fees-api/ → api | e-fees-scope/ → scope | src-tauri/ → tauri | src/ → ui | else → e-fees

      git add [modified files]
      git commit -m "style(<scope>): cleanit fixes

      Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

    If simplify reverts (tests failed after changes), that's OK — return WARN verdict.

    End your response with EXACTLY one verdict line as the very last line:
      VERDICT: PASS    — all files CLEAN or NEEDS_WORK
      VERDICT: WARN    — issues noted but nothing SERIOUS+, or simplify reverted
      VERDICT: FAIL    — at least one file SERIOUS or DISASTER

    Before the verdict, list per-file categories:
      FILE: <path> — <CLEAN|NEEDS_WORK|PROBLEM|SERIOUS|DISASTER>
```

Substitute the actual value of `$CLEANIT_FILES_PATH` from the bash block above into the prompt — it is a real temp file path on disk.

After spawning the subagent, capture its result as `CLEANIT_RESULT`. Then:

```bash
# Print the full cleanit result
echo "$CLEANIT_RESULT"
echo ""

# Parse verdict (last VERDICT: line in output)
CLEANIT_VERDICT=$(echo "$CLEANIT_RESULT" | grep '^VERDICT:' | tail -1 | awk '{print $2}')

if [ "$CLEANIT_VERDICT" = "FAIL" ]; then
  echo ""
  echo "PIPELINE BLOCKED: Cleanit found SERIOUS+ quality issues."
  echo "Fix the issues above, then re-run /sendit."
  rm -f "$CLEANIT_FILES_PATH"
  exit 1
elif [ "$CLEANIT_VERDICT" = "WARN" ]; then
  echo "⚠ Cleanit warnings noted — continuing to code review"
elif [ "$CLEANIT_VERDICT" = "PASS" ]; then
  echo "✓ Code quality check passed"
else
  echo "WARNING: Could not parse cleanit verdict — continuing"
fi

rm -f "$CLEANIT_FILES_PATH"
```

---
````

- [ ] **Step 3: Verify insertion**

Read `.claude/skills/sendit/references/agent-prompt.md` and confirm:
- Step 2.5 appears between Step 2 and Step 3
- Step 3 heading and content are unchanged
- No duplicate `---` separators

### Task 2: Add cleanit verdict to Step 12 Final Report

**Files:**
- Modify: `.claude/skills/sendit/references/agent-prompt.md` (Step 12, ~line 603)

- [ ] **Step 1: Locate the report summary block in Step 12**

Find the line: `echo "  ✓ Code review: ${VERDICT:-skipped}"`

- [ ] **Step 2: Insert cleanit verdict line**

Add immediately after the code review line:

```bash
echo "  ✓ Cleanit: ${CLEANIT_VERDICT:-skipped}"
```

- [ ] **Step 3: Verify**

Read Step 12 and confirm the new line appears after code review and before the version line.

- [ ] **Step 4: Commit**

```bash
git add .claude/skills/sendit/references/agent-prompt.md
git commit -m "feat(sendit): add cleanit quality gate as Step 2.5

Runs full cleanit pipeline (review→simplify→guard) between commit and
code review. SERIOUS+ findings block the pipeline. Auto-commits fixes.

Co-Authored-By: Claude <model> <noreply@anthropic.com>"
```

---

## Chunk 2: Update SKILL.md

### Task 3: Update SKILL.md description, diagram, and help text

**Files:**
- Modify: `.claude/skills/sendit/SKILL.md` (lines 3, 19-20, 55-60)

- [ ] **Step 1: Update frontmatter description (line 3)**

Replace:
```
description: Use when ready to ship staged changes for e-fees — runs full commit→bump→tag→CI→verify pipeline autonomously in background. Handles Forgejo PR/merge, GitHub tag push, CI polling with timed checkins, and update.json sync.
```

With:
```
description: Use when ready to ship staged changes for e-fees — runs full commit→cleanit→review→bump→tag→CI→verify pipeline autonomously in background. Handles code quality checks, Forgejo/GitHub push, CI polling with timed checkins, and update.json sync.
```

- [ ] **Step 2: Update --skip-review help text (line 20)**

Replace:
```
/sendit --skip-review   — Skip automated code review (for trivial/docs changes)
```

With:
```
/sendit --skip-review   — Skip cleanit + code review (for trivial/docs changes)
```

- [ ] **Step 3: Update pipeline diagram (lines 55-60)**

Replace:
```
Pre-flight → Commit → Version bump → Forgejo PR → Review → Merge
    → Tag → Push tag to GitHub → CI poll loop (every 60s)
    → Pull update.json from Forgejo → Push to GitHub → Verify release
    → Report
```

With:
```
Pre-flight → Commit → Cleanit (review→simplify→guard) → Code Review
    → Version bump → Push (Forgejo + GitHub)
    → Tag → CI poll loop (every 60s)
    → Pull update.json from Forgejo → Push to GitHub → Verify release
    → Report
```

- [ ] **Step 4: Verify**

Read `.claude/skills/sendit/SKILL.md` and confirm all three changes are correct.

- [ ] **Step 5: Commit**

```bash
git add .claude/skills/sendit/SKILL.md
git commit -m "docs(sendit): update SKILL.md for cleanit integration

Update description, pipeline diagram, and --skip-review help to reflect
the new cleanit quality gate step.

Co-Authored-By: Claude <model> <noreply@anthropic.com>"
```

---

## Verification

- [ ] **Read both files end-to-end** to confirm no formatting issues
- [ ] **Run `/sendit --dry-run`** to verify the pipeline prints the cleanit dry-run message in the correct position

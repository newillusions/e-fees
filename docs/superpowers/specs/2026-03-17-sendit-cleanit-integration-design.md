# Design: Add cleanit step to /sendit pipeline

**Date:** 2026-03-17
**Status:** Approved
**Scope:** `.claude/skills/sendit/` (SKILL.md + references/agent-prompt.md)

## Problem

The `/sendit` pipeline commits, reviews for bugs/security, bumps version, and ships — but has no code quality gate. Structural issues (god functions, DRY violations, complexity) pass through unchecked. The `/cleanit` skill exists to catch and fix these, but isn't integrated into the shipping workflow.

## Solution

Insert a new **Step 2.5: Code Quality (cleanit)** between Step 2 (Commit) and Step 3 (Code Review) in the sendit agent-prompt.

### Updated pipeline flow

```
Pre-flight → Commit → Cleanit → Code Review → Version bump → Push → Tag → CI → Sync → Verify → Report
```

### Step 2.5 specification

**Heading in agent-prompt.md:** `## Step 2.5: Code Quality (cleanit)` — inserted between Step 2 and Step 3. All existing step numbers remain unchanged.

**Trigger:** Runs after commit, before code review.

**Skip conditions:**
- `--skip-review` skips this step (same flag governs both quality gates)
- `--dry-run` prints `[DRY-RUN] Would spawn cleanit-reviewer subagent against changed files` and continues
- No changed files → skip

**Execution:**

1. Build the file list via bash (same pattern as Step 3's temp files):

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

2. **Spawn the cleanit subagent now** (only when not skipped/dry-run and files exist):

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

After spawning the subagent, capture its result as `CLEANIT_RESULT`.

3. **Capture and parse the result** (after subagent returns):

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

### SKILL.md updates

**1. Update the pipeline diagram** (lines 55-60). Replace:

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

**2. Update the description** (line 3 frontmatter). Replace:

```
description: Use when ready to ship staged changes for e-fees — runs full commit→bump→tag→CI→verify pipeline autonomously in background. Handles Forgejo PR/merge, GitHub tag push, CI polling with timed checkins, and update.json sync.
```

With:

```
description: Use when ready to ship staged changes for e-fees — runs full commit→cleanit→review→bump→tag→CI→verify pipeline autonomously in background. Handles code quality checks, Forgejo/GitHub push, CI polling with timed checkins, and update.json sync.
```

**3. Update `--skip-review` description** (line 20). Replace:

```
/sendit --skip-review   — Skip automated code review (for trivial/docs changes)
```

With:

```
/sendit --skip-review   — Skip cleanit + code review (for trivial/docs changes)
```

### Files to modify

| File | Change |
|------|--------|
| `.claude/skills/sendit/references/agent-prompt.md` | Add Step 2.5 section (heading, bash, subagent spawn, result capture) between Steps 2 and 3 |
| `.claude/skills/sendit/SKILL.md` | Update description, pipeline diagram, and --skip-review help text as specified above |

**4. Update Step 12 (Final Report)** — add cleanit verdict to the pipeline summary block. After the existing `echo "  ✓ Code review: ${VERDICT:-skipped}"` line, add:

```bash
echo "  ✓ Cleanit: ${CLEANIT_VERDICT:-skipped}"
```

### What doesn't change

- All existing steps (1-12) remain identical in content and numbering (except Step 12 gets one extra echo line)
- No new CLI arguments or flags
- No changes to pre-flight, version bump, push, tag, CI, sync, or verify steps
- The code review step (Step 3) continues to run after cleanit

## Edge cases

- **No changed files after commit:** Step 2.5 is skipped (same as code review behavior)
- **Cleanit simplify reverts due to test failure:** Cleanit handles revert internally (built into its Phase 2). Subagent returns WARN. Pipeline continues to Step 3.
- **Cleanit simplify + code review both find issues:** Cleanit fixes structural issues first; code review then catches bugs/security on the cleaned code. This is the intended benefit of ordering cleanit before review.

## Alternatives considered

1. **Guard-only (no auto-fix):** Cheaper but doesn't fix anything — just blocks. Rejected because the value of cleanit is its ability to auto-fix.
2. **Replace code review with cleanit:** Cleanit doesn't check for bugs, security, or SurrealDB-specific issues. Both gates serve different purposes. Rejected.

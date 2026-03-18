# E-Fees Sendit — Autonomous Background Pipeline

You are running the e-fees ship-it pipeline autonomously. No user interaction is available.
Execute every step. Log progress throughout. Deliver a complete report at the end.

**Project root:** `/Volumes/base/dev/app/e-fees`
**All commands run from the project root unless otherwise stated.**

---

## Step 0: Parse Args

Read the `## Args` section at the bottom of this prompt. Extract:

- **COMMIT_TYPE_OVERRIDE**: `fix`, `feat`, `docs`, `chore`, `refactor`, `test` — or empty (auto-detect)
- **BUMP_OVERRIDE**: `patch`, `minor`, `major` — or empty (auto-detect from commit type)
- **DRY_RUN**: true if `--dry-run` present
- **SKIP_REVIEW**: true if `--skip-review` present
- **SKIP_PUBLISH**: true if `--skip-publish` present (stops after code review, no tag/CI)

In DRY_RUN mode: print every command with `[DRY-RUN]` prefix instead of executing. Read files normally.

---

## Step 1: Pre-Flight Checks

Run ALL checks before any changes. Fail fast with clear error.

```bash
cd /Volumes/base/dev/app/e-fees

# 1. Check working directory
echo "=== Pre-Flight ==="

# Branch check
BRANCH=$(git branch --show-current)
echo "Branch: $BRANCH"
if [ "$BRANCH" != "main" ]; then
  echo "WARNING: Not on main branch (on $BRANCH). Continuing — will push to $BRANCH."
fi

# Remotes reachable
git ls-remote origin HEAD > /dev/null 2>&1 && echo "✓ origin (Forgejo) reachable" || { echo "FAIL: origin not reachable"; exit 1; }
git ls-remote github HEAD > /dev/null 2>&1 && echo "✓ github reachable" || { echo "FAIL: github remote not reachable"; exit 1; }

# gh CLI authenticated
gh auth status > /dev/null 2>&1 && echo "✓ gh CLI authenticated" || { echo "FAIL: gh CLI not authenticated (run: gh auth login)"; exit 1; }

# Node + cargo available
node --version > /dev/null 2>&1 && echo "✓ node available" || { echo "FAIL: node not found"; exit 1; }
cargo --version > /dev/null 2>&1 && echo "✓ cargo available" || { echo "FAIL: cargo not found"; exit 1; }

# Check what's staged
STAGED=$(git diff --cached --name-only)
echo ""
echo "Staged files:"
if [ -z "$STAGED" ]; then
  echo "  (nothing staged)"
else
  echo "$STAGED" | sed 's/^/  /'
fi

# Current version
CURRENT_VERSION=$(node -p "require('./package.json').version")
echo ""
echo "Current version: $CURRENT_VERSION"
echo "=== Pre-Flight OK ==="
```

---

## Step 2: Commit Staged Changes

**Skip this step if nothing is staged.**

```bash
# Detect commit type from diff (if no override provided)
if [ -n "$COMMIT_TYPE_OVERRIDE" ]; then
  COMMIT_TYPE="$COMMIT_TYPE_OVERRIDE"
else
  DIFF_STAT=$(git diff --cached --stat)
  DIFF_FILES=$(git diff --cached --name-only)

  # Auto-detect type
  if echo "$DIFF_FILES" | grep -qE '\.(test|spec)\.(ts|rs|js)$'; then
    COMMIT_TYPE="test"
  elif echo "$DIFF_FILES" | grep -qE '^(docs?/|README|CHANGELOG|\.md$)'; then
    COMMIT_TYPE="docs"
  elif echo "$DIFF_FILES" | grep -qE '^(\.github/|Cargo\.toml|package\.json|tauri\.conf\.json|vite\.config|tsconfig)'; then
    COMMIT_TYPE="chore"
  elif echo "$DIFF_FILES" | grep -qE '\.(svelte|ts|js)$' && ! echo "$DIFF_FILES" | grep -qE 'src-tauri'; then
    COMMIT_TYPE="feat"
  elif echo "$DIFF_FILES" | grep -qE '(src-tauri.*\.rs$)'; then
    COMMIT_TYPE="feat"
  else
    COMMIT_TYPE="chore"
  fi
fi

# Derive scope from most-changed area
SCOPE="e-fees"
if git diff --cached --name-only | grep -q "^e-fees-api/"; then SCOPE="api"; fi
if git diff --cached --name-only | grep -q "^e-fees-scope/"; then SCOPE="scope"; fi
if git diff --cached --name-only | grep -q "^src-tauri/"; then SCOPE="tauri"; fi
if git diff --cached --name-only | grep -q "^src/"; then SCOPE="ui"; fi

# Generate description from diff
CHANGED_FILES=$(git diff --cached --name-only | wc -l | tr -d ' ')
FIRST_FILE=$(git diff --cached --name-only | head -1 | xargs basename 2>/dev/null || echo "various")

echo ""
echo "=== Commit ==="
echo "Type: $COMMIT_TYPE | Scope: $SCOPE | Files: $CHANGED_FILES"

# Ask for description based on diff summary (read-only, no interaction)
# Generate from diff context
COMMIT_DESC="update $FIRST_FILE"
if [ "$CHANGED_FILES" -gt 3 ]; then
  COMMIT_DESC="update $SCOPE components ($CHANGED_FILES files)"
fi

COMMIT_MSG="${COMMIT_TYPE}(${SCOPE}): ${COMMIT_DESC}

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "Message: $COMMIT_MSG"
echo ""

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git commit -m \"$COMMIT_MSG\""
else
  git commit -m "$COMMIT_MSG"
fi
```

---

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

Use the Agent tool. Substitute the actual value of `$CLEANIT_FILES_PATH` from the bash block above into the prompt — it is a real temp file path on disk.

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

## Step 3: Full Code Review

**Skip if `--skip-review` was passed.**
**In DRY_RUN mode: print `[DRY-RUN]` and skip.**

Spawn a code-reviewer subagent with the full diff and file context. The reviewer checks for:
- **Bugs**: logic errors, off-by-one, null/None handling, error propagation, incorrect SurrealDB patterns
- **Security**: hardcoded secrets, injection risks, unsafe blocks, exposed credentials, insecure defaults
- **Structure**: dead code, debug statements left in, TODO/FIXME markers, overly complex logic, missing error handling
- **Red team**: what could go wrong at runtime? Data loss scenarios? Breaking changes to existing records?

The reviewer returns a structured verdict. CRITICAL findings block the pipeline. Warnings are logged and execution continues.

```bash
echo "=== Code Review ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would spawn code-reviewer subagent against git diff HEAD~1 HEAD"
else
  # Get the diff and changed file list for the reviewer
  DIFF=$(git diff HEAD~1 HEAD 2>/dev/null || git diff --cached)
  CHANGED_FILES=$(git diff HEAD~1 HEAD --name-only 2>/dev/null || git diff --cached --name-only)

  if [ -z "$DIFF" ]; then
    echo "(No diff to review — skipping)"
  else
    echo "Spawning code reviewer..."
    # Write diff to temp file so subagent can read it
    DIFF_FILE=$(mktemp /tmp/sendit-review-XXXXXX.diff)
    echo "$DIFF" > "$DIFF_FILE"
    FILES_FILE=$(mktemp /tmp/sendit-files-XXXXXX.txt)
    echo "$CHANGED_FILES" > "$FILES_FILE"

    # Subagent is spawned inline via the Agent tool — see below
    # Result is captured and parsed for VERDICT line
  fi
fi
```

**Spawn the reviewer subagent now** (only when not in DRY_RUN and diff is non-empty):

Use the Agent tool. Substitute the actual values of `$DIFF_FILE` and `$FILES_FILE` from the bash block above into the prompt — they are real temp file paths on disk.

```
subagent_type: "general-purpose"
model: "sonnet"
run_in_background: false
description: "Code review for sendit pipeline"
prompt: |
  You are a senior code reviewer performing a pre-release review for the e-fees project
  (Tauri 2 + Svelte 5 + Rust + SurrealDB). Your job is to catch real problems before they ship.
  Be specific — cite file and line where possible.

  ## What to check

  **Bugs**
  - Logic errors, off-by-one, incorrect conditionals
  - Null/None/Option handling (SurrealDB v3: NULL != NONE — only NONE is valid for option<T>)
  - Error propagation — are errors being silently swallowed?
  - Async/await correctness in Rust (missing .await, incorrect lifetimes)
  - Svelte 5 runes correctness ($state, $derived, $effect — not legacy $:)

  **Security**
  - Hardcoded secrets, API keys, passwords, tokens in new lines
  - SurrealQL injection risks (string-interpolated queries, not parameterized)
  - Unsafe Rust blocks without justification comment
  - Credentials logged or exposed in error messages

  **Structure**
  - console.log / println! / dbg! / eprintln! left in production paths
  - TODO/FIXME/HACK in new code
  - Dead code added but never called
  - Missing error handling at Tauri command boundaries (should return Result<T, String>)

  **Red team — what could go wrong at runtime?**
  - Could this break existing database records or schema integrity?
  - Could this cause data loss or silent corruption?
  - Are there breaking changes to Tauri IPC commands that would break the currently-installed app?
  - Race conditions or shared state issues in Svelte stores?
  - Edge cases with empty arrays, zero values, missing optional fields?

  ## Diff to review

  Read the diff from: <DIFF_FILE_PATH>
  Read the changed file list from: <FILES_FILE_PATH>

  For any changed file where the diff alone is insufficient to judge impact,
  read the full source file for context.

  ## Output format (REQUIRED — pipeline parses this)

  List all findings first:
    CRITICAL: <description>  ← use for anything that must be fixed before shipping
    WARNING: <description>   ← use for issues worth noting but not release-blocking
    INFO: <description>      ← use for minor observations

  End your response with EXACTLY one verdict line as the very last line:
    VERDICT: PASS    ← code is clean and ready to ship
    VERDICT: WARN    ← issues noted but nothing release-blocking
    VERDICT: BLOCK   ← at least one CRITICAL issue, must fix before shipping
```

After spawning the subagent, capture its result as `REVIEW_RESULT`. Then:

```bash
# Print the full review
echo "$REVIEW_RESULT"
echo ""

# Parse verdict (last VERDICT: line in output)
VERDICT=$(echo "$REVIEW_RESULT" | grep '^VERDICT:' | tail -1 | awk '{print $2}')

if [ "$VERDICT" = "BLOCK" ]; then
  echo ""
  echo "PIPELINE BLOCKED: Code review found critical issues."
  echo "Fix the issues above, then re-run /sendit."
  rm -f "$DIFF_FILE" "$FILES_FILE"
  exit 1
elif [ "$VERDICT" = "WARN" ]; then
  echo "⚠ Review warnings noted — continuing to publish"
elif [ "$VERDICT" = "PASS" ]; then
  echo "✓ Code review passed"
else
  echo "WARNING: Could not parse review verdict — continuing"
fi

rm -f "$DIFF_FILE" "$FILES_FILE"
```

---

## Step 4: Version Bump

Determine bump type: `feat:` → minor, everything else → patch. `BUMP_OVERRIDE` takes precedence.

```bash
echo ""
echo "=== Version Bump ==="

# Determine bump
if [ -n "$BUMP_OVERRIDE" ]; then
  BUMP_TYPE="$BUMP_OVERRIDE"
elif [ "$COMMIT_TYPE" = "feat" ]; then
  BUMP_TYPE="minor"
else
  BUMP_TYPE="patch"
fi

echo "Bump type: $BUMP_TYPE"

if [ "$DRY_RUN" = "true" ]; then
  # Show what the new version would be
  CURRENT=$(node -p "require('./package.json').version")
  echo "[DRY-RUN] Would bump $CURRENT ($BUMP_TYPE)"
  NEW_VERSION="$CURRENT-dry-run"
else
  # Bump package.json
  npm version "$BUMP_TYPE" --no-git-tag-version 2>&1

  # Sync to Cargo.toml + tauri.conf.json (including window title)
  node scripts/sync-version.cjs

  NEW_VERSION=$(node -p "require('./package.json').version")
  echo "New version: $NEW_VERSION"

  # Verify all 3 files updated
  echo "Verifying:"
  echo "  package.json: $(node -p "require('./package.json').version")"
  grep '^version' src-tauri/Cargo.toml | head -1
  grep '"version"' src-tauri/tauri.conf.json | head -1
  grep '"title"' src-tauri/tauri.conf.json
fi
```

---

## Step 5: Commit Version Bump

```bash
echo ""
echo "=== Commit Version Bump ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json"
  echo "[DRY-RUN] git commit -m \"chore: bump version to \$NEW_VERSION\""
else
  git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
  git commit -m "chore: bump version to $NEW_VERSION

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
  echo "✓ Version bump committed"
fi
```

---

## Step 6: Push to Forgejo and GitHub

```bash
echo ""
echo "=== Push ==="
BRANCH=$(git branch --show-current)

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git push origin $BRANCH"
  echo "[DRY-RUN] git push github $BRANCH"
else
  git push origin "$BRANCH" && echo "✓ Forgejo (origin) pushed" || { echo "FAIL: Forgejo push rejected"; echo "Hint: git pull origin $BRANCH --rebase, then retry"; exit 1; }
  git push github "$BRANCH" && echo "✓ GitHub pushed" || { echo "FAIL: GitHub push rejected"; exit 1; }
fi
```

**Stop here if `--skip-publish` was passed.** Report what was done and exit.

---

## Step 7: Create and Push Tag

```bash
echo ""
echo "=== Tag ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git tag v\$NEW_VERSION"
  echo "[DRY-RUN] git push origin v\$NEW_VERSION"
  echo "[DRY-RUN] git push github v\$NEW_VERSION"
  echo "[DRY-RUN] GitHub Actions build would trigger on tag push"
else
  git tag "v$NEW_VERSION"
  git push origin "v$NEW_VERSION" && echo "✓ Tag pushed to Forgejo"
  git push github "v$NEW_VERSION" && echo "✓ Tag pushed to GitHub — CI triggered"
  echo ""
  echo "GitHub Actions: https://github.com/newillusions/e-fees/actions"
fi
```

---

## Step 8: CI Poll Loop — Timed Checkins

Poll GitHub Actions every 60 seconds until the build completes.
**Expected build time: 15–25 minutes.**
All checkins are logged and included in the final report.

```bash
echo ""
echo "=== CI Build Monitoring ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would poll gh run list --repo newillusions/e-fees every 60s"
  echo "[DRY-RUN] Waiting for: macOS aarch64, macOS x64, Windows jobs to complete"
  CI_RESULT="success (dry-run)"
else
  # Wait for GitHub to register the tag push
  echo "Waiting 20s for GitHub to register tag..."
  sleep 20

  # Get run ID
  RUN_ID=""
  ATTEMPTS=0
  while [ -z "$RUN_ID" ] && [ "$ATTEMPTS" -lt 5 ]; do
    RUN_ID=$(gh run list --repo newillusions/e-fees --limit 3 --json databaseId,headBranch,event \
      --jq ".[] | select(.event==\"push\" or .event==\"create\") | .databaseId" 2>/dev/null | head -1)
    if [ -z "$RUN_ID" ]; then
      ATTEMPTS=$((ATTEMPTS + 1))
      echo "Run not found yet, waiting 15s... (attempt $ATTEMPTS/5)"
      sleep 15
    fi
  done

  if [ -z "$RUN_ID" ]; then
    echo "WARNING: Could not find GitHub Actions run. Check manually:"
    echo "  https://github.com/newillusions/e-fees/actions"
    CI_RESULT="unknown (run not found)"
  else
    echo "Run ID: $RUN_ID"
    echo "https://github.com/newillusions/e-fees/actions/runs/$RUN_ID"
    echo ""

    # Poll loop
    POLL_COUNT=0
    START_TIME=$(date +%s)
    CI_RESULT=""

    while true; do
      POLL_COUNT=$((POLL_COUNT + 1))
      ELAPSED=$(( ($(date +%s) - START_TIME) / 60 ))
      TIMESTAMP=$(date '+%H:%M:%S')

      STATUS=$(gh run view "$RUN_ID" --repo newillusions/e-fees --json status,conclusion \
        --jq '"status=\(.status) conclusion=\(.conclusion // "pending")"' 2>/dev/null)

      JOBS=$(gh run view "$RUN_ID" --repo newillusions/e-fees --json jobs \
        --jq '.jobs[] | "  \(.name): \(.status) \(.conclusion // "")"' 2>/dev/null | head -10)

      echo "[$TIMESTAMP +${ELAPSED}min] Checkin #$POLL_COUNT: $STATUS"
      echo "$JOBS"
      echo ""

      CONCLUSION=$(gh run view "$RUN_ID" --repo newillusions/e-fees --json conclusion \
        --jq '.conclusion // "pending"' 2>/dev/null)

      if [ "$CONCLUSION" = "success" ]; then
        CI_RESULT="success"
        echo "✓ CI build PASSED at +${ELAPSED}min"
        break
      elif [ "$CONCLUSION" = "failure" ] || [ "$CONCLUSION" = "cancelled" ]; then
        CI_RESULT="$CONCLUSION"
        echo "✗ CI build $CONCLUSION at +${ELAPSED}min"
        echo ""
        echo "Failed job logs:"
        gh run view "$RUN_ID" --repo newillusions/e-fees --log-failed 2>/dev/null | tail -50
        break
      fi

      # Timeout after 40 minutes
      if [ "$ELAPSED" -gt 40 ]; then
        CI_RESULT="timeout"
        echo "WARNING: Build timed out after 40min. Check manually."
        break
      fi

      sleep 60
    done
  fi

  if [ "$CI_RESULT" != "success" ]; then
    echo ""
    echo "PIPELINE STOPPED: CI build $CI_RESULT"
    echo "GitHub Actions: https://github.com/newillusions/e-fees/actions/runs/$RUN_ID"
    echo "Manual fix: see release.md troubleshooting section"
    exit 1
  fi
fi
```

---

## Step 9: Sync update.json to GitHub

CI pushes update.json to Forgejo via API commit. Must pull and push to GitHub so Tauri updater works.

```bash
echo ""
echo "=== Sync update.json ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git pull origin main"
  echo "[DRY-RUN] Verify update.json has correct content"
  echo "[DRY-RUN] git push github main"
  echo "[DRY-RUN] curl raw.githubusercontent.com to verify"
else
  git pull origin main && echo "✓ Pulled update.json from Forgejo"

  # Verify update.json exists and has correct version
  if [ -f "update.json" ]; then
    UPDATE_VERSION=$(python3 -c "import json,sys; d=json.load(open('update.json')); print(d.get('version','missing'))" 2>/dev/null)
    echo "update.json version: $UPDATE_VERSION"
    if [ "$UPDATE_VERSION" != "$NEW_VERSION" ]; then
      echo "WARNING: update.json version ($UPDATE_VERSION) != release version ($NEW_VERSION)"
    fi
  else
    echo "WARNING: update.json not found after pull"
  fi

  git push github main && echo "✓ update.json pushed to GitHub"

  # Verify endpoint is live (may take ~2min for GitHub CDN)
  sleep 30
  ENDPOINT_VERSION=$(curl -sf "https://raw.githubusercontent.com/newillusions/e-fees/main/update.json" \
    | python3 -c "import json,sys; print(json.load(sys.stdin).get('version','error'))" 2>/dev/null || echo "not yet available")
  echo "Endpoint version: $ENDPOINT_VERSION"
fi
```

---

## Step 10: Verify Forgejo Release

```bash
echo ""
echo "=== Verify Forgejo Release ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] curl forge.mms.name/api/v1/repos/emittiv/fee-prop/releases/tags/v\$NEW_VERSION"
  RELEASE_STATUS="dry-run"
else
  RELEASE_JSON=$(curl -sf \
    "https://forge.mms.name/api/v1/repos/emittiv/fee-prop/releases/tags/v$NEW_VERSION" \
    2>/dev/null)

  if [ -z "$RELEASE_JSON" ]; then
    echo "WARNING: Release not found yet (may still be uploading). Check:"
    echo "  https://forge.mms.name/emittiv/fee-prop/releases"
    RELEASE_STATUS="pending"
  else
    ASSET_COUNT=$(echo "$RELEASE_JSON" | python3 -c "
import json, sys
r = json.load(sys.stdin)
assets = r.get('assets', [])
print(f'Release: {r[\"name\"]}')
print(f'Assets ({len(assets)}):')
for a in assets:
    print(f'  {a[\"name\"]} ({a[\"size\"]/1024/1024:.1f} MB)')
print(len(assets))
" 2>/dev/null | tail -1)

    echo "Asset count: $ASSET_COUNT"
    if [ "$ASSET_COUNT" -ge 6 ]; then
      echo "✓ All platform assets present"
      RELEASE_STATUS="success"
    else
      echo "WARNING: Expected 6+ assets, found $ASSET_COUNT"
      RELEASE_STATUS="incomplete"
    fi
  fi
fi
```

---

## Step 11: Save KB Observation

Save a release observation via SurrealDB HTTP API (MCP unavailable in background agents).

```bash
if [ "$DRY_RUN" != "true" ] && [ "$CI_RESULT" = "success" ]; then
  source ~/.kb-agent.env 2>/dev/null || true

  if [ -n "$SURREALDB_URL" ] && [ -n "$SURREALDB_PASS" ]; then
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    curl -s -X POST "${SURREALDB_URL}/sql" \
      -u "martin:${SURREALDB_PASS}" \
      -H "surreal-ns: kb" \
      -H "surreal-db: knowledge" \
      -H "Accept: application/json" \
      -H "Content-Type: text/plain" \
      -d "INSERT INTO observation (entity_name, content, confidence, status, created_at) VALUES (
        'E-Fees Release',
        'Released e-fees v${NEW_VERSION} via /sendit. CI: success. Platforms: macOS aarch64, macOS x64, Windows.',
        0.9,
        'active',
        '${TIMESTAMP}'
      );" > /dev/null 2>&1 && echo "✓ KB observation saved" || echo "(KB save skipped — not critical)"
  fi
fi
```

---

## Step 12: Final Report

```bash
echo ""
echo "================================================"
echo "  /sendit Complete — E-Fees v${NEW_VERSION:-DRY-RUN}"
echo "================================================"
echo ""
echo "Pipeline Summary:"
echo ""
if [ -n "$STAGED" ] && [ "$STAGED" != "" ]; then
  echo "  ✓ Committed: ${COMMIT_TYPE}(${SCOPE}): ${COMMIT_DESC}"
else
  echo "  - No staged changes committed"
fi
echo "  ✓ Cleanit: ${CLEANIT_VERDICT:-skipped}"
echo "  ✓ Code review: ${VERDICT:-skipped}"
echo "  ✓ Version: ${CURRENT_VERSION} → ${NEW_VERSION}"
echo "  ✓ Pushed: Forgejo (origin/main) + GitHub (github/main)"

if [ "$SKIP_PUBLISH" = "true" ]; then
  echo "  - Tag/CI: skipped (--skip-publish)"
else
  echo "  ✓ Tag: v${NEW_VERSION} pushed to both remotes"
  echo "  ✓ CI Build: $CI_RESULT"
  echo "  ✓ Update endpoint: https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"
  echo "  ✓ Release assets: $RELEASE_STATUS"
fi
echo ""
echo "Links:"
echo "  Forgejo release: https://forge.mms.name/emittiv/fee-prop/releases/tag/v${NEW_VERSION}"
echo "  GitHub Actions:  https://github.com/newillusions/e-fees/actions"
echo "  Update endpoint: https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"
echo ""
if [ "$DRY_RUN" = "true" ]; then
  echo "  ↑ DRY-RUN mode — no changes were made."
fi
echo "================================================"
```

---

## Args

(Populated by SKILL.md when spawning this agent — args follow below)

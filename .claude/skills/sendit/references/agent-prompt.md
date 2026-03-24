# E-Fees Sendit — Autonomous Background Pipeline

You are running the e-fees ship-it pipeline autonomously. No user interaction is available.
Execute every step. Log progress throughout. Deliver a complete report at the end.

**Project root:** `/Volumes/base/dev/app/e-fees`
**All commands run from the project root unless otherwise stated.**

**Token discipline:** Complete the full pipeline. Do NOT read entire source files during
review — use the diff only. If you need function context, read only the specific lines
(offset + limit). Budget: review ≤ 15K tokens, entire pipeline ≤ 50K tokens.

---

## Step 0: Parse Args

Read the `## Args` section at the bottom of this prompt. Extract:

- **COMMIT_TYPE_OVERRIDE**: `fix`, `feat`, `docs`, `chore`, `refactor`, `test` — or empty (auto-detect)
- **BUMP_OVERRIDE**: `patch`, `minor`, `major` — or empty (auto-detect from commit type)
- **DRY_RUN**: true if `--dry-run` present
- **SKIP_REVIEW**: true if `--skip-review` present
- **SKIP_PUBLISH**: true if `--skip-publish` present (stops after test gate, no tag/CI)

In DRY_RUN mode: print every command with `[DRY-RUN]` prefix instead of executing. Read files normally.

---

## Step 1: Pre-Flight Checks

Run ALL checks before any changes. Fail fast with clear error.

```bash
cd /Volumes/base/dev/app/e-fees

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
gh auth status > /dev/null 2>&1 && echo "✓ gh CLI authenticated" || { echo "FAIL: gh CLI not authenticated"; exit 1; }

# Node + cargo available
node --version > /dev/null 2>&1 && echo "✓ node available" || { echo "FAIL: node not found"; exit 1; }
cargo --version > /dev/null 2>&1 && echo "✓ cargo available" || { echo "FAIL: cargo not found"; exit 1; }

# Smoke test recency check
SMOKE_FILE="/Volumes/base/dev/app/e-fees/.claude/last-smoke-test"
if [ -f "$SMOKE_FILE" ]; then
  SMOKE_AGE=$(( ($(date +%s) - $(stat -f %m "$SMOKE_FILE" 2>/dev/null || echo 0)) / 60 ))
  if [ "$SMOKE_AGE" -gt 120 ]; then
    echo "WARNING: Smoke tests last ran ${SMOKE_AGE}min ago (>2h). Consider running /smoke-test first."
  else
    echo "✓ Smoke tests ran ${SMOKE_AGE}min ago"
  fi
else
  echo "WARNING: No smoke test timestamp found. Consider running /smoke-test first."
fi

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

CHANGED_FILES=$(git diff --cached --name-only | wc -l | tr -d ' ')
FIRST_FILE=$(git diff --cached --name-only | head -1 | xargs basename 2>/dev/null || echo "various")

echo ""
echo "=== Commit ==="
echo "Type: $COMMIT_TYPE | Scope: $SCOPE | Files: $CHANGED_FILES"

COMMIT_DESC="update $FIRST_FILE"
if [ "$CHANGED_FILES" -gt 3 ]; then
  COMMIT_DESC="update $SCOPE components ($CHANGED_FILES files)"
fi

COMMIT_MSG="${COMMIT_TYPE}(${SCOPE}): ${COMMIT_DESC}

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"

echo "Message: $COMMIT_MSG"

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git commit"
else
  git commit -m "$COMMIT_MSG"
fi
```

---

## Step 3: Review + Auto-Fix (Single Pass)

**Skip if `--skip-review` was passed.**
**In DRY_RUN mode: print `[DRY-RUN]` and skip.**

This is a single combined review that replaces the old separate cleanit + code review steps.
The reviewer works from the **diff only** — no reading entire source files.

```bash
echo ""
echo "=== Code Review ==="

if [ "$SKIP_REVIEW" = "true" ]; then
  echo "(Skipped — --skip-review)"
  REVIEW_VERDICT="skipped"
elif [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would review git diff HEAD~1 HEAD"
  REVIEW_VERDICT="dry-run"
else
  DIFF=$(git diff HEAD~1 HEAD 2>/dev/null || git diff --cached)
  CHANGED_FILES_LIST=$(git diff HEAD~1 HEAD --name-only 2>/dev/null || git diff --cached --name-only)

  if [ -z "$DIFF" ]; then
    echo "(No diff to review — skipping)"
    REVIEW_VERDICT="skipped"
  else
    DIFF_FILE=$(mktemp /tmp/sendit-review-XXXXXX.diff)
    echo "$DIFF" > "$DIFF_FILE"
    echo "Changed files:"
    echo "$CHANGED_FILES_LIST" | sed 's/^/  /'
    echo ""
    echo "Spawning reviewer..."
  fi
fi
```

**Spawn the reviewer sub-agent now** (only when not DRY_RUN, not skipped, and diff exists).
Use the Agent tool with the actual `$DIFF_FILE` path substituted in:

```
Agent tool:
  subagent_type: "general-purpose"
  model: "haiku"
  run_in_background: false
  description: "Sendit code review"
  prompt: |
    You are a fast code reviewer for the e-fees project (Tauri 2 + Svelte 5 + Rust + SurrealDB).
    Your job: catch real bugs and security issues. Be fast and focused — diff only.

    TOKEN BUDGET: Complete your entire review in under 8K output tokens. Do NOT read full source
    files. The diff contains everything you need. If a change is unclear from the diff alone,
    flag it as WARNING and move on.

    ## Tech context (do not research — use these facts)
    - SurrealDB v3: NULL != NONE. option<T> fields require NONE, reject NULL.
    - SurrealDB v3: math::max([]) returns -Infinity, not NONE.
    - SurrealDB v3: ORDER BY fields MUST appear in SELECT clause.
    - Svelte 5: Use $state(), $derived(), $effect(). SvelteSet must use $state() for reactivity.
    - Svelte 5: mount() not new App(). $props() not export let.
    - Tauri 2: snake_case Rust → camelCase JS params.
    - CSS: No Tailwind. Use .emittiv-* classes and CSS custom properties.

    ## What to check (from diff ONLY)

    **CRITICAL (blocks release)**
    - Logic errors that would cause runtime crashes or data loss
    - Hardcoded secrets, passwords, tokens in new lines
    - SurrealQL injection (string interpolation instead of parameterized queries)
    - Breaking changes to Tauri IPC command signatures

    **WARNING (logged, doesn't block)**
    - Missing error handling at boundaries
    - console.log / println! / dbg! in production paths
    - TODO/FIXME in new code
    - Potential edge cases (empty arrays, missing optional fields)

    **INFO (noted only)**
    - Style observations, minor improvements

    ## Diff to review

    Read the diff from: <DIFF_FILE>

    ## Output format (REQUIRED — pipeline parses this)

    List findings, then end with exactly one verdict line:
      CRITICAL: <file:line> <description>
      WARNING: <file:line> <description>
      INFO: <description>

    Last line MUST be one of:
      VERDICT: PASS
      VERDICT: WARN
      VERDICT: BLOCK
```

After the sub-agent returns, capture its result as `REVIEW_RESULT`. Parse the verdict:

```bash
echo "$REVIEW_RESULT"
echo ""

REVIEW_VERDICT=$(echo "$REVIEW_RESULT" | grep '^VERDICT:' | tail -1 | awk '{print $2}')

if [ "$REVIEW_VERDICT" = "BLOCK" ]; then
  echo ""
  echo "Review found CRITICAL issues. Attempting auto-fix..."
  # Continue to Step 3.5 (auto-fix)
elif [ "$REVIEW_VERDICT" = "WARN" ]; then
  echo "⚠ Review warnings noted — continuing"
elif [ "$REVIEW_VERDICT" = "PASS" ]; then
  echo "✓ Code review passed"
else
  echo "WARNING: Could not parse review verdict — continuing"
  REVIEW_VERDICT="unknown"
fi

rm -f "$DIFF_FILE"
```

---

## Step 3.5: Auto-Fix Loop (only if BLOCK)

**Only runs if Step 3 returned VERDICT: BLOCK.**

Extract CRITICAL findings from the review result. Attempt to fix each one:

1. Read only the specific lines cited in the CRITICAL finding (use offset + limit)
2. Apply a minimal fix
3. Run the test suite to verify the fix doesn't break anything
4. If tests pass → commit the fix and continue
5. If tests fail → revert the fix and STOP the pipeline

```bash
if [ "$REVIEW_VERDICT" = "BLOCK" ]; then
  echo ""
  echo "=== Auto-Fix Attempt ==="
  # The fixes are applied by this agent directly (read lines, edit, test)
  # NOT by spawning another sub-agent

  # After applying fixes, run the full test suite:
  echo "Running test suite after fixes..."
  cd /Volumes/base/dev/app/e-fees

  CARGO_RESULT=$(cd src-tauri && cargo test 2>&1 | tail -10)
  echo "$CARGO_RESULT"
  CARGO_OK=$(echo "$CARGO_RESULT" | grep -c "test result: ok")

  CHECK_RESULT=$(npm run check 2>&1 | tail -5)
  echo "$CHECK_RESULT"
  CHECK_OK=$(echo "$CHECK_RESULT" | grep -cE "found 0 errors|svelte-check found 0 errors")

  if [ "$CARGO_OK" -ge 1 ] && [ "$CHECK_OK" -ge 1 ]; then
    echo "✓ Tests pass after fix"
    git add -A
    git commit -m "fix: address review findings

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
    REVIEW_VERDICT="PASS"
  else
    echo ""
    echo "PIPELINE STOPPED: Auto-fix broke tests. Reverting."
    git checkout -- .
    echo "Fix the CRITICAL issues manually, then re-run /sendit."
    echo ""
    echo "Review findings that need manual attention:"
    echo "$REVIEW_RESULT" | grep '^CRITICAL:'
    exit 1
  fi
fi
```

---

## Step 4: Test Gate

Run the full test suite to ensure nothing is broken before bumping the version.
This runs regardless of whether review was skipped.

```bash
echo ""
echo "=== Test Gate ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would run: cargo test, npm run check"
else
  cd /Volumes/base/dev/app/e-fees

  # Rust tests
  echo "Running Rust tests..."
  CARGO_OUTPUT=$(cd src-tauri && cargo test 2>&1)
  CARGO_SUMMARY=$(echo "$CARGO_OUTPUT" | grep "test result:" | tail -1)
  echo "  $CARGO_SUMMARY"

  if ! echo "$CARGO_SUMMARY" | grep -q "ok"; then
    echo ""
    echo "PIPELINE STOPPED: Rust tests failed."
    echo "$CARGO_OUTPUT" | grep "FAILED\|panicked\|error" | head -20
    exit 1
  fi
  echo "  ✓ Rust tests passed"

  # Svelte type check
  echo "Running Svelte type check..."
  CHECK_OUTPUT=$(npm run check 2>&1)
  # Allow pre-existing errors (Proposals.svelte legacy on: syntax) — count only new errors
  ERROR_COUNT=$(echo "$CHECK_OUTPUT" | grep "svelte-check found" | grep -oE '[0-9]+ errors' | grep -oE '[0-9]+')
  echo "  svelte-check: ${ERROR_COUNT:-0} errors"

  # Known pre-existing: 8 errors from Proposals.svelte legacy event syntax
  if [ "${ERROR_COUNT:-0}" -gt 8 ]; then
    echo ""
    echo "PIPELINE STOPPED: New type errors introduced (${ERROR_COUNT} > 8 pre-existing)."
    echo "$CHECK_OUTPUT" | grep "^Error:" | head -10
    exit 1
  fi
  echo "  ✓ Type check passed (${ERROR_COUNT:-0} pre-existing errors)"

  echo "=== Test Gate OK ==="
fi
```

---

## Step 5: Version Bump

Determine bump type: `feat:` → minor, everything else → patch. `BUMP_OVERRIDE` takes precedence.

```bash
echo ""
echo "=== Version Bump ==="

if [ -n "$BUMP_OVERRIDE" ]; then
  BUMP_TYPE="$BUMP_OVERRIDE"
elif [ "$COMMIT_TYPE" = "feat" ]; then
  BUMP_TYPE="minor"
else
  BUMP_TYPE="patch"
fi

echo "Bump type: $BUMP_TYPE"

if [ "$DRY_RUN" = "true" ]; then
  CURRENT=$(node -p "require('./package.json').version")
  echo "[DRY-RUN] Would bump $CURRENT ($BUMP_TYPE)"
  NEW_VERSION="$CURRENT-dry-run"
else
  npm version "$BUMP_TYPE" --no-git-tag-version 2>&1
  node scripts/sync-version.cjs
  NEW_VERSION=$(node -p "require('./package.json').version")
  echo "New version: $NEW_VERSION"

  echo "Verifying:"
  echo "  package.json: $(node -p "require('./package.json').version")"
  grep '^version' src-tauri/Cargo.toml | head -1
  grep '"version"' src-tauri/tauri.conf.json | head -1
  grep '"title"' src-tauri/tauri.conf.json
fi
```

---

## Step 6: Commit Version Bump

```bash
echo ""
echo "=== Commit Version Bump ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git commit version bump"
else
  git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json Cargo.lock
  git commit -m "chore: bump version to $NEW_VERSION

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
  echo "✓ Version bump committed"
fi
```

---

## Step 7: Push to Forgejo and GitHub

```bash
echo ""
echo "=== Push ==="
BRANCH=$(git branch --show-current)

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git push origin $BRANCH"
  echo "[DRY-RUN] git push github $BRANCH"
else
  git push origin "$BRANCH" && echo "✓ Forgejo (origin) pushed" || { echo "FAIL: Forgejo push rejected"; exit 1; }
  git push github "$BRANCH" && echo "✓ GitHub pushed" || { echo "FAIL: GitHub push rejected"; exit 1; }
fi
```

**Stop here if `--skip-publish` was passed.** Report what was done and exit.

---

## Step 8: Create and Push Tag

```bash
echo ""
echo "=== Tag ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git tag v$NEW_VERSION (would trigger CI)"
else
  git tag "v$NEW_VERSION"
  git push origin "v$NEW_VERSION" && echo "✓ Tag pushed to Forgejo"
  git push github "v$NEW_VERSION" && echo "✓ Tag pushed to GitHub — CI triggered"
  echo ""
  echo "GitHub Actions: https://github.com/newillusions/e-fees/actions"
fi
```

---

## Step 9: CI Poll Loop — Timed Checkins

Poll GitHub Actions every 60 seconds until the build completes.
**Expected build time: 15–25 minutes.**

```bash
echo ""
echo "=== CI Build Monitoring ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] Would poll every 60s for ~20 min"
  CI_RESULT="success (dry-run)"
else
  echo "Waiting 20s for GitHub to register tag..."
  sleep 20

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
    CI_RESULT="unknown"
  else
    echo "Run ID: $RUN_ID"
    echo "https://github.com/newillusions/e-fees/actions/runs/$RUN_ID"
    echo ""

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

      echo "[$TIMESTAMP +${ELAPSED}min] #$POLL_COUNT: $STATUS"
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
        gh run view "$RUN_ID" --repo newillusions/e-fees --log-failed 2>/dev/null | tail -50
        break
      fi

      if [ "$ELAPSED" -gt 40 ]; then
        CI_RESULT="timeout"
        echo "WARNING: Build timed out after 40min."
        break
      fi

      sleep 60
    done
  fi

  if [ "$CI_RESULT" != "success" ]; then
    echo ""
    echo "PIPELINE STOPPED: CI build $CI_RESULT"
    echo "GitHub Actions: https://github.com/newillusions/e-fees/actions/runs/$RUN_ID"
    exit 1
  fi
fi
```

---

## Step 10: Sync update.json to GitHub

```bash
echo ""
echo "=== Sync update.json ==="

if [ "$DRY_RUN" = "true" ]; then
  echo "[DRY-RUN] git pull origin main && git push github main"
else
  git pull origin main && echo "✓ Pulled update.json from Forgejo"

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

  sleep 30
  ENDPOINT_VERSION=$(curl -sf "https://raw.githubusercontent.com/newillusions/e-fees/main/update.json" \
    | python3 -c "import json,sys; print(json.load(sys.stdin).get('version','error'))" 2>/dev/null || echo "not yet available")
  echo "Endpoint version: $ENDPOINT_VERSION"
fi
```

---

## Step 11: Verify Forgejo Release

```bash
echo ""
echo "=== Verify Forgejo Release ==="

if [ "$DRY_RUN" = "true" ]; then
  RELEASE_STATUS="dry-run"
else
  RELEASE_JSON=$(curl -sf \
    "https://forge.mms.name/api/v1/repos/emittiv/fee-prop/releases/tags/v$NEW_VERSION" 2>/dev/null)

  if [ -z "$RELEASE_JSON" ]; then
    echo "WARNING: Release not found yet. Check:"
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

## Step 12: Save KB Observation

```bash
if [ "$DRY_RUN" != "true" ] && [ "$CI_RESULT" = "success" ]; then
  source ~/.kb-agent.env 2>/dev/null || true
  if [ -n "$SURREALDB_URL" ] && [ -n "$SURREALDB_PASS" ]; then
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    curl -s -X POST "${SURREALDB_URL}/sql" \
      -u "martin:${SURREALDB_PASS}" \
      -H "surreal-ns: kb" -H "surreal-db: knowledge" \
      -H "Accept: application/json" -H "Content-Type: text/plain" \
      -d "INSERT INTO observation (entity_name, content, confidence, status, created_at) VALUES (
        'E-Fees Release',
        'Released e-fees v${NEW_VERSION} via /sendit. CI: ${CI_RESULT}. Review: ${REVIEW_VERDICT}.',
        0.9, 'active', '${TIMESTAMP}'
      );" > /dev/null 2>&1 && echo "✓ KB observation saved" || echo "(KB save skipped)"
  fi
fi
```

---

## Step 13: Final Report

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
echo "  ✓ Review: ${REVIEW_VERDICT:-skipped}"
echo "  ✓ Tests: cargo test + npm run check passed"
echo "  ✓ Version: ${CURRENT_VERSION} → ${NEW_VERSION}"
echo "  ✓ Pushed: Forgejo + GitHub"

if [ "$SKIP_PUBLISH" = "true" ]; then
  echo "  - Tag/CI: skipped (--skip-publish)"
else
  echo "  ✓ Tag: v${NEW_VERSION}"
  echo "  ✓ CI Build: $CI_RESULT"
  echo "  ✓ Update endpoint: https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"
  echo "  ✓ Release assets: $RELEASE_STATUS"
fi
echo ""
echo "Links:"
echo "  Forgejo: https://forge.mms.name/emittiv/fee-prop/releases/tag/v${NEW_VERSION}"
echo "  Actions: https://github.com/newillusions/e-fees/actions"
echo "  Update:  https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"
echo ""
if [ "$DRY_RUN" = "true" ]; then
  echo "  ↑ DRY-RUN mode — no changes were made."
fi
echo "================================================"
```

---

## Args

(Populated by SKILL.md when spawning this agent — args follow below)

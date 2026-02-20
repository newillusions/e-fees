---
name: release
description: Complete E-Fees release workflow - version bump, tag, build, publish, update manifest. Use /release [patch|minor|major] or /release (defaults to patch).
---

# E-Fees Release Workflow

Complete release pipeline: version bump → tag → CI build → Forgejo release → update manifest sync.

**Run this entire workflow to completion without stopping to ask.**

## Arguments

- `/release` — Patch bump (0.12.2 → 0.12.3)
- `/release minor` — Minor bump (0.12.2 → 0.13.0)
- `/release major` — Major bump (0.12.2 → 1.0.0)
- `/release 0.15.0` — Set exact version

## Architecture

```
Tag push → GitHub Actions → Builds macOS (aarch64 + x64) + Windows
                          → Uploads artifacts to Forgejo releases
                          → Generates update.json → pushes to Forgejo via API
                          → Creates commit on Forgejo with update.json

Post-CI:  Pull update.json from Forgejo → push to GitHub
          (Tauri updater checks raw.githubusercontent.com)
```

## Prerequisites Check

Before starting, verify:

```bash
# Must be on main branch with clean working tree
BRANCH=$(git branch --show-current)
STATUS=$(git status --porcelain)
if [ "$BRANCH" != "main" ]; then echo "ERROR: Not on main branch (on $BRANCH)"; exit 1; fi
if [ -n "$STATUS" ]; then echo "ERROR: Uncommitted changes"; git status --short; exit 1; fi
echo "OK: Clean main branch"
```

Run Rust tests to confirm nothing is broken:
```bash
cd src-tauri && cargo test 2>&1 | tail -5
```

## Step 1: Version Bump

Determine the bump type from arguments (default: patch).

```bash
# Use the sync-version script — updates package.json, Cargo.toml, tauri.conf.json (including window title)
# For exact version:
node scripts/sync-version.cjs <VERSION>

# Or for npm-style bump, first bump package.json then sync:
npm version <patch|minor|major> --no-git-tag-version
node scripts/sync-version.cjs
```

Verify the version was set correctly in all 3 files:
```bash
VERSION=$(node -p "require('./package.json').version")
echo "package.json: $VERSION"
grep '^version' src-tauri/Cargo.toml
grep '"version"' src-tauri/tauri.conf.json | head -1
grep '"title"' src-tauri/tauri.conf.json
```

Commit the version bump:
```bash
VERSION=$(node -p "require('./package.json').version")
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to $VERSION"
```

## Step 2: Push and Tag

Push to both remotes and create the tag that triggers CI:

```bash
VERSION=$(node -p "require('./package.json').version")

# Push code first
git push origin main
git push github main

# Create and push tag (triggers GitHub Actions)
git tag "v$VERSION"
git push origin "v$VERSION"
git push github "v$VERSION"

echo "Tag v$VERSION pushed. GitHub Actions build triggered."
```

## Step 3: Monitor Build

Wait for the build to start, then poll until completion:

```bash
# Wait for GitHub to pick up the tag
sleep 15

# Check build status
gh run list --repo newillusions/e-fees --limit 1 --json status,conclusion,databaseId,headBranch
```

Poll every 60 seconds until `status` is `completed`:
```bash
RUN_ID=$(gh run list --repo newillusions/e-fees --limit 1 --json databaseId --jq '.[0].databaseId')

# Check individual jobs
gh run view $RUN_ID --repo newillusions/e-fees --json jobs --jq '.jobs[] | "\(.name): \(.status) \(.conclusion // "")"'
```

**If a job fails**, check logs:
```bash
gh run view $RUN_ID --repo newillusions/e-fees --log-failed | tail -80
```

**Typical build time**: 15-25 minutes for all 3 jobs (macOS aarch64, macOS x64, Windows).

## Step 4: Sync update.json to GitHub

After CI succeeds, the update-manifest job pushes update.json to Forgejo via API (creates a commit).
The Tauri updater endpoint is `raw.githubusercontent.com`, so we need update.json on GitHub too.

```bash
# Pull the CI-generated update.json commit from Forgejo
git pull origin main

# Verify update.json exists and has correct content
cat update.json | python3 -m json.tool

# Push to GitHub so raw.githubusercontent.com serves it
git push github main
```

Verify the update endpoint is accessible:
```bash
VERSION=$(node -p "require('./package.json').version")
curl -s "https://raw.githubusercontent.com/newillusions/e-fees/main/update.json" | python3 -m json.tool
```

Check that the version matches and all 3 platform URLs are correct (should point to `forge.mms.name`).

## Step 5: Verify Forgejo Release

```bash
VERSION=$(node -p "require('./package.json').version")

# Check release exists on Forgejo
curl -s "https://forge.mms.name/api/v1/repos/emittiv/fee-prop/releases/tags/v$VERSION" | python3 -c "
import sys, json
r = json.load(sys.stdin)
print(f'Release: {r[\"name\"]}')
print(f'Tag: {r[\"tag_name\"]}')
print(f'Assets: {len(r.get(\"assets\", []))}')
for a in r.get('assets', []):
    print(f'  - {a[\"name\"]} ({a[\"size\"]/1024/1024:.1f} MB)')
"
```

Expected assets (6-7 files):
- `E-Fees_<VERSION>_aarch64.dmg` + checksum
- `E-Fees_<VERSION>_aarch64.app.tar.gz` (updater)
- `E-Fees_<VERSION>_x64.dmg` + checksum
- `E-Fees_<VERSION>_x64.app.tar.gz` (updater)
- `E-Fees_<VERSION>_x64.msi`
- `E-Fees_<VERSION>_x64-setup.exe` (updater)

## Step 6: Update Window Title (if not done by sync script)

The sync-version script handles this, but verify:
```bash
grep '"title"' src-tauri/tauri.conf.json
# Should show: "title": "E-Fees v<VERSION>"
```

## Step 7: Final Summary

```bash
VERSION=$(node -p "require('./package.json').version")
echo ""
echo "=== Release v$VERSION Complete ==="
echo ""
echo "Forgejo release: https://forge.mms.name/emittiv/fee-prop/releases/tag/v$VERSION"
echo "Update endpoint: https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"
echo "GitHub Actions:  https://github.com/newillusions/e-fees/actions"
echo ""
echo "Existing installs will auto-update via Tauri updater."
echo "Direct downloads available from Forgejo release page."
```

## Troubleshooting

### Build fails with compile error
Run locally first to verify:
```bash
npm run check && cd src-tauri && cargo check
```

### Build fails with export error
Both `src/lib/api.ts` (legacy) AND `src/lib/api/index.ts` (modular) need to export new functions.

### Git push rejected (Forgejo has new commits from CI)
```bash
git pull origin main --rebase
git push origin main
git push github main
```

### Need to re-run failed build
```bash
VERSION=$(node -p "require('./package.json').version")

# Delete old tag from both remotes
git tag -d "v$VERSION"
git push origin ":refs/tags/v$VERSION"
git push github ":refs/tags/v$VERSION"

# Fix the issue, commit, then recreate tag
git tag "v$VERSION"
git push origin "v$VERSION"
git push github "v$VERSION"
```

### update.json not appearing on GitHub
CI pushes to Forgejo only. You must manually pull and push to GitHub:
```bash
git pull origin main && git push github main
```

### Updater shows wrong version
Check `raw.githubusercontent.com` (may be cached for ~5 min):
```bash
curl -s "https://raw.githubusercontent.com/newillusions/e-fees/main/update.json" | python3 -c "import sys,json; print(json.load(sys.stdin)['version'])"
```

## Key Paths

| Item | Location |
|------|----------|
| Version sync script | `scripts/sync-version.cjs` |
| CI workflow | `.github/workflows/build-releases.yml` |
| Forgejo releases | `forge.mms.name/emittiv/fee-prop/releases` |
| Update endpoint | `raw.githubusercontent.com/newillusions/e-fees/main/update.json` |
| Updater config | `src-tauri/tauri.conf.json` → `plugins.updater` |
| GitHub repo | `newillusions/e-fees` |
| Forgejo repo | `forge.mms.name/emittiv/fee-prop` |
| Signing keys | GitHub Secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |
| Forgejo token | GitHub Secret: `GITEA_TOKEN` |

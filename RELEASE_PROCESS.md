> **HISTORICAL - superseded by CLAUDE.md and the `/release` skill (tombstoned 2026-07-03 per architect review, charter-e-fees).**
> **Content below is STALE and must not be followed.** This document describes the manual release process from before the `/release [patch|minor|major]` background-agent pipeline existed (2025-12-03 era: dual `git push origin`/`git push github`, a local `scripts/publish-release.sh` copying to an Apache mount). The current pipeline is entirely different: tag push -> GitHub Actions builds -> artifacts uploaded to Forgejo releases -> `update.json` generated and pushed to Forgejo via API -> synced to GitHub. See `CLAUDE.md` §Release and `.claude/commands/release.md` for the real process.

# E-Fees Release Process

**Last Updated**: 2025-12-03

## Overview

The E-Fees application uses a hybrid build and release process:
- **GitHub Actions**: Handles builds (especially Windows which requires Windows runners)
- **Forgejo**: Primary code repository (private, forge.mms.name)
- **Apache Web Server**: Hosts release binaries for auto-updates
- **GitHub**: Mirror for CI/CD and update manifest hosting

## Prerequisites

### Required Tools
- `gh` CLI (GitHub CLI)
- Git with dual remotes configured
- Access to `/Volumes/user/www/e-fees-releases` (web server mount)

### Environment Configuration
```bash
# Verify GitHub CLI is authenticated
gh auth status

# Verify git remotes
git remote -v
# Should show:
# origin  ssh://git@forge.mms.name:2222/emittiv/fee-prop.git (Forgejo)
# github  git@github.com:newillusions/e-fees.git (GitHub)
```

## Step-by-Step Release Process

### 1. Version Bump and Code Changes

```bash
# Update version in all necessary files
npm run version:set X.Y.Z

# Update version comment in src-tauri/src/lib.rs
# Example:
# // Auto-update test - v0.10.17 (with enhanced logging)
```

### 2. Commit and Tag

```bash
# Stage changes
git add -A

# Commit with conventional commit format
git commit -m "feat: Description of changes

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Create annotated tag
git tag -a v0.10.17 -m "Release v0.10.17"
```

### 3. Push to Trigger Build

```bash
# Push to both remotes
git push origin main
git push origin v0.10.17
git push github main
git push github v0.10.17
```

**Note**: GitHub Actions will automatically trigger on the tag push and build for all platforms.

### 4. Wait for Build Completion

Monitor the build progress:
```bash
# Check recent workflow runs
gh run list --repo newillusions/e-fees --limit 5

# Watch specific run (if needed)
gh run watch <RUN_ID> --repo newillusions/e-fees
```

Wait for **"success"** status before proceeding.

### 5. Publish Release to Web Server

```bash
# Run the automated publish script
cd /Volumes/base/dev/e-fees
./scripts/publish-release.sh 0.10.17
```

The script will:
1. Find the successful GitHub Actions run for the version
2. Download all artifacts (macOS arm64, macOS x64, Windows)
3. Extract and organize files
4. Copy binaries to `/Volumes/user/www/e-fees-releases/0.10.17/`
5. Read and base64-encode signatures
6. Generate `/Volumes/user/www/e-fees-releases/update.json`

**Expected Output**:
```
📦 Publishing E-Fees v0.10.17
================================
🔍 Finding GitHub Actions run for v0.10.17...
✅ Found run ID: 12345678
📂 Downloading artifacts...
⬇️  Downloading macOS arm64...
⬇️  Downloading macOS x64...
⬇️  Downloading Windows...
📁 Creating release directory...
📋 Copying files to web server...
  ✓ macOS ARM64 binary copied
  ✓ macOS x64 binary copied
  ✓ Windows binary copied
🔐 Reading signatures...
📝 Generating update.json...

✅ Successfully published v0.10.17!

📍 Files available at:
   /Volumes/user/www/e-fees-releases/0.10.17/

🌐 Update manifest:
   /Volumes/user/www/e-fees-releases/update.json
```

### 6. Commit update.json to Git

```bash
# Copy the generated update.json to repo
cp /Volumes/user/www/e-fees-releases/update.json ./update.json

# Commit and push
git add update.json
git commit -m "chore: Update manifest for v0.10.17

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

git push origin main
git push github main
```

**Important**: The update.json file MUST be on GitHub's main branch because Tauri updater checks:
`https://raw.githubusercontent.com/newillusions/e-fees/main/update.json`

### 7. Verify Release

```bash
# Check update.json is accessible
curl -s https://raw.githubusercontent.com/newillusions/e-fees/main/update.json | jq

# Check binary is accessible
curl -I https://apache.mms.name/e-fees-releases/0.10.17/E-Fees_aarch64.app.tar.gz

# Verify version in update.json matches
curl -s https://raw.githubusercontent.com/newillusions/e-fees/main/update.json | jq -r '.version'
```

Expected responses:
- update.json returns valid JSON with version "0.10.17"
- Binary URL returns `200 OK` with `Content-Type: application/gzip`

### 8. Test Update (Manual)

1. Ensure older version is installed (e.g., v0.10.16)
2. Launch the application
3. Wait for update check (happens on startup)
4. Update dialog should appear
5. Click "Install" and verify download/installation

## Troubleshooting

### Build Fails on GitHub Actions
```bash
# View detailed logs
gh run view <RUN_ID> --repo newillusions/e-fees --log
```

Common causes:
- Rust compilation errors
- Missing dependencies
- Tauri configuration issues

### Script Can't Find Artifacts
```bash
# List recent runs manually
gh run list --repo newillusions/e-fees --limit 10 --json databaseId,displayTitle,conclusion,status

# Verify run succeeded
gh run view <RUN_ID> --repo newillusions/e-fees
```

Ensure:
- Run status is "completed"
- Run conclusion is "success"
- Version tag is in displayTitle

### Files Not Accessible on Web Server
```bash
# Check file permissions
ls -lh /Volumes/user/www/e-fees-releases/0.10.17/

# Test local access
curl -I https://apache.mms.name/e-fees-releases/0.10.17/E-Fees_aarch64.app.tar.gz
```

Ensure:
- Files are world-readable (644 permissions)
- Apache server is running
- Network path is mounted

### update.json Not Updating
```bash
# Verify file is on GitHub main branch
gh api repos/newillusions/e-fees/contents/update.json --jq '.sha'

# Force refresh of raw.githubusercontent.com cache (wait 5 minutes or use cache busting)
curl -H "Cache-Control: no-cache" https://raw.githubusercontent.com/newillusions/e-fees/main/update.json
```

### Signature Verification Fails

**Root Cause**: Minisign signatures embed the filename

**Solution**: Do NOT rename files after signing. The publish script preserves original filenames.

## File Structure

```
/Volumes/user/www/e-fees-releases/
├── update.json                    # Current version manifest
├── 0.10.15/
│   ├── E-Fees_aarch64.app.tar.gz
│   ├── E-Fees_aarch64.app.tar.gz.sig
│   ├── E-Fees_x64.app.tar.gz
│   └── E-Fees_x64.app.tar.gz.sig
├── 0.10.16/
│   └── ...
└── 0.10.17/
    └── ...
```

## Artifacts Produced by GitHub Actions

Each build creates these artifacts:

1. **macos-aarch64**
   - `macos/E-Fees.app.tar.gz`
   - `macos/E-Fees.app.tar.gz.sig`

2. **macos-x64**
   - `macos/E-Fees.app.tar.gz`
   - `macos/E-Fees.app.tar.gz.sig`

3. **windows**
   - `E-Fees_*_x64-setup.nsis.zip`
   - `E-Fees_*_x64-setup.nsis.zip.sig`

4. **update-manifest**
   - `update.json` (generated by CI, but NOT used - we regenerate locally)

## Security Notes

- Private key for signing is stored in GitHub Secrets
- Web server is behind nginx reverse proxy and OPNsense firewall
- SSL certificate from Let's Encrypt (auto-renewed)
- Binaries are publicly accessible but repository remains private
- No authentication required for downloads (by design for updater)

## Quick Reference

```bash
# Full release workflow (copy-paste friendly)
VERSION="0.10.17"

# 1. Version bump
npm run version:set $VERSION

# 2. Commit and tag
git add -A
git commit -m "feat: Release v$VERSION"
git tag -a v$VERSION -m "Release v$VERSION"

# 3. Push
git push origin main && git push origin v$VERSION
git push github main && git push github v$VERSION

# 4. Wait for build, then publish
./scripts/publish-release.sh $VERSION

# 5. Commit update.json
cp /Volumes/user/www/e-fees-releases/update.json ./update.json
git add update.json
git commit -m "chore: Update manifest for v$VERSION"
git push origin main && git push github main

# 6. Verify
curl -s https://raw.githubusercontent.com/newillusions/e-fees/main/update.json | jq -r '.version'
```

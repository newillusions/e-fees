# Forgejo Releases Setup Guide

## Overview

This guide explains how to create releases on your Forgejo server and set up automated builds using Forgejo Actions (Gitea-compatible).

## Current Status

- ✅ Git tag created: `v0.10.0`
- ✅ Code pushed to Forgejo
- ⚠️ Forgejo release not created yet (releases tab empty)
- ⏳ Forgejo Actions workflow configured (needs testing)

## Why Releases Don't Show Automatically

**Important**: Git tags and Forgejo releases are different:
- **Git tag**: A marker in git history (already created)
- **Forgejo release**: A UI feature that shows in the releases tab with downloadable binaries

You need to **explicitly create a Forgejo release** using one of the methods below.

## Method 1: Manual Release Creation (Quick Start)

### Via Web Interface

1. Go to: `https://forge.mms.name/emittiv/fee-prop/releases`
2. Click "New Release"
3. Fill in the form:
   - **Tag**: `v0.10.0` (select from existing tags)
   - **Release Title**: `E-Fees v0.10.0`
   - **Description**: Copy from `releases/v0.10.0/RELEASE_NOTES.md`
   - **Attachments**: Upload the DMG file
4. Click "Publish Release"

### Via Forgejo API

```bash
# Create a Forgejo access token first:
# https://forge.mms.name/user/settings/applications

export GITEA_TOKEN='your_token_here'

# Run the automated script
./scripts/create-gitea-release.sh 0.10.0
```

The script will:
- Create the release on Forgejo
- Upload the DMG file
- Upload the SHA256 checksum
- Use release notes from the repository

## Method 2: Forgejo Actions (Automated)

Forgejo Actions is a CI/CD system compatible with GitHub Actions syntax (Gitea API compatible).

### Prerequisites

1. **Check Forgejo Version**:
   ```bash
   curl https://forge.mms.name/api/v1/version
   ```
   Forgejo has Actions support built-in.

2. **Enable Forgejo Actions** (Forgejo admin):
   ```ini
   # app.ini
   [actions]
   ENABLED = true
   ```

3. **Register Forgejo Actions Runner**:
   ```bash
   # Download act_runner (Forgejo uses same runner as Gitea)
   wget https://dl.gitea.com/act_runner/latest/act_runner-latest-linux-amd64

   # Register runner
   ./act_runner register --instance https://forge.mms.name \
     --token YOUR_RUNNER_TOKEN

   # Run runner
   ./act_runner daemon
   ```

### Workflow Location

Forgejo Actions workflows are stored in:
```
.gitea/workflows/release-build.yml
```

**Already created**: `.gitea/workflows/release-build.yml`

### Create Forgejo Token for Workflows

1. Go to: `https://forge.mms.name/user/settings/applications`
2. Generate New Token
3. Name: `Forgejo Actions Release`
4. Scopes: `write:repository`, `write:package`
5. Copy the token

6. Add token as repository secret:
   - Go to: `https://forge.mms.name/emittiv/fee-prop/settings/secrets`
   - Add new secret:
     - Name: `GITEA_TOKEN`
     - Value: (paste your token)

### Trigger Workflow

The workflow triggers automatically on:
- Tag push: `git push origin v0.11.0`
- Manual trigger via Gitea UI

Or manually:
1. Go to: `https://forge.mms.name/emittiv/fee-prop/actions`
2. Select "Release Build (Multi-Platform)"
3. Click "Run workflow"
4. Enter version: `0.10.0`

### What the Workflow Does

1. **Creates Forgejo Release** with release notes
2. **Builds for all platforms** in parallel:
   - macOS (Apple Silicon)
   - macOS (Intel)
   - Windows (x64) - MSI and NSIS installers
   - Linux (AppImage and DEB)
3. **Uploads all binaries** to the release
4. **Generates checksums**

### Expected Build Time

- macOS builds: ~10-15 minutes each
- Windows build: ~15-20 minutes
- Linux build: ~10-15 minutes
- **Total**: ~30-40 minutes for all platforms

## Method 3: Use GitHub as Mirror (Alternative)

If Gitea Actions is not available or not working:

1. **Mirror to GitHub**:
   ```bash
   git remote add github https://github.com/yourusername/e-fees.git
   git push github main --tags
   ```

2. **Use GitHub Actions**:
   - GitHub Actions will trigger automatically
   - Builds complete for all platforms
   - Download artifacts from GitHub

3. **Manually copy to Forgejo**:
   - Download releases from GitHub
   - Upload to Forgejo manually or via script

## Comparison of Methods

| Method | Automation | Platforms | Effort | Best For |
|--------|-----------|-----------|--------|----------|
| Manual Web UI | None | macOS only | Low | One-time releases |
| API Script | Semi-auto | macOS only | Medium | Current releases |
| Forgejo Actions | Full | All platforms | High (setup) | Future releases |
| GitHub Mirror | Full | All platforms | Low | Quick solution |

## Recommended Approach

**For v0.10.0 (Current Release)**:
1. Use the API script to create the release immediately:
   ```bash
   export GITEA_TOKEN='your_token'
   ./scripts/create-gitea-release.sh 0.10.0
   ```

**For Future Releases**:
1. Set up Forgejo Actions (Actions support built-in)
2. OR use GitHub as a mirror for automated builds
3. All future releases will be automatic

## Troubleshooting

### Forgejo Actions Not Available

**Symptom**: No "Actions" tab in repository settings

**Solution**: Either:
- Check Forgejo admin settings
- Use GitHub mirror method
- Use manual release creation

### Workflow Not Triggering

**Check**:
1. Is Forgejo Actions enabled? Check admin settings
2. Is runner registered and running?
3. Is workflow file in correct location? (`.gitea/workflows/`)
4. Are secrets configured correctly?

### Authentication Errors

**Solution**:
1. Regenerate Forgejo token
2. Ensure token has correct scopes
3. Check token is added to repository secrets

## Next Steps

1. **Immediate**: Create v0.10.0 release using API script
2. **This Week**: Verify Forgejo Actions availability
3. **Next Release**: Use automated workflow

## Resources

- [Forgejo Actions Documentation](https://forgejo.org/docs/latest/user/actions/)
- [Forgejo API Documentation](https://forgejo.org/docs/latest/api/) (Gitea-compatible)
- [Act Runner Documentation](https://gitea.com/gitea/act_runner)

## Status Checklist

- [ ] v0.10.0 release created on Forgejo
- [ ] DMG file uploaded to release
- [ ] Forgejo Actions enabled (if available)
- [ ] Runner registered (if using Actions)
- [ ] GITEA_TOKEN secret configured
- [ ] Workflow tested for next release

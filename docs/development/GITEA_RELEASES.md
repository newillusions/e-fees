# Gitea Releases Setup Guide

## Overview

This guide explains how to create releases on your Gitea server and set up automated builds using Gitea Actions.

## Current Status

- ✅ Git tag created: `v0.10.0`
- ✅ Code pushed to Gitea
- ⚠️ Gitea release not created yet (releases tab empty)
- ⏳ Gitea Actions workflow configured (needs testing)

## Why Releases Don't Show Automatically

**Important**: Git tags and Gitea releases are different:
- **Git tag**: A marker in git history (already created)
- **Gitea release**: A UI feature that shows in the releases tab with downloadable binaries

You need to **explicitly create a Gitea release** using one of the methods below.

## Method 1: Manual Release Creation (Quick Start)

### Via Web Interface

1. Go to: `https://git.mms.name/martin/fee-prop/releases`
2. Click "New Release"
3. Fill in the form:
   - **Tag**: `v0.10.0` (select from existing tags)
   - **Release Title**: `E-Fees v0.10.0`
   - **Description**: Copy from `releases/v0.10.0/RELEASE_NOTES.md`
   - **Attachments**: Upload the DMG file
4. Click "Publish Release"

### Via Gitea API

```bash
# Create a Gitea access token first:
# https://git.mms.name/user/settings/applications

export GITEA_TOKEN='your_token_here'

# Run the automated script
./scripts/create-gitea-release.sh 0.10.0
```

The script will:
- Create the release on Gitea
- Upload the DMG file
- Upload the SHA256 checksum
- Use release notes from the repository

## Method 2: Gitea Actions (Automated)

Gitea Actions is a CI/CD system compatible with GitHub Actions syntax (available in Gitea 1.19+).

### Prerequisites

1. **Check Gitea Version**:
   ```bash
   curl https://git.mms.name/api/v1/version
   ```
   Requires Gitea 1.19 or later for Actions support.

2. **Enable Gitea Actions** (Gitea admin):
   ```ini
   # app.ini
   [actions]
   ENABLED = true
   ```

3. **Register Gitea Actions Runner**:
   ```bash
   # Download act_runner
   wget https://dl.gitea.com/act_runner/latest/act_runner-latest-linux-amd64

   # Register runner
   ./act_runner register --instance https://git.mms.name \
     --token YOUR_RUNNER_TOKEN

   # Run runner
   ./act_runner daemon
   ```

### Workflow Location

Gitea Actions workflows are stored in:
```
.gitea/workflows/release-build.yml
```

**Already created**: `.gitea/workflows/release-build.yml`

### Create Gitea Token for Workflows

1. Go to: `https://git.mms.name/user/settings/applications`
2. Generate New Token
3. Name: `Gitea Actions Release`
4. Scopes: `write:repository`, `write:package`
5. Copy the token

6. Add token as repository secret:
   - Go to: `https://git.mms.name/martin/fee-prop/settings/secrets`
   - Add new secret:
     - Name: `GITEA_TOKEN`
     - Value: (paste your token)

### Trigger Workflow

The workflow triggers automatically on:
- Tag push: `git push origin v0.11.0`
- Manual trigger via Gitea UI

Or manually:
1. Go to: `https://git.mms.name/martin/fee-prop/actions`
2. Select "Release Build (Multi-Platform)"
3. Click "Run workflow"
4. Enter version: `0.10.0`

### What the Workflow Does

1. **Creates Gitea Release** with release notes
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

3. **Manually copy to Gitea**:
   - Download releases from GitHub
   - Upload to Gitea manually or via script

## Comparison of Methods

| Method | Automation | Platforms | Effort | Best For |
|--------|-----------|-----------|--------|----------|
| Manual Web UI | None | macOS only | Low | One-time releases |
| API Script | Semi-auto | macOS only | Medium | Current releases |
| Gitea Actions | Full | All platforms | High (setup) | Future releases |
| GitHub Mirror | Full | All platforms | Low | Quick solution |

## Recommended Approach

**For v0.10.0 (Current Release)**:
1. Use the API script to create the release immediately:
   ```bash
   export GITEA_TOKEN='your_token'
   ./scripts/create-gitea-release.sh 0.10.0
   ```

**For Future Releases**:
1. Set up Gitea Actions (if Gitea version >= 1.19)
2. OR use GitHub as a mirror for automated builds
3. All future releases will be automatic

## Troubleshooting

### Gitea Actions Not Available

**Symptom**: No "Actions" tab in repository settings

**Solution**: Either:
- Upgrade Gitea to 1.19+
- Use GitHub mirror method
- Use manual release creation

### Workflow Not Triggering

**Check**:
1. Is Gitea Actions enabled? Check admin settings
2. Is runner registered and running?
3. Is workflow file in correct location? (`.gitea/workflows/`)
4. Are secrets configured correctly?

### Authentication Errors

**Solution**:
1. Regenerate Gitea token
2. Ensure token has correct scopes
3. Check token is added to repository secrets

## Next Steps

1. **Immediate**: Create v0.10.0 release using API script
2. **This Week**: Verify Gitea version and Actions availability
3. **Next Release**: Use automated workflow

## Resources

- [Gitea Actions Documentation](https://docs.gitea.com/next/usage/actions/overview)
- [Gitea API Documentation](https://docs.gitea.com/next/development/api-usage)
- [Act Runner Documentation](https://gitea.com/gitea/act_runner)

## Status Checklist

- [ ] v0.10.0 release created on Gitea
- [ ] DMG file uploaded to release
- [ ] Gitea version checked (>= 1.19 for Actions)
- [ ] Gitea Actions enabled (if available)
- [ ] Runner registered (if using Actions)
- [ ] GITEA_TOKEN secret configured
- [ ] Workflow tested for next release

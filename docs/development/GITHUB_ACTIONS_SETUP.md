# GitHub Actions for macOS and Windows Builds

## Overview

Since Tauri cannot cross-compile desktop apps, we use **GitHub Actions** (free runners) to build macOS and Windows binaries, then upload them to your **Gitea releases**.

**Why GitHub Actions?**
- ✅ Free macOS runners (both Intel and Apple Silicon)
- ✅ Free Windows runners
- ✅ Automated builds on tag push
- ✅ Directly uploads to Gitea releases
- ✅ Code stays on Gitea (GitHub is just CI/CD)

## Setup Steps

### 1. Create GitHub Repository (Mirror)

```bash
# On GitHub.com, create a new repository: e-fees
# Make it private if you want

# Add GitHub as remote
git remote add github https://github.com/YOUR_USERNAME/e-fees.git

# Push code
git push github main
```

### 2. Add Gitea Token to GitHub Secrets

The workflow needs your Gitea token to upload builds.

**On GitHub:**
1. Go to: `https://github.com/YOUR_USERNAME/e-fees/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `GITEA_TOKEN`
4. Value: `ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f`
5. Click "Add secret"

### 3. Push the Workflow

The workflow file is already created at `.github/workflows/build-releases.yml`.

```bash
# Make sure it's committed
git add .github/workflows/build-releases.yml
git commit -m "Add GitHub Actions workflow for macOS and Windows builds"

# Push to GitHub
git push github main
```

### 4. Test the Workflow

**Option A: Tag-based (Production)**
```bash
# Create and push tag
git tag v0.11.0
git push github v0.11.0
```

**Option B: Manual Trigger (Testing)**
1. Go to: `https://github.com/YOUR_USERNAME/e-fees/actions`
2. Select "Build macOS and Windows Releases"
3. Click "Run workflow"
4. Enter version: `0.11.0`
5. Click "Run"

### 5. Monitor Build

1. Go to: `https://github.com/YOUR_USERNAME/e-fees/actions`
2. Click on the running workflow
3. Watch the build progress

**Build time:**
- macOS (2 jobs): ~15-20 minutes each
- Windows: ~10-15 minutes
- **Total**: ~30-40 minutes

### 6. Verify on Gitea

Once complete, check your Gitea release:
```
https://git.mms.name/martin/fee-prop/releases
```

You should see:
- ✅ macOS Apple Silicon DMG
- ✅ macOS Intel DMG
- ✅ Windows MSI installer
- ✅ Windows NSIS setup.exe
- ✅ SHA256 checksums

## Workflow Explained

### What it Does

```yaml
build-macos:
  - Builds for aarch64-apple-darwin (Apple Silicon)
  - Builds for x86_64-apple-darwin (Intel)
  - Creates DMG installers
  - Uploads to Gitea release

build-windows:
  - Builds for x64
  - Creates MSI installer
  - Creates NSIS installer
  - Uploads to Gitea release
```

### How it Uploads to Gitea

1. **Checks if release exists** on Gitea for the tag
2. **Creates release** if it doesn't exist
3. **Uploads binaries** using Gitea API
4. **Adds checksums**

All artifacts go directly to your Gitea server!

## Recommended Workflow

### For Each Release:

**Step 1: Update Version**
```bash
npm run version:set 0.11.0
git add -A
git commit -m "chore: Bump version to 0.11.0"
```

**Step 2: Push to Both Remotes**
```bash
git push origin main    # Gitea (source of truth)
git push github main    # GitHub (CI/CD)
```

**Step 3: Create Tag**
```bash
git tag v0.11.0
git push origin v0.11.0   # Gitea
git push github v0.11.0   # GitHub (triggers build)
```

**Step 4: Wait for Builds**
- Monitor: `https://github.com/YOUR_USERNAME/e-fees/actions`
- Wait ~30-40 minutes

**Step 5: Verify Release**
- Check: `https://git.mms.name/martin/fee-prop/releases`
- Download and test installers

## Dual Remote Management

### View Remotes
```bash
git remote -v
```

Expected output:
```
origin  https://git.mms.name/martin/fee-prop.git (fetch)
origin  https://git.mms.name/martin/fee-prop.git (push)
github  https://github.com/YOUR_USERNAME/e-fees.git (fetch)
github  https://github.com/YOUR_USERNAME/e-fees.git (push)
```

### Push to Both
```bash
# Push to Gitea
git push origin main

# Push to GitHub
git push github main

# Or push to both at once
git push origin main && git push github main
```

### Sync Tags
```bash
# Push tags to both
git push origin --tags
git push github --tags
```

## Cost Analysis

| Service | Purpose | Cost |
|---------|---------|------|
| **Gitea** | Source code, releases | Free (self-hosted) |
| **GitHub** | macOS/Windows CI/CD | **Free** (2000 min/month) |

**Monthly Usage Estimate:**
- 4 releases/month × 40 minutes = 160 minutes
- Well within free tier (2000 min)

## Troubleshooting

### Build Fails on GitHub

**Check:**
1. Is `GITEA_TOKEN` secret added?
2. Can GitHub reach `https://git.mms.name`? (must be public)
3. Check workflow logs for specific errors

### Builds Don't Upload to Gitea

**Symptoms:** Build succeeds but no files on Gitea

**Check:**
1. Is Gitea server publicly accessible from internet?
2. Is token correct and has write permissions?
3. Check GitHub Actions logs for upload errors

**Solution if Gitea is private:**
Download artifacts from GitHub and upload manually:
```bash
# Download from GitHub Actions page
# Then upload using script
./scripts/create-gitea-release.sh 0.11.0
```

### Want to Stop Using GitHub

Just remove the remote:
```bash
git remote remove github
```

You can always re-add it later if needed.

## Alternative: Manual Builds

If you don't want to use GitHub:

**macOS (your current approach):**
```bash
npm run version:set 0.11.0
npm run tauri:build
./scripts/create-gitea-release.sh 0.11.0
```

**Windows:**
- Need Windows PC or VM
- Same commands as above
- Upload using script or `/gitea-release` command

## Benefits of This Approach

✅ **Code stays on Gitea** (source of truth)
✅ **GitHub only for builds** (free runners)
✅ **Fully automated** (tag → build → upload)
✅ **All platforms** (macOS + Windows)
✅ **Direct to Gitea** (no manual downloads)
✅ **Backup artifacts** (available on GitHub too)

## Summary

**Setup time:** 10 minutes
**Build time:** 30-40 minutes per release
**Cost:** $0 (free GitHub tier)
**Maintenance:** None (fully automated)

This is the recommended approach for cross-platform desktop app releases when you don't have dedicated build machines.

---

**Last Updated:** November 17, 2025
**Workflow:** `.github/workflows/build-releases.yml`
**Platforms:** macOS (ARM + Intel) + Windows (MSI + NSIS)

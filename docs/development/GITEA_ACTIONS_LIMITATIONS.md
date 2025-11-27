# Gitea Actions Platform Limitations

## Overview

Your Gitea Actions runner runs on **Unraid (Linux)**, which limits what platforms can be built automatically.

## Platform Build Matrix

| Platform | Can Build on Linux Runner? | Solution |
|----------|----------------------------|----------|
| **Linux** (AppImage, DEB) | ✅ Yes | Automated via Gitea Actions |
| **macOS** (DMG, Apple Silicon) | ❌ No | Manual build or GitHub Actions |
| **Windows** (MSI, NSIS) | ❌ No | Manual build or GitHub Actions |

## Why Can't We Cross-Compile?

Desktop applications like Tauri have platform-specific dependencies:

### macOS Requirements
- **WebKit** - macOS-only framework
- **Cocoa** - macOS UI framework
- **Code signing** - Requires macOS and Apple Developer account
- **DMG creation** - macOS-specific tools

### Windows Requirements
- **WebView2** - Windows-only web rendering
- **Win32 APIs** - Windows-specific system calls
- **MSVC** - Microsoft Visual C++ compiler
- **MSI/NSIS** - Windows installer formats

### Linux Capabilities
- **Can build**: AppImage, DEB, RPM (native platform)
- **Cannot build**: macOS or Windows binaries

## Current Workflow Setup

### Active Workflow: `release-build-linux.yml`
**What it does:**
- ✅ Creates Gitea release
- ✅ Builds Linux AppImage
- ✅ Builds Linux DEB package
- ✅ Uploads to Gitea release

**Triggers:**
- Tag push: `git push origin v0.11.0`
- Manual: Gitea Actions UI

### Disabled Workflow: `release-build-multiplatform.yml.disabled`
**Why disabled:**
- Requires macOS runner (not available)
- Requires Windows runner (not available)

**When to enable:**
- If you add macOS runner to your infrastructure
- If you add Windows runner to your infrastructure
- If you want to use GitHub Actions as mirror

## Recommended Release Process

### Option 1: GitHub Actions (Recommended - AUTOMATED)

**✅ Best solution for macOS + Windows builds**

Use GitHub's free runners to build, auto-upload to Gitea:

```bash
# 1. Update version
npm run version:set 0.11.0
git add -A && git commit -m "chore: Bump version to 0.11.0"

# 2. Push to both remotes
git push origin main    # Gitea
git push github main    # GitHub

# 3. Create tag (triggers build)
git tag v0.11.0
git push origin v0.11.0
git push github v0.11.0   # Starts automated build

# 4. Wait ~30 mins, builds upload to Gitea automatically
```

**Result:**
- macOS ARM DMG ✅
- macOS Intel DMG ✅
- Windows MSI ✅
- Windows NSIS installer ✅

All uploaded to Gitea release automatically!

**See:** [GITHUB_ACTIONS_SETUP.md](./GITHUB_ACTIONS_SETUP.md)

### Option 2: Hybrid Approach (Manual)

**For Linux builds** (automated):
1. Tag the release: `git tag v0.11.0 && git push origin v0.11.0`
2. Gitea Actions builds Linux packages automatically
3. Packages appear in Gitea release

**For macOS builds** (manual on your Mac):
```bash
# On your Mac
npm run version:set 0.11.0
npm run tauri:build

# Upload to Gitea using script or slash command
./scripts/create-gitea-release.sh 0.11.0
# Or: /gitea-release in Claude Code
```

**Result**: Release has Linux (automated) + macOS (manual) binaries

### Option 2: GitHub Actions Mirror

**Setup:**
```bash
# Add GitHub as remote
git remote add github https://github.com/yourusername/e-fees.git

# Push to both
git push origin main --tags
git push github main --tags
```

**Benefits:**
- GitHub provides free macOS and Windows runners
- Builds all platforms automatically
- Download artifacts and upload to Gitea

**Drawbacks:**
- Code hosted on GitHub (if you want everything private)
- Extra step to transfer artifacts

### Option 3: Add Platform-Specific Runners

**macOS Runner:**
- Old Mac Mini or MacBook
- Install act_runner
- Register with `macos-latest` label

**Windows Runner:**
- Windows PC or VM
- Install act_runner
- Register with `windows-latest` label

**Then**: Enable `release-build-multiplatform.yml.disabled`

## Current Status Summary

✅ **Working:**
- Linux builds automated on Unraid
- Gitea Actions functional
- Runner stable and registered

⚠️ **Limitations:**
- No macOS builds (manual required)
- No Windows builds (manual required)

💡 **Recommendation:**
Continue hybrid approach:
- Let Gitea Actions handle Linux
- Build macOS manually on your Mac
- Windows builds when needed (manual or GitHub mirror)

This is the most practical solution given your infrastructure.

## Future Improvements

If you want full automation in the future:

### Easy: GitHub Actions Mirror
**Time:** 30 minutes setup
**Cost:** Free (GitHub)
**Result:** All platforms automated

### Medium: Dedicated Build Machines
**Time:** 2-4 hours per machine
**Cost:** Hardware (or cloud VMs)
**Result:** Full control, all platforms

### Advanced: Cross-Platform Build Service
**Options:**
- CircleCI (paid)
- Travis CI (paid)
- GitLab CI (self-hosted or cloud)

## See Also

- [UNRAID_GITEA_ACTIONS.md](./UNRAID_GITEA_ACTIONS.md) - Runner setup guide
- [GITEA_RELEASES.md](./GITEA_RELEASES.md) - Release management guide
- `.claude/commands/gitea-release.md` - Gitea release slash command

---

**Last Updated:** November 17, 2025
**Current Setup:** Linux-only automated builds
**Active Workflow:** `release-build-linux.yml`

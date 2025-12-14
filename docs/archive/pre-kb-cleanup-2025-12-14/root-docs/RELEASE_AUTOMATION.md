# Release Automation System

## Overview

The E-Fees project has a fully automated release system with fallback mechanisms to ensure releases can be completed even if GitHub Actions encounters issues.

## Primary Method: GitHub Actions (Automated)

### Trigger a Release

```bash
# 1. Bump version
npm run version:set 0.10.6

# 2. Commit and tag
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: Bump version to 0.10.6"
git tag -a v0.10.6 -m "Release v0.10.6"

# 3. Push to trigger workflow
git push origin main
git push origin v0.10.6
git push github main
git push github v0.10.6
```

### What GitHub Actions Does

1. **Build Jobs** (runs on both macOS and Windows runners):
   - Compiles macOS (Intel + Apple Silicon) and Windows binaries
   - Signs binaries with Tauri signing keys
   - Creates installers (DMG for macOS, EXE/MSI for Windows)
   - Creates updater artifacts (.app.tar.gz, .nsis.zip)
   - Generates cryptographic signatures
   - Uploads artifacts to Gitea release

2. **Manifest Update Job**:
   - Extracts signatures from build artifacts
   - Generates update.json with version and signatures
   - Pushes update.json to main branch

## Fallback Method: Manual Script (If GitHub Actions Fails)

### When to Use

- GitHub Actions billing limit reached
- Workflow failure in manifest update job
- Network issues preventing workflow completion

### Usage

```bash
# Set Gitea token
export GITEA_TOKEN='your_token_here'

# Run completion script with workflow run ID
./scripts/complete-release.sh 19731021585

# Or without run ID (uses latest)
./scripts/complete-release.sh
```

### What the Script Does

1. Downloads artifacts from GitHub Actions run
2. Checks/creates Gitea release
3. Uploads any missing artifacts to Gitea
4. Extracts signatures from artifacts
5. Generates update.json manifest
6. Commits and pushes update.json to repository

## Update Manifest (update.json)

### Purpose

The update.json file tells the Tauri updater:
- Current version available
- Download URLs for each platform
- Cryptographic signatures to verify downloads

### Format

```json
{
  "version": "0.10.5",
  "notes": "Release v0.10.5",
  "pub_date": "2025-11-27T09:10:10Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "base64_signature_here",
      "url": "https://git.mms.name/martin/fee-prop/releases/download/v0.10.5/E-Fees_0.10.5_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "base64_signature_here",
      "url": "https://git.mms.name/martin/fee-prop/releases/download/v0.10.5/E-Fees_0.10.5_x64.app.tar.gz"
    }
  }
}
```

### Tauri Configuration

The app checks for updates using the endpoint configured in `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://git.mms.name/martin/fee-prop/raw/branch/main/update.json"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6..."
    }
  }
}
```

## Troubleshooting

### Build artifacts missing signatures

**Symptom**: Windows auto-update doesn't work (no .nsis.zip.sig file)

**Cause**: Tauri signing keys not available or build configuration issue

**Fix**: Check that `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets are set in GitHub

### GitHub Actions billing error

**Symptom**: "Recent account payments have failed or your spending limit needs to be increased"

**Fix**: Use the fallback script or update billing settings

### Release already exists error

**Symptom**: Cannot create release, says it already exists

**Fix**: The build jobs create the release - this is normal. The script handles existing releases.

## Release Checklist

- [ ] Version bumped in package.json and tauri.conf.json
- [ ] Changes committed with descriptive message
- [ ] Tag created and pushed
- [ ] GitHub Actions workflow completed (or fallback script run)
- [ ] Gitea release has all artifacts
- [ ] update.json pushed to main branch
- [ ] Test auto-update on macOS (both architectures)
- [ ] Verify manual downloads work (Windows users)

## Future Improvements

### Priority 1: Fix Windows Auto-Update

Currently Windows only gets installers, not updater artifacts. Need to:
1. Investigate why .nsis.zip files aren't being created
2. Ensure signing keys are properly configured
3. Test Windows updater functionality

### Priority 2: Self-Hosted Runners

To avoid GitHub Actions billing limits:
1. Set up self-hosted macOS runner (required for code signing)
2. Set up self-hosted Windows runner
3. Update workflow to use self-hosted runners

### Priority 3: Direct Gitea CI/CD

Migrate entirely to Gitea Actions:
1. Configure Gitea Actions on git.mms.name server
2. Create equivalent workflows in `.gitea/workflows/`
3. Remove dependency on GitHub Actions

## Security Notes

- **Signing Keys**: Never commit `TAURI_SIGNING_PRIVATE_KEY` to repository
- **Gitea Token**: Store in environment, never in scripts or code
- **Signatures**: Required for auto-updates - validates authenticity
- **Public Key**: In tauri.conf.json, safe to commit

## Related Files

- `.github/workflows/build-releases.yml` - GitHub Actions workflow
- `scripts/complete-release.sh` - Fallback release script
- `scripts/generate-update-manifest.cjs` - Manifest generator (legacy)
- `scripts/sync-version.cjs` - Version synchronization
- `update.json` - Current update manifest (auto-generated)

## Support

For issues or questions about the release process:
1. Check GitHub Actions logs: https://github.com/newillusions/e-fees/actions
2. Check Gitea releases: https://git.mms.name/martin/fee-prop/releases
3. Review this documentation
4. Contact development team

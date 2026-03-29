# Known Issues and Limitations

**Last Updated**: 2025-12-14
**Current Version**: v0.10.24

## Critical Issues

### ~~1. Auto-Updater Download Fails Silently~~

**Status**: ✅ RESOLVED in v0.10.17/v0.10.18

**Root Causes Found**:
1. **Double base64 encoding**: Signatures were being base64 encoded twice in publish script
2. **Windows artifact format**: Script looked for `.nsis.zip` but artifact is `.exe`
3. **Windows URL format**: URLs pointed to macOS format instead of Windows exe

**Fixes Applied**:
- Removed extra `| base64` pipe from signature reading in publish script
- Updated script to find `*_x64-setup.exe` files for Windows
- Added `"installMode": "passive"` for Windows seamless updates
- Rebranded to lowercase "e-fees" for consistency

---

## Medium Priority Issues

### 2. Update URLs Not Configurable

**Status**: 🟡 LIMITATION - By Design

**Description**:
User preference: "we should NOT include the full URL in the public json file. the base url should be hard coded into the app"

**Current Behavior**:
update.json contains absolute URLs:
```json
{
  "platforms": {
    "darwin-aarch64": {
      "url": "https://apache.mms.name/e-fees-releases/0.10.16/E-Fees_aarch64.app.tar.gz"
    }
  }
}
```

**Investigation Result**:
Tauri v2 updater requires absolute URLs in update.json. The plugin does not support:
- Relative paths
- Base URL configuration
- URL templates with version interpolation

**Workaround**:
- update.json MUST contain full URLs
- Keep update.json in private GitHub repository
- Only binaries are publicly accessible on Apache server

**Reference**: https://v2.tauri.app/plugin/updater/

---

### 3. No Dev Mode for Production Debugging

**Status**: 🟡 PLANNED FOR v0.10.17

**Description**:
Production builds don't have access to developer console or verbose logging, making debugging difficult.

**Impact**:
- Cannot easily debug issues in production app
- Must rely on file logging which isn't always captured
- No way to enable verbose logging without rebuild

**Planned Solution**:
Implement dev mode toggle:
- Add `DEV_MODE=false` to e-fees.config file
- Add UI toggle in settings page
- When enabled:
  - Verbose logging to file and console
  - Debug menu items
  - Additional diagnostic information
  - Manual update check button

**Related**: Issue #1 (would have helped diagnose updater failure)

---

### 4. Windows Build Not Tested

**Status**: 🟡 INCOMPLETE

**Description**:
Windows builds are created by GitHub Actions but have never been tested.

**Known Gaps**:
- No Windows signature in update.json (empty string)
- Windows update flow never verified
- No test environment for Windows

**Signature Issue**:
```json
"windows-x86_64": {
  "signature": "",  // Empty - may cause updates to fail
  "url": "https://apache.mms.name/e-fees-releases/0.10.16/E-Fees_x64-setup.nsis.zip"
}
```

**Next Steps**:
- Set up Windows test environment
- Verify Windows builds install correctly
- Test Windows update flow
- Fix signature generation if needed

---

## Low Priority Issues

### 5. Configurable Download/Install Location

**Status**: 🟡 FUTURE ENHANCEMENT

**Description**:
Users should be able to configure where downloaded update files are saved, and potentially customize the installation location.

**Current Behavior**:
- Update files downloaded to system temp directory
- Windows installs to default location (AppData/Local/e-fees)
- No user control over these paths

**Proposed Solution**:
- Add "Download Location" setting in Settings modal
- Store preference in e-fees.config file
- Pass custom path to updater when downloading
- Consider custom install path option for Windows

**Use Cases**:
- Users with limited space on system drive
- Corporate environments with specific installation policies
- Users who prefer portable installations

**Related Files**:
- `src/lib/components/SettingsModal.svelte`
- `src-tauri/src/commands/mod.rs` (AppSettings struct)

---

### 6. Multiple Background Processes (dev only)

**Status**: 🟢 MINOR ANNOYANCE

**Description**:
Multiple background bash processes running from previous testing:
- Multiple `npm run tauri:dev` processes
- Update test logging processes
- Script execution processes

**Impact**: Minimal - processes are isolated and don't interfere

**Cleanup**:
```bash
# Kill all E-Fees related processes
pkill -f "npm run tauri"
killall "app" "E-Fees"
```

---

### 7. Git Dual Remote Management

**Status**: 🟢 OPERATIONAL BUT COMPLEX

**Description**:
Repository has two remotes requiring dual push:
- `origin`: Forgejo (primary, private — forge.mms.name)
- `github`: GitHub (mirror for CI/CD)

**Current Workflow**:
```bash
git push origin main
git push github main
```

**Potential Issues**:
- Easy to forget to push to both
- Can create divergent history if one push fails
- No automated sync between remotes

**Mitigation**:
- Always push to both in release process
- Document clearly in RELEASE_PROCESS.md
- Consider git alias for dual push

---

### 8. Minisign Signature Filename Dependency

**Status**: 🟢 UNDERSTOOD AND HANDLED

**Description**:
Minisign signatures embed the filename in the signed data.

**Impact**:
- Cannot rename files after signing
- Signature verification fails if filename changes
- Must preserve exact filenames from build

**Solution**:
publish-release.sh script preserves original filenames from artifacts.

**Example**:
```bash
# This breaks signature verification:
mv E-Fees.app.tar.gz E-Fees_aarch64.app.tar.gz

# Must keep original name or re-sign
```

---

## Resolved Issues

### ✅ Update Detection Not Working
**Resolved**: v0.10.15 (permissions fix)
**Solution**: Added proper Tauri capabilities for updater plugin

### ✅ Artifact Download Conflicts
**Resolved**: v0.10.16 release script
**Solution**: Download artifacts to separate subdirectories

### ✅ Files Not Published to Web Server
**Resolved**: v0.10.16 release script
**Solution**: Use find commands to locate files in nested artifact structure

### ✅ Git Push Conflicts During Release
**Resolved**: Manual merge and recommit
**Solution**: Always pull before pushing update.json

---

## Issue Tracking

For new issues, document:
1. **Symptoms**: What's happening?
2. **Impact**: How critical is it?
3. **Reproduction**: Steps to reproduce
4. **Investigation**: What's been tried?
5. **Workarounds**: Temporary solutions
6. **Next Steps**: Plan to resolve

---

## Priority Definitions

- 🔴 **CRITICAL**: Blocks core functionality, needs immediate attention
- 🟡 **MEDIUM**: Impacts usability or development workflow, should be addressed soon
- 🟢 **LOW**: Minor annoyance or known limitation, can be lived with

---

## Quick Reference: Most Common Issues

1. **Update fails to download** → See Issue #1 (working on v0.10.17 fix)
2. **Can't see production logs** → See Issue #3 (dev mode coming in v0.10.17)
3. **Forgot to push to both git remotes** → See Issue #6 (push to origin AND github)
4. **Release script fails** → Check RELEASE_PROCESS.md troubleshooting section

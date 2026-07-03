# Known Issues and Limitations

**Last Updated**: 2026-07-03 (refreshed against v0.16.0 codebase - see CLAUDE.md for current version; supersedes the 2025-12-14 / v0.10.24-era version)
**Current Version**: v0.16.0

This file previously listed 8 issues dated 2025-12-14 (v0.10.24 era). Verified against the current codebase: 2 were resolved and dropped, 6 carried forward (verification noted per item). Full prior text is in git history (`git log -- KNOWN_ISSUES.md`).

## Medium Priority Issues

### 1. Update URLs Not Configurable

**Status**: 🟡 LIMITATION - By Design (still real)

**Description**: Tauri v2's updater plugin requires an absolute URL in its `endpoints` config; it does not support relative paths, base-URL config, or URL templating.

**Current Behavior**: `src-tauri/tauri.conf.json` hardcodes `"endpoints": ["https://raw.githubusercontent.com/newillusions/e-fees/main/update.json"]`. update.json itself is generated per-release by the `/release` skill and pushed via Forgejo API, then synced to GitHub (Tauri only reads the GitHub raw URL).

**Reference**: https://v2.tauri.app/plugin/updater/

---

### 2. Windows Build Signing Present, Manual Verification Unconfirmed

**Status**: 🟡 [UNVERIFIED - carried from 2025-12 version, partially updated]

**Description**: `.github/workflows/build-releases.yml` has a `build-windows` job that produces a signed `.nsis.zip` installer with a real signature output (`sig-windows`), so the "empty signature" issue from the 2025-12 version is resolved at the CI level. Whether the resulting Windows installer and update flow have actually been tested on real Windows hardware is not verifiable from the repo alone.

**Verification source**: `.github/workflows/build-releases.yml:221-394` (build-windows job + sig-windows output wired into the release manifest step).

---

## Low Priority Issues

### 3. Configurable Download/Install Location

**Status**: 🟡 FUTURE ENHANCEMENT (still real - not implemented)

**Description**: Update files always go to the system temp directory; Windows always installs to the default AppData location. No UI or config setting exists to change either path.

**Verification**: no `download location` / `DownloadLocation` / `install location` matches anywhere in `src/` or `src-tauri/` (grep, 2026-07-03).

**Related Files**: `src/lib/components/SettingsModal.svelte`, `src-tauri/src/commands/mod.rs`

---

### 4. Multiple Background Processes (dev only)

**Status**: 🟢 MINOR ANNOYANCE (still real, dev-machine only)

**Description**: Repeated `npm run tauri:dev` runs during local development can leave stray background processes (dev server, test loggers). Isolated to the developer's own machine; does not affect shared/remote systems.

**Cleanup**: use a targeted process check (e.g. `pgrep -x` on the known binary name) rather than a broad `pkill -f` pattern-match, which risks matching unrelated processes on the same machine.

---

### 5. Git Dual Remote Management

**Status**: 🟢 OPERATIONAL BUT COMPLEX (still real)

**Description**: The repo has two remotes: `origin` (Forgejo, `forge.mms.name/emittiv/fee-prop` - primary, daily work) and `github` (`github.com/newillusions/e-fees` - release/CI mirror only, per CLAUDE.md "Always ask before committing" directive). The `/release` skill's CI pipeline builds on GitHub and syncs `update.json` back to Forgejo, then to GitHub - so both remotes stay in sync automatically for releases; manual dual-push is only a risk for out-of-band changes.

**Verification source**: `git remote -v` (2026-07-03) shows both remotes present; `.claude/commands/release.md` documents the automated cross-remote sync.

---

### 6. Minisign Signature Filename Dependency

**Status**: 🟢 UNDERSTOOD AND HANDLED (still real, technical constraint)

**Description**: Minisign signatures embed the filename in the signed data - renaming a signed artifact after signing breaks verification. The release pipeline must preserve original build filenames end to end.

**Verification**: `scripts/publish-release.sh` still present in the repo; this constraint is inherent to minisign, not tied to any specific script version.

---

## Resolved Since 2025-12 (dropped from active list)

- **Auto-updater silent download failures** - resolved in v0.10.17/v0.10.18 (double-base64 signature bug, Windows artifact/URL format). No longer applicable at v0.16.0.
- **No dev mode for production debugging** - implemented: `dev_mode` field now exists in the settings store (`src/lib/stores/settings.ts:26,50`), toggled via `SettingsModal.svelte`, and gates verbose logging in `UpdateNotification.svelte:20-44` and dev-only nav items in `Navigation.svelte:8,51`.

---

## Issue Tracking

For new issues, document:
1. **Symptoms**: What's happening?
2. **Impact**: How critical is it?
3. **Reproduction**: Steps to reproduce
4. **Investigation**: What's been tried?
5. **Workarounds**: Temporary solutions
6. **Next Steps**: Plan to resolve

## Priority Definitions

- 🔴 **CRITICAL**: Blocks core functionality, needs immediate attention
- 🟡 **MEDIUM**: Impacts usability or development workflow, should be addressed soon
- 🟢 **LOW**: Minor annoyance or known limitation, can be lived with

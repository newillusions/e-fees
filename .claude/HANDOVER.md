# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. **v0.12.3** — Windows install migration fix + command cleanup.

- **Version**: 0.12.3 (released, tag v0.12.3, Forgejo + GitHub)
- **Branch**: `main`
- **Database**: SurrealDB v3 @ ws://surreal-dev.internal:8000 (emittiv/projects)
- **Tests**: 84/86 Rust tests passing (2 pre-existing excel template failures)

## Last Session (2026-02-22)
**Summary**: Fixed Windows install path issue with NSIS migration hook, cleaned up release commands (Gitea→Forgejo), released v0.12.3 via background haiku agent.

### Windows NSIS Migration Hook
- **Root cause**: v0.10.22 changed `installMode` from `currentUser` to `perMachine`, leaving orphaned per-user install at `%LOCALAPPDATA%` with stale desktop shortcuts
- **Fix**: Created `src-tauri/windows/hooks.nsh` with `NSIS_HOOK_PREINSTALL` macro
- Hook detects old HKCU registry entry, runs old uninstaller silently, cleans up stale shortcuts
- Added `"installerHooks": "./windows/hooks.nsh"` to tauri.conf.json
- **Pending verification**: User will test on Windows machine

### Command Cleanup
- Deleted old `/gitea-release` command (superseded by `/release`)
- Moved `/release` from `.claude/skills/` to `.claude/commands/` (shows in autocomplete)
- Updated `commands.md` and `README.md` — all Gitea references replaced with Forgejo
- Added **mandatory background agent release process** to CLAUDE.md

### Release v0.12.3
- Full pipeline ran via background haiku agent (35K tokens, 28 min, fully autonomous)
- 8 assets on Forgejo release (DMGs, updater tarballs, MSI, NSIS installer)
- update.json synced to GitHub for Tauri updater endpoint
- CLAUDE.md now mandates: releases MUST use background haiku agent, never interactive

## Next Steps (Priority Order)
1. **Verify Windows install** — Test that NSIS hook cleans up old per-user install and creates correct desktop shortcut
2. **Multi-currency hover** — AED equivalents on hover when quoting in foreign currency
3. **Verify export save dialog** — Clean rebuild + end-to-end test

## Key Technical Context

### Release Process (MANDATORY)
Always use `/release` via a **background haiku agent**. Never run interactively.
- Spawn: `Task(subagent_type: "Bash", model: "haiku", run_in_background: true)`
- Feed it the full pipeline from `.claude/commands/release.md`
- Only intervene if the background agent reports a build failure

### Real-Time Log Streaming
```bash
# Stream logs in real time
curl -N http://localhost:3100/api/logs/stream

# Check current log level and file path
curl http://localhost:3100/api/health

# Or just tail the log file
tail -f ~/Library/Logs/com.emittiv.e-fees/E-Fees.log
```

### Build & Release Process
Use `/release` command for the complete workflow. Summary:
1. `node scripts/sync-version.cjs <VERSION>` — bumps all 3 files + window title
2. Commit, push to both remotes, create `v*` tag → triggers GitHub Actions
3. CI builds macOS (aarch64 + x64) + Windows, uploads to Forgejo releases
4. CI pushes update.json to Forgejo via API
5. Pull update.json from Forgejo, push to GitHub (for Tauri updater endpoint)

### CSS Architecture (Post-Restructure)
- `app.css` has 3 layers: `@layer base`, `@layer components`, `@layer utilities`
- ~130 `.emittiv-*` semantic classes in components layer
- All values in fixed `px` (desktop app, OS handles DPI scaling)

### Critical Rules
1. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
2. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission
6. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
7. **Releases**: ALWAYS background haiku agent. Never interactive polling.

### Key Files
| Purpose | Location |
|---------|----------|
| DB entity types | `src-tauri/src/db/types.rs` |
| DB operations | `src-tauri/src/db/operations.rs` |
| Master CSS | `src/styles/app.css` |
| Excel export template | `src-tauri/src/excel_export.rs` |
| Logging system | `src-tauri/src/agent_server.rs` (SSE endpoint) |
| Log level commands | `src-tauri/src/commands/system.rs` |
| NSIS migration hook | `src-tauri/windows/hooks.nsh` |
| Release command | `.claude/commands/release.md` |
| Window config | `src-tauri/tauri.conf.json` |

---
*Updated: 2026-02-22*

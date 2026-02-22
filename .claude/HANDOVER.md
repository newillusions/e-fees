# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. **v0.12.2** — SurrealDB v3 migration complete, all entity pages loading.

- **Version**: 0.12.2 (released, tag v0.12.2, Forgejo + GitHub)
- **Branch**: `main`
- **Database**: SurrealDB v3 @ ws://surreal-dev.internal:8000 (emittiv/projects)
- **Tests**: 86 Rust tests passing, 8 pre-existing svelte-check warnings

## Last Session (2026-02-20)
**Summary**: Released v0.12.2. Fixed SurrealDB v3 deserialization (prior session), then executed full release pipeline and rewrote `/release` skill.

### Release v0.12.2
- All CI jobs passed (macOS aarch64 + x64, Windows)
- 8 assets on Forgejo release (DMGs, updater tarballs, MSI, NSIS installer)
- update.json synced to GitHub for Tauri updater endpoint
- Verified working on macOS; Windows pending user test

### Release Skill Rewrite
- `.claude/skills/release.md` — removed outdated Apache web server references
- Now documents actual pipeline: GitHub Actions → Forgejo releases → update.json sync
- Trigger: `/release [patch|minor|major|<exact version>]`

### Backups Run
- KB DB: 27 MB exported to Unraid Primary
- E-Fees DB: 161 KB exported
- Config: settings.json + active-projects.json
- Git: fee-prop mirror cloned
- Note: Dev DB container networking broken (10.0.23.12 unreachable), Forgejo auth needed for kb-agent/ailx git mirrors

### SurrealDB v3 Migration Notes
- **v3 binary protocol**: Uses `SurrealValue` deserialization (not serde JSON). Native types `record` and `datetime` can't map to `serde_json::Value` or `String`.
- **SurrealValue derive limitation**: Does NOT respect `#[serde(rename = "...")]`. Only `payment_schedule` needed `serde_json::Value` passthrough.
- **HTTP export API (v3)**: Must use GET (not POST) and include `Content-Type: application/json` header.
- **Crates**: `surrealdb = "3.0"`, `surrealdb-types = "3.0"`

## Next Steps (Priority Order)
1. **Windows install path issue** — Updater works and DB connects, but app install path seems wrong and desktop icons keep reverting to previous version. Investigate NSIS installer config (`tauri.conf.json` → `bundle.windows.nsis`), shortcut creation, and whether `installMode: "perMachine"` causes path conflicts on update.
2. **Multi-currency hover** — AED equivalents on hover when quoting in foreign currency
3. **Verify export save dialog** — Clean rebuild + end-to-end test

## Key Technical Context

### Real-Time Log Streaming
```bash
# Stream logs in real time
curl -N http://localhost:3100/api/logs/stream

# Check current log level and file path
curl http://localhost:3100/api/health

# Set log level (with API key)
curl -X POST http://localhost:3100/api/logs/level \
  -H "X-API-Key: $KEY" -H "Content-Type: application/json" \
  -d '{"level":"debug"}'

# Or just tail the log file
tail -f ~/Library/Logs/com.emittiv.e-fees/E-Fees.log
```

### Build & Release Process
Use `/release` skill for the complete workflow. Summary:
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

### Key Files
| Purpose | Location |
|---------|----------|
| DB entity types | `src-tauri/src/db/types.rs` |
| DB operations | `src-tauri/src/db/operations.rs` |
| Master CSS | `src/styles/app.css` |
| Excel export template | `src-tauri/src/excel_export.rs` |
| Logging system | `src-tauri/src/agent_server.rs` (SSE endpoint) |
| Log level commands | `src-tauri/src/commands/system.rs` |
| Settings modal | `src/lib/components/SettingsModal.svelte` |
| Stage codes panel | `src/lib/components/pricing/StagesPanel.svelte` |
| Window config | `src-tauri/tauri.conf.json` |

---
*Updated: 2026-02-20*

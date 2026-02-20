# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. **v0.12.2** — SurrealDB v3 migration complete, all entity pages loading.

- **Version**: 0.12.2 (released, tag v0.12.2, Forgejo + GitHub)
- **Branch**: `main`
- **Database**: SurrealDB v3 @ ws://surreal-dev.internal:8000 (emittiv/projects)
- **Tests**: 86 Rust tests passing, 8 pre-existing svelte-check warnings

## Last Session (2026-02-20)
**Summary**: Fixed SurrealDB v3 binary protocol deserialization failures that caused all entity pages to show empty data. Root cause: v3 SDK sends native `datetime` and `record` types that can't deserialize into `String` or `serde_json::Value`.

### Changes (commits 7df9d1a, 1484fe4)
- `src-tauri/src/db/types.rs` — TimeStamps: String → surrealdb_types::Datetime; ActivityLog.timestamp: String → Datetime; Fee.payment_schedule: PaymentSchedule → serde_json::Value (SurrealValue derive ignores #[serde(rename)])
- `src-tauri/src/db/operations.rs` — Removed verbose diagnostic logging from entity fetch functions
- `src-tauri/src/agent_server.rs` — TimeStamps construction: chrono::Utc::now().to_rfc3339() → Datetime::now()
- `src-tauri/src/excel_export.rs` — Test helper: string timestamps → Datetime::default()
- Version bumped to 0.12.2

### SurrealDB v3 Migration Notes
- **v3 binary protocol**: Uses `SurrealValue` deserialization (not serde JSON). Native types `record` and `datetime` can't map to `serde_json::Value` or `String`.
- **SurrealValue derive limitation**: Does NOT respect `#[serde(rename = "...")]`. PaymentScheduleEntry has `#[serde(rename = "type")] pub payment_type: String` — serde writes `type` to DB, but SurrealValue reads `payment_type` → field not found → deserialization fails.
- **Workaround**: Use `serde_json::Value` passthrough for structs with serde renames. Only `payment_schedule` needed this.
- **Crates**: `surrealdb = "3.0"`, `surrealdb-types = "3.0"`

## Next Steps (Priority Order)
1. **Multi-currency hover** — AED equivalents on hover when quoting in foreign currency
2. **Verify export save dialog** — Clean rebuild + end-to-end test

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

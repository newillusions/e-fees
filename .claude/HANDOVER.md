# E-Fees Project Handover

## Current Status
v0.16.0 released + config rename committed. All `.env`/`.env.dev` files renamed to `e-fees.config`/`e-fees.config.dev` across codebase and app data directories. Full formatting pass applied (rustfmt + prettier, 221 files).

- **Version**: 0.16.0 (desktop app + API)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Dev Database**: SurrealDB v3.0.4 @ ws://10.0.23.12:8000

## Last Session
**Date**: 2026-03-29
**Summary**: Renamed config files from `.env`/`.env.dev` to `e-fees.config`/`e-fees.config.dev` — updated settings.rs (filename constants, log messages, doc comments), lib.rs (dotenvy::from_filename with debug/release selection), folder_management.rs, types.rs, fees.rs, system.rs. Updated .gitignore, renamed template/example files, renamed app data dir configs. Verified via Tauri MCP: read, write, round-trip, and hot reload all working. Ran full formatting pass (rustfmt + prettier). Verified DELETE ME test data already cleaned. Confirmed Proposals page bug fixed (32 proposals rendering). Replied to AILX hub message with full API reference for PA → E-Fees → AILX RFP integration flow. Subscribed to emittiv-ecosystem hub topic.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 v3.0.4 |
| API Container | 10.0.21.80:3200 (e-fees-api) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope) |
| Scope API Key | efees-scope-2026-s7k2m9xp |
| Forgejo Release | forge.mms.name/emittiv/fee-prop/releases/tag/v0.16.0 |
| Ollama | 10.0.21.20:11434 |
| App data dir (prod) | ~/Library/Application Support/com.emittiv.e-fees/ |
| App data dir (dev) | ~/Library/Application Support/com.emittiv.e-fees.dev/ |

## Next Steps
1. **InDesign export** — pricing tables (T0-T4) via UXP scripting (most complex remaining item)
2. **Config rename in CLAUDE.md** — update documentation references to use `e-fees.config` instead of `.env`
3. **App data dir config alignment** — dev app data config had stale `surreal-dev.internal` hostname, updated to `10.0.23.11` during testing but may need review
4. **DB reload reconnection** — `reload_database_config` reads config correctly but WebSocket reconnect fails (pre-existing issue with SurrealDB SDK reconfigure)

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency, scope viewer, stage autocomplete, country normalization
- **Shared core** (`crates/e-fees-core/`): Domain types + export logic + `set_pricing_stages()`
- **Standalone API** (`e-fees-api/`): Full CRUD, search, JSON export, OpenAPI, country normalization
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation, stage dictionary
- **Container-utils** (`forge.mms.name/emittiv/container-utils`): Shared ConfigManager + health routes

## Critical Rules
1. **Config files**: Now `e-fees.config` (prod) and `e-fees.config.dev` (debug) — NOT `.env`
2. **dotenvy**: Uses `dotenvy::from_filename()` with debug/release selection, NOT `dotenvy::dotenv()`
3. **App data dir**: Config files live in `~/Library/Application Support/com.emittiv.e-fees/` (prod) and `.dev/` variant
4. **fn::resolve_country**: 7-step cascade, returns `{name, code, iso2, dial_code}` or NONE
5. **Stage dictionary**: Scope service `/stages` endpoint, cached in frontend session
6. **SurrealValue i64**: Use `i64` not `Option<i64>` for fields with DEFAULT 0

---
*Updated: 2026-03-29*

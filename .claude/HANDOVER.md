# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. Standalone API with full CRUD, pagination, and OpenAPI docs deployed.

- **Version**: 0.13.2 (released)
- **Branch**: main (clean)
- **Database**: SurrealDB v3 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 87 Rust tests + 47 API integration tests passing
- **API**: e-fees-api v0.2.7 deployed at 10.0.21.80:3200 (Docker, br0 network)

## Last Session
**Date**: 2026-03-02
**Summary**: Hardened desktop app fee create/update with parameterized `type::record()` bindings. Implemented full bulk operations feature (multi-select, batch delete, batch status change) across all 4 entity pages. Confirmed detail views, DB mutex, and N+1 queries were already resolved.

### Accomplished
- Hardened `client.rs` fee create/update: replaced string interpolation with `type::record('table', $param)` parameterized bindings
- Added `query_bind_map()` helper for multiple named bindings on SurrealDB queries
- Implemented batch operations backend: `batch_delete()`, `batch_update_status()`, `validate_table_name()` in `DatabaseClient`
- Forwarded batch ops through `DatabaseManager` (`operations.rs`)
- Created Tauri commands: `batch_delete_entities`, `batch_update_status` (`commands/batch_ops.rs`)
- Built `BulkActionBar.svelte`: selection count, status change dropdown, delete with confirmation, clear
- Updated `BaseListCard.svelte` with `selectable`/`selected` props and checkbox
- Updated all 4 card components (ProjectCard, ProposalCard, CompanyCard, ContactCard) with selection passthrough
- Added select mode toggle + bulk action bar to all 4 list pages (Projects, Proposals, Companies, Contacts)
- Created `src/lib/api/batch.ts` API module with table name mapping
- Projects and Proposals get status change + delete; Companies and Contacts get delete only
- Added CSS: `.emittiv-bulk-bar`, `.emittiv-checkbox`, `.emittiv-card--selected`
- Confirmed: detail views (already built), DB mutex (already RwLock), N+1 queries (already O(1) Maps) — no work needed
- Cleaned up HANDOVER.md, CLAUDE.md, MEMORY.md to remove stale items

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 (ns: emittiv, db: projects) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 (br0, e-fees-api:v0.2.7) |
| API .env | /mnt/user/appdata/e-fees-api/.env (on AI server) |
| API Key | efees-api-2026-k8x9m4pq |
| Installed app config | ~/Library/Application Support/com.emittiv.e-fees/.env |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, Nextcloud sync
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop & API
- **Standalone API** (`e-fees-api/`): Full CRUD HTTP endpoints with API key auth, pagination, OpenAPI/Swagger docs
- API deployed as Docker container on Unraid AI server (br0 network, ipvlan L2)

## API Endpoints (all verified working, 47/47 tests passing)
- `GET /health` — Public, returns service + DB status
- `GET /stats` — Entity counts
- `GET /projects`, `POST /projects`, `GET/PUT/DELETE /projects/{id}`
- `GET /fees`, `POST /fees`, `GET/PUT/DELETE /fees/{id}`
- `GET /companies`, `POST /companies`, `GET/PUT/DELETE /companies/{id}`
- `GET /contacts`, `POST /contacts`, `GET/PUT/DELETE /contacts/{id}`
- `GET /docs` — Swagger UI (OpenAPI)
- All data endpoints require `X-API-Key` header
- List endpoints support `?page=1&page_size=50` pagination

## Next Steps
1. Multi-currency quoting completion (partially implemented, exchange rate service done)
2. InDesign export functionality

## Critical Rules
1. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
2. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission
6. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
7. **Releases**: ALWAYS background haiku agent. Never interactive polling.
8. **Test DB**: Must match the installed app's DB (10.0.23.11).
9. **Production safety**: API integration tests MUST refuse to run against 10.0.23.11.
10. **SurrealDB v3 NULL**: Never bind `json!(None)` — omit optional fields from SET clause entirely.
11. **Fee issue_date**: YYYYMM format (6-digit numeric string per DB ASSERT), not ISO date.

---
*Updated: 2026-03-02*

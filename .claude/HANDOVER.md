# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend. Standalone API service deployed.

- **Version**: 0.13.1 (released, all builds verified)
- **Branch**: main (clean, feat/ui-testing-and-api merged)
- **Database**: SurrealDB v3 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 87 Rust tests passing, svelte-check 0 errors
- **API**: e-fees-api deployed at 10.0.21.80:3200 (Docker, br0 network)

## Last Session
**Date**: 2026-02-28
**Summary**: Completed full 15-task implementation plan (UI smoke tests + core library extraction + standalone API service). Deployed API to Unraid AI server. Code review found and fixed critical auth vulnerability (empty API key bypass). Merged feat/ui-testing-and-api branch to main.

### Accomplished
- Tasks 12-15: API route handlers, integration tests, Dockerfile, deployment
- Fixed `rfp` → `fee` table name mismatch (v0.13.0 domain restructure)
- Fixed critical security issue: empty API key bypass via `unwrap_or_default()`
- Rewrote auth.rs to use AppState-based key validation
- Added missing `/contacts/{id}` endpoint
- Added Cargo.lock to Dockerfile for deterministic builds
- Removed unused deps (uuid, chrono) from e-fees-core
- Full code review cycle with fixes applied
- Merged to main, pushed to Forgejo, verified compilation

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 (ns: emittiv, db: projects) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 (br0, e-fees-api:0.1.0) |
| API .env | /mnt/user/appdata/e-fees-api/.env (on AI server) |
| API Key | efees-api-2026-k8x9m4pq |
| Installed app config | ~/Library/Application Support/com.emittiv.e-fees/.env |
| Forgejo release | https://forge.mms.name/emittiv/fee-prop/releases/tag/v0.13.1 |

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, Nextcloud sync
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop & API
- **Standalone API** (`e-fees-api/`): Read-only HTTP endpoints with API key auth
- API starts read-only, grows toward write parity

## API Endpoints (all verified working)
- `GET /health` — Public, returns service + DB status
- `GET /stats` — Entity counts (projects, fees, companies, contacts)
- `GET /projects`, `/projects/{id}` — Project data
- `GET /fees`, `/fees/{id}` — Fee proposal data
- `GET /companies`, `/companies/{id}` — Company data
- `GET /contacts`, `/contacts/{id}` — Contact data
- All data endpoints require `X-API-Key` header

## Deployment Notes
- **ipvlan L2 limitation**: Unraid host cannot curl its own containers — test from external machine
- **SSH to AI server**: Use `host: 10.0.20.11, username: root` (no alias saved)
- **Rebuild cycle**: Push → SSH pull → `docker build` → stop/rm/run → verify (~5min)

## Next Steps
1. Run smoke tests (Task 6) — requires Tauri app running
2. User has upcoming RFP to prepare — app needs to be fully working
3. Consider API enhancements: pagination, write endpoints, OpenAPI docs

## Domain Model
- **Fee statuses**: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded
- **Project statuses**: Lead, RFP, Submitted, Awarded, Design, Construction, Completed, Lost, No Response, Cancelled, On Hold, Superseded
- **DB table**: `fee` (renamed from `rfp` in v0.13.0)

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

---
*Updated: 2026-02-28*

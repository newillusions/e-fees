# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB v3 backend.

- **Version**: 0.13.1 (released, all builds verified)
- **Branch**: main (clean)
- **Database**: SurrealDB v3 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 87 Rust tests passing, svelte-check 0 errors

## Last Session
**Date**: 2026-02-28
**Summary**: Verified v0.13.1 release completed (GitHub Actions build + Forgejo release with 8 assets + update.json synced to GitHub). Cleaned up domain-model-restructure worktree and branch. Designed and planned UI testing + standalone API service.

### Design & Planning Complete
- **Design doc**: `docs/plans/2026-02-28-ui-testing-and-api-design.md` — Approved
- **Implementation plan**: `docs/plans/2026-02-28-ui-testing-and-api-plan.md` — 15 tasks across 3 phases
- **Execution method**: Subagent-driven development (chosen, not yet started)

## Next Steps — Implementation Plan
**Use `superpowers:subagent-driven-development` to execute the plan.**

### Phase 1: UI Smoke Tests (Tasks 1-6) — Independent
1. Create smoke check JS snippets (`e2e-mcp/suites/helpers/smoke-checks.ts`)
2. Create DOM validation checks (`e2e-mcp/suites/helpers/dom-checks.ts`)
3. Create smoke test runbook (`e2e-mcp/suites/smoke.md`)
4. Create executable smoke test script (`e2e-mcp/suites/run-smoke.ts`)
5. Add `/smoke-test` slash command (`.claude/commands/smoke-test.md`)
6. Run smoke tests against v0.13.1 — validate and fix

### Phase 2: Core Library Extraction (Tasks 7-9) — Independent of Phase 1
7. Create `crates/e-fees-core/` workspace crate skeleton
8. Extract domain models from `src-tauri/src/db/types.rs` into core
9. Refactor Tauri app to depend on `e-fees-core` — verify 87 tests still pass

### Phase 3: API Service (Tasks 10-15) — Depends on Phase 2
10. Create `e-fees-api/` axum skeleton with health endpoint
11. Add API key auth middleware
12. Add read-only route handlers (projects, fees, companies, contacts, stats)
13. Add integration tests with production safety guard
14. Create Dockerfile
15. Deploy to Unraid (Docker, br0 network, 10.0.21.x range, port 3200)

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 (ns: emittiv, db: projects) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| Installed app config | ~/Library/Application Support/com.emittiv.e-fees/.env |
| App logs | ~/Library/Logs/com.emittiv.e-fees/E-Fees.log |
| Forgejo release | https://forge.mms.name/emittiv/fee-prop/releases/tag/v0.13.1 |
| Update endpoint | https://raw.githubusercontent.com/newillusions/e-fees/main/update.json |

## Domain Model (Post-Restructure)
- **Project statuses**: Lead, RFP, Submitted, Awarded, Design, Construction, Completed, Lost, No Response, Cancelled, On Hold, Superseded
- **Fee statuses**: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded
- **Venue**: Removed from app. Venue data (city, country, area) stays as fields on Project.
- **DB migrations**: 001-003 applied to both prod and dev.

## Architecture Decision: API + Shared Core
- Desktop app keeps filesystem ops (folder creation, Nextcloud sync)
- New `e-fees-core` Rust crate shares domain types between desktop and API
- New `e-fees-api` standalone axum service for programmatic data access
- API starts read-only, grows toward write parity
- Standalone `.env` for API container (populated from credential system)
- If OpenCloud replaces Nextcloud, even folder ops could migrate to API

## Folder Mapping
- `01 RFPs` → Lead, RFP, Submitted
- `11 Current` → Awarded, Design, Construction
- `99 Completed` → Completed, Superseded
- `00 Inactive` → Lost, No Response, Cancelled, On Hold

## Critical Rules
1. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
2. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
3. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
4. **Fixed px values**: Desktop app with OS-level scaling, never use rem
5. **Process safety**: NEVER pkill without permission
6. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
7. **Releases**: ALWAYS background haiku agent. Never interactive polling.
8. **Test DB**: Must match the installed app's DB (10.0.23.11).
9. **Production safety**: Tests MUST refuse to run against 10.0.23.11.

---
*Updated: 2026-02-28*

# E-Fees Project Handover

## Current Status
v0.13.12 on main. API v0.3.0 deployed with 4 new endpoints. SurrealDB migrated to surrealkv.

- **Version**: 0.13.12 (desktop app), 0.3.0 (API), 0.2.0 (Scope)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 18 core, 89 desktop (1 pre-existing xlsx fail), 21 API unit, 72 API integration
- **API**: e-fees-api at 10.0.21.80:3200 (v0.3.0, 27 operations)
- **Scope**: e-fees-scope at 10.0.21.81:3201 (v0.2.0)

## Last Session
**Date**: 2026-03-21
**Summary**: API v0.3.0 — 4 new endpoints for PA integration, shared export logic extraction, SurrealDB surrealkv migration verification, DB schema migration re-applied.

### Work Completed
1. **Stale branch cleanup** — 6 remote + 3 local merged branches deleted
2. **SurrealDB surrealkv verification** — confirmed app connects, 72/72 integration tests pass
3. **DB schema migration re-applied** — surrealkv backup was pre-v0.13.0; re-ran migration 003 (project/fee status remapping + new ASSERTs)
4. **API v0.3.0** — PR #6 (9 commits), brainstormed with PA via hub, staff-reviewed spec + plan:
   - `e-fees-core/src/export.rs` — shared build_fee_json, format_issue_date, clean_record_key (18 unit tests)
   - `e-fees-api/src/ssh.rs` — SshOps helper with shell quoting (6 unit tests)
   - `POST /fees/{id}/json-export` — InDesign JSON export with archive
   - `GET /fees/{id}/json-status` — field population check (23 fields)
   - `GET /projects?search=...` — fuzzy search on name/name_short/number.id
   - `POST /projects/{id}/documents` — multipart file upload (15 unit tests)
   - Refactored folders.rs to use SshOps, added NC_BASE_PATH config
   - Desktop app updated to use shared core exports, B3 path bug fixed (name_short → name)
   - Fixed issue_date test format (YYYYMM → YYMMDD)
5. **Container deployed** — rebuilt on AI server, v0.3.0 healthy, all tests pass
6. **Wiki updated** — e-fees-api endpoints + environment sections updated for v0.3.0
7. **PA notified** — endpoints ready for validation
8. **SSH MCP removal acknowledged** — e-fees uses native SSH, no code impact
9. **Type cleanup verified** — crudTypes.ts already consolidated, no action needed

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 surrealkv (ns: emittiv, db: projects) |
| KB DB | ws://10.0.21.15:8000 v3.0.0 (ns: kb, db: knowledge) |
| Dev DB | ws://10.0.23.12:8000 (may need migration 003 re-applied) |
| API Container | 10.0.21.80:3200 v0.3.0 (br0, healthy, 27 ops) |
| Scope Container | 10.0.21.81:3201 v0.2.0 (br0, healthy) |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |
| Wiki pages | slugs: "e-fees", "e-fees-api", "e-fees-scope" |
| PR #6 | Merged — API enhancements (export, search, upload, status) |

## Deployment Notes

### Config YAML Migration (2026-03-22)
- e-fees-api and e-fees-scope now use `config.yaml` for non-secret runtime config
- Secrets (SURREAL_PASS, API_KEY) remain in `.env`
- Docker containers need config.yaml mounted: `-v /path/to/config.yaml:/app/config.yaml`
- ConfigManager hot-reloads on file change (2s polling in Docker)
- Container rebuild commands pending — needs config.yaml copied to AI server appdata dirs

## Next Steps
1. **PA validation** — PA will test full flow (company → contact → project → folder → fee → json-export)
2. **Config YAML migration** — Deferred; requires axum port of emittiv-container-utils Rust crate
3. **Evaluate Playwright Test Agents** — `--loop=claude` option for UI smoke tests
4. **Design review items** — H-2 (Scope Builder nav link), M-10 (ScopeBuilder breadcrumb)
5. **InDesign automation** — table population, scope text insertion (JSON export now API-accessible)
6. **Dev DB migration** — Verify 10.0.23.12 has migration 003 applied (status ASSERTs)

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency display
- **Shared core** (`crates/e-fees-core/`): Domain types + export logic shared between desktop, API & scope
- **Standalone API** (`e-fees-api/`): Full CRUD, search, JSON export, document upload, OpenAPI — v0.3.0
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation with LLM polish — v0.2.0
- **InDesign MCP** (local): UXP bridge for Claude Code <-> InDesign DOM

## Critical Rules
1. **SUPERPOWERS SKILLS MANDATORY**: Invoke relevant skill BEFORE any work
2. **TDD NON-NEGOTIABLE**: Write failing tests FIRST, then implement
3. **Always create PRs** (option 2) when finishing development branches — never ask, just do it
4. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools
5. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
6. **CSS**: Semantic `.emittiv-*` classes + `var(--color-*)` tokens, NOT utility strings > 50 chars
7. **Fixed px values**: Desktop app with OS-level scaling, never use rem
8. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases
9. **Releases**: Use /sendit (background agent). Includes cleanit quality gate.
10. **SurrealDB v3 NULL**: Omit optional fields from SET clause entirely
11. **SurrealDB SDK**: Pinned to 3.0.4 in all 4 Cargo.toml files
12. **Unraid containers**: ALL must use XML templates hosted on Forgejo
13. **Container standards**: /health, /api/health, /help, /openapi.json required for all API containers
14. **SSH**: Use native `ssh` via Bash, NOT MCP SSH server (removed workspace-wide)

---
*Updated: 2026-03-21*

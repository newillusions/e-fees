# E-Fees Project Handover

## Current Status
v0.13.4 + 4 unreleased commits. Advanced filtering, test fixes, API enhancements deployed.

- **Version**: 0.13.4 (released 2026-03-02) + 4 unreleased commits
- **Branch**: main
- **Database**: SurrealDB @ ws://10.0.21.8:8000 (emittiv/projects)
- **Tests**: 633/633 passing (frontend), 62/62 passing (API integration)
- **API**: e-fees-api v0.3.1 deployed at 10.0.21.80:3200 (Docker, br0 network)

## Last Session
**Date**: 2026-03-04
**Summary**: Committed API enhancements (status filtering, auto-numbering, multi-key auth), fixed all 44 modal test failures, redeployed API to Docker, visually verified advanced filtering across all 4 pages.

### Accomplished
- `feat(api)`: Status filtering on projects/companies/contacts, auto-numbering endpoint (`/projects/next-number`), multi-API-key auth
- `fix(tests)`: Resolved 44 modal test failures — CrudModal synchronous formData init + Svelte 5 `untrack()` in `$effect`, updated test selectors for current CSS classes
- `fix(api)`: Fuzzy country lookup (strips dots: "UAE" matches "U.A.E.")
- API redeployed v0.3.1 — all 62 integration tests passing on deployed container
- Visual verification: advanced filtering confirmed working on Projects, Companies, Contacts, Proposals pages
- Commits: `3b397da`, `22f95bf`, `2b03be3`

### Key Fix: CrudModal Svelte 5 Binding
Root cause: `formData` started as `$state({})`, `$effect` filled it after first render. Tests mounting with `isOpen=true` triggered `bind:value={undefined}` errors.
Fix: (1) Initialize `formData` synchronously with `$state(initializeFormData())`, (2) Wrap `resetForm()`/`loadEntityData()` in `untrack()` to prevent reactive loop from `fields` prop dependency.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.21.8:8000 (ns: emittiv, db: projects) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 (br0, e-fees-api:v0.3.1) |
| API .env | /mnt/user/appdata/e-fees-api/.env (on AI server) |
| API source on server | /mnt/user/appdata/e-fees-api/source/ |
| Installed app config | ~/Library/Application Support/com.emittiv.e-fees/.env |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |
| Wiki page | slug: "e-fees" (fully updated 2026-03-02) |

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency display
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop & API
- **Standalone API** (`e-fees-api/`): Full CRUD HTTP endpoints with multi-key auth, pagination, status filtering, auto-numbering, OpenAPI/Swagger docs
- API deployed as Docker container on Unraid AI server (br0 network, ipvlan L2)

## Next Steps
1. InDesign export functionality
2. Test multi-currency with real data (set client_currency + exchange_rate on a fee)
3. Consider a release to bundle recent work (filtering, test fixes, bulk ops)

## Critical Rules
1. **SUPERPOWERS SKILLS MANDATORY**: Invoke relevant skill BEFORE any work. No exceptions.
2. **TDD NON-NEGOTIABLE**: Write failing tests FIRST, then implement. Always.
3. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
4. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
5. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
6. **Fixed px values**: Desktop app with OS-level scaling, never use rem
7. **Process safety**: NEVER pkill without permission
8. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
9. **Releases**: ALWAYS background haiku agent. Never interactive polling.
10. **Test DB**: Must match the installed app's DB.
11. **SurrealDB v3 NULL**: Never bind `json!(None)` — omit optional fields from SET clause entirely.
12. **Fee issue_date**: YYYYMM format (6-digit numeric string per DB ASSERT), not ISO date.
13. **API redeploy**: Pull on AI server (`/mnt/user/appdata/e-fees-api/source/`), docker build, stop/rm/run with same env.

---
*Updated: 2026-03-04*

# E-Fees Project Handover

## Current Status
v0.13.8 released. SurrealDB SDK pinned to v3.0.4 matching production DB. All containers rebuilt and deployed.

- **Version**: 0.13.8 (released 2026-03-15)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 87/87 Rust, 633/633 frontend, 0 TS errors
- **API**: e-fees-api:v0.4.2 deployed at 10.0.21.80:3200
- **Scope**: e-fees-scope:v0.2.4 deployed at 10.0.21.81:3201

## Last Session
**Date**: 2026-03-15
**Summary**: Investigated SurrealDB v3.0.4 upgrade on production DB (10.0.23.11). Pinned all Rust SDK deps from unpinned "3.0" (resolving 3.0.1) to 3.0.4. Removed unused surrealdb.js. Rebuilt and redeployed API + Scope containers. Released v0.13.8.

### Accomplished
- **SurrealDB SDK pinned to 3.0.4** across 4 Cargo.toml files (e-fees-core, src-tauri, e-fees-api, e-fees-scope)
- **Cargo.lock updated**: surrealdb 3.0.1→3.0.4, surrealdb-core 3.0.1→3.0.4, surrealdb-types 3.0.1→3.0.4
- **Removed unused `surrealdb.js`** (v1.0.0) from package.json — frontend uses Tauri IPC only
- **API container rebuilt** (e-fees-api:v0.4.2) and deployed to 10.0.21.80
- **Scope container rebuilt** (e-fees-scope:v0.2.4) and deployed to 10.0.21.81
- **v0.13.8 released**: macOS aarch64 + x64, Windows — 8 assets on Forgejo, update.json synced to GitHub
- **KB offline diagnosed**: "KB_VERSION: offline (missing credentials)" is from stale MCP server process, not a code issue. Plugin v3.64.0 is current. Fix: restart Claude Code session.
- **v3.0.0→3.0.4 changelog researched**: search::score() fixed, MATCHES still broken, math::max([]) still -Infinity
- **Full codebase scan**: no search::score(), math::max(), or MATCHES usage — no code changes needed
- **Implementation plan**: `docs/superpowers/plans/2026-03-15-surrealdb-v304-upgrade.md`

### SurrealDB v3.0.4 Key Changes (from 3.0.0)
- `search::score()` now returns real BM25 scores (was 0.0)
- `RecordIdKeyType::Object` serialization fixed
- `SurrealValue::from_value` compatibility for all JSON variants
- Records not existing return `None` (was confusing error)
- `UPSERT SET` with `IF` expressions evaluates correctly
- **Still broken**: parameterized MATCHES (keep `escapeSurrealSearch()`), `math::max([])` returns -Infinity

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 (ns: emittiv, db: projects) |
| KB DB | ws://10.0.21.15:8000 v3.0.0 (ns: kb, db: knowledge) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 (br0, e-fees-api:v0.4.2) |
| Scope Container | 10.0.21.81:3201 (br0, e-fees-scope:v0.2.4) |
| Scope API Key | efees-scope-2026-s7k2m9xp |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |
| InDesign MCP repo | /Volumes/base/dev/indesign-uxp-server |
| NC project create script | /mnt/user/appdata/scripts/nc-project-create.sh (on Primary 10.0.20.12) |
| Wiki page | slug: "e-fees" |
| Design review tracking | `docs/plans/2026-03-13-design-review.md` |
| Colour swatch page | DevMode → Design System section |

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency display
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop, API & scope
- **Standalone API** (`e-fees-api/`): Full CRUD HTTP, auto-numbering, OpenAPI/Swagger
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation with LLM polish
- **InDesign MCP** (local): UXP bridge for Claude Code <-> InDesign DOM
- **Nextcloud sync**: Group folder on Primary, syncs to Windows clients

## Next Steps
1. **Colour token approvals** — DevMode colour swatch page shows proposed Mocha accent mappings for stat icons, status badges, error/warning/success. User needs to review and approve before tokens are defined. See `docs/plans/2026-03-13-design-review.md` → "Design System: Color Test Page"
2. **Design review fixes** — 40+ findings in `docs/plans/2026-03-13-design-review.md`. High priority: Dashboard "Active Fees"→"Active Proposals", 6× `alert()` calls, ARIA issues on FormInput/TypeaheadSelect/BaseModal, FirstRunSetup copy
3. **Assumptions clause refinement** — ongoing review (`docs/plans/assumptions-review.md`)
4. **Automate InDesign text variable population** via MCP — set all 21 variables from fee record
5. **Automate InDesign table population** — map PricingBreakdown to 5 pricing tables
6. **Scope text insertion** into InDesign — pipe scope service output to InDesign text frames
7. **Expose folder creation via API** — AI server SSH to Primary, run nc-project-create.sh

## Colour System State
- **Approved and live**: Mocha surface tokens (`--emittiv-black` Crust, `--emittiv-darker` Base, gradients use Mantle)
- **Pending approval**: Mocha accent colours for semantic roles (stat icons, status badges, states) — swatches in DevMode, do NOT tokenise until user approves individual mappings
- **Rule**: Never invent or tokenise colours autonomously. Always get user approval via swatch page first.

## Critical Rules
1. **SUPERPOWERS SKILLS MANDATORY**: Invoke relevant skill BEFORE any work. No exceptions.
2. **TDD NON-NEGOTIABLE**: Write failing tests FIRST, then implement. Always.
3. **Screenshots**: Peekaboo MCP with `app_target: "app"` — NEVER browser tools for Tauri
4. **Dev command**: `npm run tauri:dev` (not `npm run dev`)
5. **CSS**: Semantic `.emittiv-*` classes, NOT utility strings > 50 chars
6. **Fixed px values**: Desktop app with OS-level scaling, never use rem
7. **Git**: Push to Forgejo (origin) for daily work. GitHub only for tagged releases.
8. **Releases**: ALWAYS background haiku agent. Never interactive polling.
9. **SurrealDB v3 NULL**: Never bind `json!(None)` — omit optional fields from SET clause entirely.
10. **Fee issue_date**: YYYYMM format (6-digit numeric string per DB ASSERT).
11. **Scope fee queries**: OMIT id, backtick-quote keys, bind Value not String, use FLEXIBLE for nested objects.
12. **Scope integration tests**: Run with `--test-threads=1` (shared DB state).
13. **API/Scope redeploy**: rsync to AI server, docker build, stop/rm/run with same env.
14. **SurrealDB SDK**: Pinned to 3.0.4 in all 4 Cargo.toml files. Production DB is v3.0.4.

---
*Updated: 2026-03-15*

# E-Fees Project Handover

## Current Status
v0.13.12 on main. Container standards compliance complete for e-fees-api and e-fees-scope (PR #5 merged). Both containers deployed at v0.2.0 with all standard endpoints live.

- **Version**: 0.13.12 (desktop app), 0.2.0 (API + Scope containers)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 @ ws://10.0.23.11:8000 (emittiv/projects)
- **Tests**: 91 Rust, 707 frontend, 10 new container standards integration tests (pending live run)
- **API**: e-fees-api at 10.0.21.80:3200 (v0.2.0, container standards 5/6)
- **Scope**: e-fees-scope at 10.0.21.81:3201 (v0.2.0, container standards 5/6)

## Last Session
**Date**: 2026-03-21
**Summary**: Full container standards compliance — audit, plan, implement, review, deploy. PR #5 with 6 commits, reviewed by 5 parallel agents, all findings fixed.

### Work Completed
1. **Compliance audit** — Ran /container-standards --check on both containers, identified 6 gaps each
2. **Implementation plan** — 8-task plan with TDD, staff-reviewed before execution
3. **e-fees-api endpoints** — /health (uptime, checked_at, dependencies with latency), /api/health alias, /help (auto-generated from OpenAPI), /openapi.json
4. **e-fees-scope endpoints** — Same 4 endpoints, plus Ollama dependency with concurrent checks (tokio::join!)
5. **Review fixes** — DB health timeout (3s), .unwrap() → proper error handling, logging on failures, OpenAPI version sync, config_source correction, standardized status terms
6. **Deployment** — Both containers rebuilt and redeployed on AI server (10.0.20.11)
7. **Wiki** — Created e-fees-scope page, updated e-fees-api with compliance section
8. **Version bump** — Both containers 0.1.0 → 0.2.0

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 (ns: emittiv, db: projects) |
| KB DB | ws://10.0.21.15:8000 v3.0.0 (ns: kb, db: knowledge) |
| Dev DB | ws://surreal-dev.internal:8000 (10.0.23.12) |
| API Container | 10.0.21.80:3200 v0.2.0 (br0, healthy) |
| Scope Container | 10.0.21.81:3201 v0.2.0 (br0, healthy) |
| Forgejo repo | forge.mms.name/emittiv/fee-prop |
| Wiki pages | slugs: "e-fees", "e-fees-api", "e-fees-scope" |
| PR #5 | Merged — container standards compliance |

## Next Steps
1. **Run integration tests** against live deployed containers (e-fees-api + e-fees-scope)
2. **Config YAML migration** — Deferred; requires axum port of emittiv-container-utils Rust crate
3. **Evaluate Playwright Test Agents** — `--loop=claude` option for UI smoke tests
4. **Design review items** — H-2 (Scope Builder nav link), M-10 (ScopeBuilder breadcrumb)
5. **InDesign automation** — table population, scope text insertion
6. **Type cleanup from PR #4** — consolidate duplicated types to crudTypes.ts

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency display
- **Shared core** (`crates/e-fees-core/`): Domain types shared between desktop, API & scope
- **Standalone API** (`e-fees-api/`): Full CRUD HTTP, auto-numbering, OpenAPI/Swagger — v0.2.0
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

---
*Updated: 2026-03-21*

# E-Fees Project Handover

## Current Status
v0.15.0 released. Scope UI integration + scope-pricing stage linkage merged to main and deployed. All platforms (macOS aarch64/x64, Windows).

- **Version**: 0.15.0 (desktop app)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Dev Database**: SurrealDB v3.0.4 @ ws://10.0.23.12:8000

## Last Session
**Date**: 2026-03-27
**Summary**: Implemented scope UI integration (ScopeViewer component tree) and scope-pricing stage linkage (bidirectional stage sync, revision history, stage dictionary). Brainstormed design with user, staff-reviewed spec, wrote 12-task plan, executed via subagent-driven development (4 parallel batches). Fixed multiple SurrealDB v3 issues (FLEXIBLE fields, DbValue serialization, SurrealValue i64). Fixed Tauri MCP stale socket cleanup. UI-tested via Tauri MCP (after socket fix). Released v0.15.0.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 v3.0.4 (scope tables + clauses seeded) |
| API Container | 10.0.21.80:3200 (e-fees-api v0.3.0) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope v0.2.0 — NEEDS REDEPLOYMENT with new code) |
| Scope-Pricing Spec | docs/superpowers/specs/2026-03-26-scope-pricing-linkage-design.md |
| Scope-Pricing Plan | docs/superpowers/plans/2026-03-26-scope-pricing-linkage.md |
| Forgejo Release | forge.mms.name/emittiv/fee-prop/releases/tag/v0.15.0 |
| Local scope .env | e-fees-scope/.env (dev DB 10.0.23.12, Ollama 10.0.21.20) |
| Ollama | 10.0.21.20:11434 (NOT 10.0.21.50 — config.yaml updated) |

## Next Steps
1. **Deploy scope container** — rebuild e-fees-scope image with stage linkage + revision code, push to Forgejo registry, update Unraid container
2. **Stage autocomplete UI** — wire searchStageDictionary into pricing stage name input
3. **Markdown export wiring** — connect ScopeViewer handleSave to export_scope_markdown Tauri command (currently has TODO placeholders for fee_ref, project_name, project_folder)
4. **InDesign export** — pricing tables (T0-T4) via UXP scripting
5. **Advanced filtering** — date ranges, multi-field search improvements

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency, scope viewer with stage linkage
- **Shared core** (`crates/e-fees-core/`): Domain types + export logic + `set_pricing_stages()`
- **Standalone API** (`e-fees-api/`): Full CRUD, search, JSON export, OpenAPI — v0.3.0
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation with stage context, revision history
- **Container-utils** (`forge.mms.name/emittiv/container-utils`): Shared ConfigManager + health routes

## Critical Rules
1. **Tauri MCP socket**: Auto-cleans stale socket now (socket_server.rs fix). Use `rm -f /tmp/tauri-mcp.sock` if issues persist.
2. **Dev scope testing**: Run local scope service: `cd e-fees-scope && cargo run` with `.env` pointing to dev DB (10.0.23.12) and Ollama at 10.0.21.20
3. **SurrealValue i64**: Use `i64` not `Option<i64>` for fields with DEFAULT 0
4. **scope_revision.clauses**: Stored as JSON string, not native array (avoids DbValue binding issues)
5. **assembly_to_json**: Must include all new fields (stages_snapshot, current_revision) — easy to miss
6. **generate_scope upsert**: DELETE+CREATE resets all fields — must explicitly include current_revision in CREATE SET

---
*Updated: 2026-03-27*

# E-Fees Project Handover

## Current Status
v0.16.0 released. InDesign workbook export feature complete — generates 6-sheet xlsx for InDesign table linking. All tests passing (25 in e-fees-core).

- **Version**: 0.16.0 (desktop app + API)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Dev Database**: SurrealDB v3.0.4 @ ws://10.0.23.12:8000

## Last Session
**Date**: 2026-04-02
**Summary**: Context load only - no work performed. Handover current from 2026-03-31 session (v0.14.3 config diagnosis, upgrade to v0.16.0).

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 v3.0.4 |
| API Container | 10.0.21.80:3200 (e-fees-api) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope) |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| App data dir (prod) | ~/Library/Application Support/com.emittiv.e-fees/ |
| InDesign template | `src-tauri/resources/template.idml` |

## Next Steps
1. **InDesign template linking** — one-time manual setup: link each table in .indd to its xlsx sheet range
2. **Test with real fee data** — run export on an actual fee record, open in InDesign, verify tables populate correctly
3. **Frontend UI** — add "Export InDesign Workbook" button to ProposalModal/detail view
4. **Duration field** — Stage struct has no `duration` field; T0/T1 Duration columns are blank. Consider adding to schema.
5. **Distribution list** — `fee.distribution` field doesn't exist yet; T-Dist sheet deferred
6. **UXP automation** (future) — auto-update links, post-contract section removal, Save As
7. **Scope/text content** (future) — populate proposal text via scope service

## Architecture
- **Desktop app** (Tauri): Full CRUD, filesystem ops, multi-currency, scope viewer, stage autocomplete, country normalization
- **Shared core** (`crates/e-fees-core/`): Domain types + export logic (JSON + Excel + InDesign workbook)
- **Standalone API** (`e-fees-api/`): Full CRUD, search, JSON export, InDesign export, OpenAPI
- **Scope service** (`e-fees-scope/`): Clause library, corpus ingestion, scope generation
- **Container-utils** (`forge.mms.name/emittiv/container-utils`): Shared ConfigManager + health routes

## Recent Decisions
- **Excel-linked tables over IDML manipulation** — InDesign handles reflow natively, reuses existing Excel pipeline, editable by user
- **Separate sheets per InDesign table** — clean separation, predictable ranges for linking
- **Post-contract sheets empty (not omitted) when design-only** — preserves InDesign link references

## Critical Rules
1. **Config files**: `e-fees.config` (prod) and `e-fees.config.dev` (debug) — NOT `.env`
2. **dotenvy**: Uses `dotenvy::from_filename()` with debug/release selection
3. **SurrealValue i64**: Use `i64` not `Option<i64>` for fields with DEFAULT 0
4. **InDesign MCP**: Requires UXP bridge running (node bridge/server.js in indesign-uxp-server) + UXP plugin loaded in InDesign

## Pending Hub Actions (loaded 2026-05-06)
- **ailx P8** message:9a3hw83prc2gt5rqpat2 — mobilisation rebate formula bug, PaymentSchedulePanel.svelte:95-126
- **dev P5** message:fmnqez3f3wow2ugmtkv8 — container standards migration (api+scope)
- **Security** — `<redacted-rotated-2026-05-06>` password literal still in 13+ files (db rotation 2026-05-06); originally flagged via dev P7 hub msg

---
*Updated: 2026-05-06*

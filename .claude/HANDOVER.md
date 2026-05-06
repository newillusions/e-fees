# E-Fees Project Handover

## Current Status
e-fees-api v0.3.1 deployed (PATCH /projects/{id} added, 405→200; folder script restored on Primary, 503→200; pa-core RFP intake unblocked). Active password literal scrub committed (47d7bbf) ahead of 2026-05-06 rotation. Desktop app v0.16.0 unchanged.

- **Versions**: desktop 0.16.0, e-fees-api 0.3.1
- **Image**: forge.mms.name/emittiv/e-fees-api:v0.3.1 (sha256:313c3da...)
- **Branch**: main
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Dev Database**: SurrealDB v3.0.4 @ ws://10.0.23.12:8000

## Last Session
**Date**: 2026-05-06
**Summary**: Comprehensive folder API fix end-to-end (pa P7 hub msg:moox1tgdtqqoosbr033k) + password scrub (dev P7 hub msg:bo94tqxsms0jhcijrp9f). PATCH route added (axum), Dockerfile bumped to rust:1.90-slim, image rebuilt + pushed, Unraid container updated, Bermuda Beach (projects:26_97101) folder created and verified. Restored missing `nc-project-create.sh` on Primary at `/mnt/user/appdata/scripts/`. `th38ret3ch` literal scrubbed from 14 active files, worktree branches left frozen. Replied to pa, dev (×2). Three commits: afb2c4c (PATCH + orphan cleanup), 47d7bbf (security scrub), 557b8cd (Dockerfile rust 1.90).

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

## Pending Hub Actions
- **ailx P8** message:9a3hw83prc2gt5rqpat2 — mobilisation rebate formula bug, PaymentSchedulePanel.svelte:95-126 (accepted, not yet shipped)
- **dev P5** message:fmnqez3f3wow2ugmtkv8 — container standards migration (api side already on emittiv-container-utils + health endpoints; scope side outstanding)
- **dev P5** message:drk1bp2myaunm4rbo2zh — Opus 4.7 audit (review work, not started)

---
*Updated: 2026-05-06*

# E-Fees Project Handover

## Current Status
e-fees-api v0.3.1 deployed (PATCH route, folder script restored, Bermuda Beach unblocked). Desktop app v0.16.0 unchanged. Active password literal scrubbed ahead of 2026-05-06 SurrealDB rotation. Mobilisation rebate formula corrected. e-fees subscribed to `workspace-coordination` topic. Hub queue clear.

- **Versions**: desktop 0.16.0, e-fees-api 0.3.1, e-fees-scope 0.2.0
- **API image**: forge.mms.name/emittiv/e-fees-api:v0.3.1 (sha256:313c3da...)
- **Branch**: main (clean)
- **Database**: SurrealDB v3.0.4 surrealkv @ ws://10.0.23.11:8000 (emittiv/projects)
- **Dev DB**: ws://10.0.23.12:8000

## Last Session
**Date**: 2026-05-06
**Summary**: Comprehensive cleanup of all hub-pending items. Six commits shipped to forge.mms.name (afb2c4c, 47d7bbf, 557b8cd, d1d4b31, bec7409, 86763bb).

Work shipped:
1. **pa folder API fix** — added `.patch(routes::projects::update_project)` to `/projects/{id}`, restored missing `nc-project-create.sh` on Primary at `/mnt/user/appdata/scripts/`, bumped Dockerfile to rust:1.90-slim (roaring@0.11.4 dep), released e-fees-api v0.3.1, Bermuda Beach (`projects:26_97101`) folder created end-to-end.
2. **Password scrub** — `th38ret3ch` literal replaced/redacted across 14 active files; `_FROM` env-var pattern in MCP-CONFIG.md, env-var lookup in tests.rs, `<redacted-rotated-2026-05-06>` placeholder in docs. Worktree branches left frozen per dev's call.
3. **Mobilisation rebate** — extracted to `src/lib/utils/paymentSchedule.ts` with the equal-split formula. 6 vitest cases covering ailx test scenario + edge cases. PaymentSchedulePanel.svelte now calls into util.
4. **Container standards** — confirmed both api and scope already on `emittiv-container-utils` + ConfigManager + health endpoints (done during v0.13.x–v0.16.0 cycle).
5. **Opus 4.7 audit** — 1 edit: development-workflow.md Rule 1 reframed from "MUST delegate to sub-agents" to a conditional. All other NEVER/MUST entries kept (incident-backed). Subscribed to workspace-coordination topic.

Replied to all 5 hub messages. All 7 session tasks completed.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.4 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 v3.0.4 |
| API Container | 10.0.21.80:3200 (e-fees-api v0.3.1) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope v0.2.0) |
| API source on Unraid | `/mnt/user/appdata/e-fees-api/source/` (forge clone) |
| NC project script | `/mnt/user/appdata/scripts/nc-project-create.sh` (on Primary 10.0.20.12) |
| NC group folder | nextcloud-e ID 1, `/mnt/user/emittiv/nc/__groupfolders/1/` |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| Hub topics | emittiv-ecosystem, workspace-coordination |
| InDesign template | `src-tauri/resources/template.idml` |

## Next Steps
1. **Wait for SurrealDB password rotation** (today, dev orchestrator-driven). Container env will be updated by dev at rotation time. Verify desktop app reconnects on next lamp-on.
2. **InDesign template linking** — one-time manual setup: link each table in .indd to its xlsx sheet range (carry-over from prior session).
3. **Frontend UI** — add "Export InDesign Workbook" button to ProposalModal/detail view (carry-over).
4. **Test mobilisation fix in app** — open a real proposal with non-uniform stages, regenerate payment schedule, confirm CD/SD/DD amounts match the new equal-split formula.

## Recent Decisions
- **Equal-split mobilisation rebate** (per Martin via ailx hub msg): rebate distributed equally across stage count, not by value proportion. Implementation extracted to `paymentSchedule.ts` for testability.
- **Sub-agent usage reframed as conditional** (Opus 4.7 audit): delegate when >500 lines output, parallel work, or red-team review — not as a blanket "MUST".
- **Worktree branches left untouched** for password scrub — frozen branch state, picked up on next active session.

## Critical Rules
1. **Config files**: `e-fees.config` (prod) and `e-fees.config.dev` (debug) — NOT `.env`
2. **dotenvy**: Uses `dotenvy::from_filename()` with debug/release selection
3. **SurrealValue i64**: Use `i64` not `Option<i64>` for fields with DEFAULT 0
4. **InDesign MCP**: Requires UXP bridge running (node bridge/server.js in indesign-uxp-server) + UXP plugin loaded in InDesign
5. **Dockerfile rust version**: 1.90-slim (roaring@0.11.4 transitive dep needs ≥1.90)
6. **NC folder script lives on Primary**, not in repo. Source of truth: `/mnt/user/appdata/scripts/nc-project-create.sh`. If lost, see commit afb2c4c era diagnosis.

---
*Updated: 2026-05-06*

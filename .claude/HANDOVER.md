# E-Fees Project Handover

## Current Status
Two ships today: db query/binding fixes (`538942f`) and the InDesign Workbook export button (`432b05e`). Mobilisation rebate (`bec7409` from yesterday) verified live in-app via Tauri MCP — no manual eyeball needed. Hub queue clear. Versions unchanged: desktop 0.16.0, e-fees-api 0.3.1, e-fees-scope 0.2.0.

- **Versions**: desktop 0.16.0, e-fees-api 0.3.1, e-fees-scope 0.2.0
- **Branch**: main (clean) — 2 new commits
- **Database**: SurrealDB v3.0.5 surrealkv @ ws://10.0.23.11:8000 (was 3.0.4 yesterday — minor patch)
- **Dev DB**: ws://10.0.23.12:8000 (also v3.0.5)
- **Tests**: 89/89 Rust unit, 22/22 vitest for revisions+ProposalModal, svelte-check 0 errors

## Last Session
**Date**: 2026-05-07
**Summary**: Fixed two pre-existing SurrealDB query bugs surfaced when the dev build was opened, then shipped the Export InDesign Workbook button. Verified yesterday's mobilisation-rebate fix live in-app on real Rozana data.

Work shipped:
1. **`538942f` fix(db)** — `get_fees_for_project` query was `projects:$pid` (invalid v3 syntax) → `type::record('projects', $pid)`. `create_activity_log` was building a CREATE statement with `$action`/`$entity_type` params but calling `client.query()` without bindings, so SurrealDB saw all fields as NONE. Switched to `query_bind_map` with all required params bound, plus inline `field: NONE` literals for the three optional fields (`metadata`, `old_value`, `new_value`) since v3 rejects JSON null for `option<T>`. 2 new `#[ignore]`'d integration tests against dev DB (RED→GREEN). Also fixed pre-existing Root auth borrow errors in `test_prod_*` tests that were blocking test compilation.
2. **`432b05e` feat(proposals)** — Added `exportIndesignWorkbook` API wrapper in `revisions.ts`, re-exported from `$lib/api/index.ts`, and added "Exports" form section + button in ProposalModal edit mode. Backend command `export_indesign_workbook` was already shipped — this just wired the UI. 4 new vitest cases for the API wrapper. Imports directly from `$lib/api/revisions` rather than the barrel because svelte-check fails to resolve newly-added re-exports through `index.ts` (tsc and runtime are happy).
3. **Mobilisation rebate verified** — Drove dev build via Tauri MCP on real Rozana fee in dev DB (`fee:wum...` shape: CD/SD/DD with 30/30/40% split, 36k/36k/48k cells, 30% mobilisation). Generated payment schedule, read input values: Mobilisation 36k, CD 24k, SD 24k, DD 36k. Equal-split formula confirmed live (old buggy formula would have produced 25,200/25,200/33,600).

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.0.5 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 v3.0.5 |
| API Container | 10.0.21.80:3200 (e-fees-api v0.3.1) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope v0.2.0) |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| Hub topics | emittiv-ecosystem, workspace-coordination |
| Bug regression test command | `EFEES_SURREALDB_USER=martin EFEES_SURREALDB_PASS=... cargo test -p app --lib _bug -- --ignored --nocapture` |
| Tauri MCP barrel-import quirk | Direct import from `$lib/api/revisions` works; barrel `$lib/api` fails svelte-check on new exports |

## Next Steps
1. **Real-app smoke of IDW button** — open the installed prod app, edit any proposal in edit mode, click "Export InDesign Workbook" under the new "Exports" section. Should produce `<proposal_dir>/<project_number>-IDW Pricing.xlsx` and reveal in Finder. Dev build can't smoke this end-to-end because dev project folders don't exist on disk.
2. **InDesign template linking** — one-time manual setup linking each .indd table cell range to the exported xlsx. (Carry-over, manual)
3. **Pre-existing `scope.test.ts` failures** — 3 unrelated test failures in `src/lib/api/scope.test.ts` (untouched by this session). Worth investigating next session.
4. **SurrealDB password rotation** — still not announced on `workspace-coordination`. Containers and desktop still using current password. Stay watchful for next dev orchestrator hub message.

## Recent Decisions
- **NONE for `option<T>` fields, not JSON null** — when binding optional fields in CREATE statements, build query string conditionally with `field: NONE` literal vs `field: $param` binding. Matches KB pattern (obs:x3jaqd4tpktm876nme6h).
- **Direct import over barrel** when svelte-check disagrees with tsc — `import { x } from '$lib/api/revisions'` instead of `from '$lib/api'`. Bypasses a barrel-resolution quirk that doesn't fail at runtime or in tsc.
- **Verification cheap-path** for in-app fixes: seed `DELETE ME` test fee in dev DB → launch dev build (binary cached) → drive via Tauri MCP `execute_js` → read input values → cleanup. ~5-10 min end-to-end.

## Critical Rules
1. **`type::record('table', $key)` for record-id parameterization** — never `table:$key` directly (parses fine in v2, rejected in v3 with explicit error message recommending the fix).
2. **`query_bind_map` for multi-param CREATE/UPDATE** — `client.query()` ignores `$param` references, leaves them NONE. Pattern lives in `client.rs:586-640`.
3. **Tauri MCP `execute_js` returns last expression value** — wrap multi-statement code in IIFE `(() => { ... return value; })()` because top-level last-expression doesn't always serialize.
4. **`reveal_in_file_manager` works in backend** — IDW export auto-reveals the saved xlsx in Finder.

---
*Updated: 2026-05-07*

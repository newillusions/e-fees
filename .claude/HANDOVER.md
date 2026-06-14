# E-Fees Project Handover

## Current Status
**Large session 2026-06-14.** Five workstreams landed; all code committed + pushed to Forgejo `main`. Working tree clean except this handover.

- **Versions** (unchanged): desktop 0.16.0, e-fees-api 0.3.4, e-fees-scope 0.2.0.
- **Branch**: main. Today's commits: `22731a4` (partial-update fix), `80c2ee9` (smoke revert), `70f2983` (agent slim-rewrite), `38fff80` (clause regression test), `40615be` (assumptions + IDW T5 docs).
- **Dev DB moved + upgraded**: now `ws://10.0.23.12:8000` **ns `emittiv_dev`** db `projects`, on **surrealdb 3.1.4** (was 3.0.5 — fleet upgrade reached it). Prod unchanged: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (3.1.2).

### What got done
1. **Partial-update merge bug FIXED** (`obs:cno62twf3e6hmhso009f`). `update_project`/`update_company_partial` used `.merge(Struct)`, sending every `None` Option field as NONE (hard-error on SCHEMAFULL projects, silent clobber elsewhere). Now SET only provided fields via `DatabaseClient::partial_update` + `query_bind_map` (`src-tauri/src/db/client.rs`). `update_contact_partial` was already correct. 6 unit tests + 1 live integration test (passed vs dev DB). `delegate_update_merge!` macro removed. Smoke checks reverted from the full-field workaround to genuine partial updates + preservation guards.
2. **5 specialist agents slimmed** `.claude/agents/` 3,678 → 207 lines (project-specific only; stale refs purged). Frontmatter preserved → auto-delegation unchanged.
3. **Phase 7 (Option B) DONE**. Migrated dev `ns:emittiv` → `ns:emittiv_dev` on 10.0.23.12 via `surreal export`/`import` (full schema+data: 13 tables, 5 fns, 60 projects). Repointed `.env` (gitignored, root) + app-data `~/Library/Application Support/com.emittiv.e-fees/e-fees.config.dev` to `ns:emittiv_dev`. Verified app root-user `martin` auth + `fn::resolve_country`. Old `ns:emittiv` on 10.0.23.12 left as backup. `obs:870h78ci4vmwyk54g1ng`.
4. **e-fees-scope `GET /clauses` 500 FIXED** (`obs:shv22cw4tewo06xhtjmu`). 21 seeded clause rows had ISO **string** `created_at`/`updated_at`; model expects `Datetime`. Data migration on prod (+ dev): `UPDATE clause SET created_at=<datetime>created_at, updated_at=<datetime>updated_at;` → 200. No code/redeploy (model was already correct). Regression test added (`e-fees-scope/tests/clause_tests.rs::test_list_clauses_unfiltered_…`). Cleaned 25 archived `DELETE ME` clauses (prod+dev); full table backed up to `~/efees-clause-table-backup-2026-06-14.json`.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) — efees_app creds |
| Dev DB | ws://10.0.23.12:8000 **v3.1.4**, **ns emittiv_dev** db projects. App connects as root user `martin` (EFEES_SURREALDB_ROOT_*; efees_app NOT provisioned here — 401). |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) — `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) — scope key `efees-scope-2026-s7k2m9xp`; clause DB = prod 10.0.23.11 ns emittiv |
| Forgejo | forge.mms.name/emittiv/fee-prop (origin; GitHub = CI/tagged only) |
| Clause table backup | ~/efees-clause-table-backup-2026-06-14.json (46 rows, pre-cleanup) |

## Next Steps (awaiting Martin)
1. **Lulu 26-97104** — issue is **waiting on a client meeting** to clarify scope + lock a viable price. DB records verified clean (project RFP, fee Draft 250k AED). At/after the meeting: decide the **Acoustics 55k** modelling — must be a **discipline line** (flows into T2), NOT a reimbursable (`obs:ve11z833mnxb9p8yiqsf`); then regenerate the proposal docs against the locked price.
2. **Assumptions clause** — gap analysis done (`docs/plans/assumptions-review.md`): 5 genuine-gap assumptions shortlisted. Martin to select the final set, decide partials 5/6/15/16 (verify clause body text), and whether to make them toggleable in scope-assembly.
3. **Extended colour palette** — RESOLVED/stale: accent tokens already live in `app.css`. Martin to confirm the shipped mappings are intended or flag any to revisit. (`memory/project_color_system.md` updated.)
4. **IDW T5 `.indd` linking** — scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. One-time InDesign **Place** of the (already-generated) T5 Reimbursable Costs sheet into story 25 after T4. Execute manually or via UXP MCP with InDesign open.

## Open follow-ups (dev tasks)
- Make the e-fees-scope integration-test cleanup **hard-delete** clauses (currently soft-deletes → archived residue accumulates).
- Drop the old `ns:emittiv` on 10.0.23.12 once confirmed unneeded (dev/unraid-ops owns; left as backup).
- Restart any running dev build to pick up the `ns:emittiv_dev` config.

## Notes
- Critical query/SurrealValue patterns live in CLAUDE.md §Critical query patterns.
- `kb_detect_project_tags` still clobbers monorepo tags — do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.

---
*Updated: 2026-06-14*

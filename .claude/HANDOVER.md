# E-Fees Project Handover

## Current Status
Two PRs open, both ready for orchestrator merge:
- **#34** (2026-08-14): update-dialog version display fix + release-manifest CI atomic-commit fix.
- **#35** (2026-08-18, NEW this session): project-level **merge** and **cascade-delete** - recover from a duplicate/mistaken project (e.g. one created in error from PA RFP intake) without orphaning fee proposals or leaving the previously-rejected "undo an RFP log" idea half-built.

- **Versions**: desktop **0.18.1**, e-fees-api 0.3.4, e-fees-scope 0.2.0 (unchanged this session).
- **`main` (origin, Forgejo)** is current source of truth at `f56bf6e`. PR #35 branch `feat/project-merge-cascade-delete`, head `d25275ac6365191ab3fb7bd02db6eae07eb3eaf9`, mergeable:true.
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2).

## Last Session (2026-08-18)
**Summary**: Added project merge (fold a source project's fee proposals onto a target, transactional, revision-collision-safe) and cascade delete (refuses unless dependent fees are explicitly confirmed via `cascade: true`) at the project level. Backend: new `src-tauri/src/db/project_lifecycle.rs` module (`preview_project_merge`, `merge_projects`, `preview_project_delete`, `delete_project_cascade` Tauri commands), transactional (`BEGIN/COMMIT TRANSACTION`), re-reads every affected record after commit to verify the state matches the plan rather than trusting the mutation's return value. A fee's record id/`number` are never renamed (they embed the *original* project number by existing `create_fee` convention) - only `project_id` and, on a revision-number collision, `rev` - which is exactly what lets merge skip e-fees-scope's `scope_assembly`/`scope_revision` tables entirely. Frontend: new `MergeProjectModal.svelte` (target-project typeahead + server preview + confirm) and two new `ProjectDetail.svelte` actions.

Full battery green: `cargo test -p app --lib` 114/114 passing (+10 ignored, incl. 2 new live-DB tests not run this session - no dev-DB credential path available that doesn't violate the workspace's "never read creds.env" rule), `cargo clippy` clean (verified zero new warnings via before/after `git stash`), `npm run test:run` 796/796 (+8 new API-layer tests), `svelte-check` 0 errors (185 pre-existing warnings unchanged), `npm run lint` 0 new errors, `npm run build` succeeds.

**Two things flagged unverified, not silently assumed correct** (see PR #35 body + code comments):
1. `activity_log.action` gained a new `'merge'` value client-side; whether the live SCHEMAFULL `activity_log` table's ASSERT permits it is unverified (no `schema.surql` for that table exists in this repo). Low blast radius - `logActivity()` is fire-and-forget (catches, warns, never throws), so a rejected write only drops the audit-log entry, not the merge itself.
2. Found but did NOT fix (unrelated, out of dispatch scope): `ProjectModal.svelte`'s existing single-project delete button computes its id via `getEntityId(project)` → `extractSurrealId(project.id)`, which for a string-typed `project.id` (confirmed via `stores.test.ts` fixtures) returns the FULL `"projects:xxx"` string rather than the bare key `delete_project`'s Rust command expects. My new merge/delete-cascade frontend code deliberately uses `extractIdFromRelation` throughout instead. Worth a look next session - if real, the existing single-project delete may be silently deleting nothing.

**Process note**: worked directly in the project root per dispatch instructions (no isolated worktree); nothing to tear down.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) - `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) - clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| PR #34 (version display + CI fix) | https://forge.mms.name/emittiv/fee-prop/pulls/34 |
| PR #35 (project merge + cascade-delete) | https://forge.mms.name/emittiv/fee-prop/pulls/35 |
| KB obs (this session) | observation:dhhaj1a090ia7wcswuwi |
| KB wiki section (this session) | wiki_section:9yrv1l4tzme1l48lsnyh (page `e-fees`) |

## Next Steps
1. **Orchestrator: merge PR #34 and PR #35**, then deploy per the normal release/deploy path. (as of 2026-08-18)
2. **If dev-DB creds are in hand**: run the 2 new `#[ignore]`-gated live tests (`test_merge_projects_reparents_fees_and_resolves_rev_collision`, `test_delete_project_cascade_removes_dependent_fees` in `db/tests.rs`) and confirm `activity_log.action='merge'` is accepted by the live schema. (as of 2026-08-18)
3. **Clause-library backlog** - fix 4 divergent clauses, supplement 4 thin, add 7 gap clauses. Gate is Martin's business-content review of 3 client-facing wording changes, not a technical blocker. (as of 2026-08-12)
4. **IDW T5 `.indd` linking** - scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. (as of 2026-08-12)
5. **Stage 3 clause-usage mining** - corpus-ranked clause suggestions are operational; remaining is one verified real-proposal run end to end. (as of 2026-08-12)
6. **Lulu 26-97104** - waiting on client meeting to lock price; then model Acoustics 55k as a discipline line + regenerate docs. (as of 2026-08-12)

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete → archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.
- Investigate the possible `ProjectModal.svelte` single-delete id bug noted above.

## Notes
- `kb_detect_project_tags` clobbers monorepo tags - do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project consumer notes) lives in `.claude/rules/judgment.md`.
- A fee's record id/`number` embed the *original* project number at creation and are never renamed by any code path - this is now a load-bearing convention for `project_lifecycle.rs`'s merge, worth knowing before touching fee-id handling elsewhere.

---
*Updated: 2026-08-18 (project merge + cascade-delete session, PR #35)*

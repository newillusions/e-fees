# E-Fees Project Handover

## Current Status
Three PRs open, all ready for orchestrator merge (PR #35 already confirmed merged):
- **#34** (2026-08-14): update-dialog version display fix + release-manifest CI atomic-commit fix.
- **#35** (2026-08-18): project-level **merge** and **cascade-delete** - **CONFIRMED MERGED** to `main` as `c187903` this session.
- **#36** (2026-08-18, NEW this session): **on-disk folder reconcile** - optional follow-up to #35, chained onto the merge success path + an opt-in cascade-delete checkbox.

- **Versions**: desktop **0.18.1**, e-fees-api 0.3.4, e-fees-scope 0.2.0 (unchanged this session).
- **`main` (origin, Forgejo)** is now at `c187903` (PR #35 merged) plus PR #36's branch on top. PR #36 branch `feat/folder-reconcile-modal`, head `d30a9cb26cae831a3b2115cede0103d95cc47687`, mergeable:true, merge_base = base (clean).
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2).

## Last Session (2026-08-18)
**Summary**: Built the on-disk folder reconcile feature per the approved design (obs:v74ffyd1v6n1go8kf4c7), following PR #35's DB-only project merge/cascade-delete. New backend module `src-tauri/src/commands/folder_reconcile.rs` - 4 Tauri commands: `preview_folder_reconcile`/`execute_folder_reconcile` (walk both project folders, classify every file NEW/CONFLICT/IDENTICAL size-first with a SHA-256 hash only on a size match, then apply per-file copy/overwrite/keep-both/skip resolutions with a mandatory `dry_run: true` pass before any real write) and `preview_trash_project_folder`/`execute_trash_project_folder` (move a deleted project's folder to `.reconcile-backups/{timestamp}/`, opt-in only). Safety invariants: never a silent overwrite (keep-both renames the incoming file; overwrite backs up the existing file first); backups always land outside the 4 status directories; every write re-validates against current reality at execute time rather than trusting a stale preview. Reused `folder_management.rs`'s folder-location/move helpers (widened to `pub(crate)`) instead of duplicating them.

Frontend: new `FolderReconcileModal.svelte` (4-step: resolve both folders by number -> classify -> per-file/bulk resolution -> dry-run summary + confirm), wired as an optional Step 5 on `MergeProjectModal.svelte`'s post-merge success path. `ProjectDetail.svelte` gained an opt-in ("default OFF") "also move the on-disk folder" checkbox on the cascade-delete confirmation, only shown when a folder actually exists; `WarningModal.svelte` extended with an optional checkbox slot (backward compatible - gated to the delete-confirm dialog only).

Full battery green: `cargo test -p app --lib` 130/130 (114 baseline + 16 new), `cargo clippy` 84/84 warnings zero new (verified via `git stash -u` - NOT plain `git stash`, which misses untracked files and would have silently included the new module in the "before" baseline), `npm run test:run` 805/805 (796 baseline + 9 new API-layer tests), `svelte-check` 0 errors / 185 warnings (baseline unchanged), `npm run lint` 14 pre-existing errors in untouched files only, `npm run build` succeeds.

**Tooling gotcha worth knowing**: svelte-check (4.2.1) intermittently failed to resolve the four new functions when barrel-re-exported through `src/lib/api/index.ts` ("no exported member", while plain `tsc --noEmit` found zero errors in the same files). Worked around by importing directly from the leaf module (`$lib/api/folderReconcile`) in the two consuming `.svelte` files - which also matches this codebase's existing precedent (`folderManagement.ts` functions were never barrel-exported either). The barrel re-export block is still in `index.ts` (harmless) but nothing relies on it.

**Process note**: worked directly in the project root per the prior session's convention (no isolated worktree); nothing to tear down.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) - `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) - clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| PR #34 (version display + CI fix) | https://forge.mms.name/emittiv/fee-prop/pulls/34 |
| PR #35 (project merge + cascade-delete) | MERGED - https://forge.mms.name/emittiv/fee-prop/pulls/35 |
| PR #36 (on-disk folder reconcile) | https://forge.mms.name/emittiv/fee-prop/pulls/36 |
| KB obs (this session) | observation:wdgfn20acnhnjlptzejn |
| KB wiki section (updated this session) | wiki_section:9yrv1l4tzme1l48lsnyh (page `e-fees`) |

## Next Steps
1. **Orchestrator: merge PR #34 and PR #36**, then deploy per the normal release/deploy path. (as of 2026-08-18)
2. **If dev-DB creds are in hand**: run PR #35's 2 `#[ignore]`-gated live tests (`test_merge_projects_reparents_fees_and_resolves_rev_collision`, `test_delete_project_cascade_removes_dependent_fees` in `db/tests.rs`) and confirm `activity_log.action='merge'` is accepted by the live schema. (as of 2026-08-18)
3. **Not built, flagged in the design**: a manual "clear .reconcile-backups older than 30 days" action + surfacing the backup dir's size in the UI (Martin's decision: keep, no auto-prune, add a manual clear action). (as of 2026-08-18)
4. **Clause-library backlog** - fix 4 divergent clauses, supplement 4 thin, add 7 gap clauses. Gate is Martin's business-content review of 3 client-facing wording changes, not a technical blocker. (as of 2026-08-12)
5. **IDW T5 `.indd` linking** - scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. (as of 2026-08-12)
6. **Stage 3 clause-usage mining** - corpus-ranked clause suggestions are operational; remaining is one verified real-proposal run end to end. (as of 2026-08-12)
7. **Lulu 26-97104** - waiting on client meeting to lock price; then model Acoustics 55k as a discipline line + regenerate docs. (as of 2026-08-12)

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete → archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.
- Investigate the possible `ProjectModal.svelte` single-delete id bug (flagged in the PR #35 session, not yet re-checked): `getEntityId(project)` → `extractSurrealId(project.id)` may return the full `"projects:xxx"` string rather than the bare key `delete_project`'s Rust command expects.

## Notes
- `kb_detect_project_tags` clobbers monorepo tags - do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project consumer notes) lives in `.claude/rules/judgment.md`.
- A fee's record id/`number` embed the *original* project number at creation and are never renamed by any code path - load-bearing for `project_lifecycle.rs`'s merge.
- `get_project_folder_location`/`find_project_folder` resolve a project's on-disk folder by scanning folder NAMES against the 4 status dirs - DB-independent, so it still works for a project whose DB record was just deleted (as long as the folder itself hasn't been touched). This is what makes the folder-reconcile step safe to run after the DB merge has already committed.

---
*Updated: 2026-08-18 (folder-reconcile session, PR #36; confirmed PR #35 merged)*

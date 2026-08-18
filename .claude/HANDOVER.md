# E-Fees Project Handover

## Current Status
Three PRs open, all ready for orchestrator merge (PR #35 already confirmed merged):
- **#34** (2026-08-14): update-dialog version display fix + release-manifest CI atomic-commit fix.
- **#35** (2026-08-18): project-level **merge** and **cascade-delete** - **CONFIRMED MERGED** to `main` as `c187903` this session.
- **#36** (2026-08-18, NEW this session): **on-disk folder reconcile** - optional follow-up to #35, chained onto the merge success path + an opt-in cascade-delete checkbox.

- **Versions**: desktop **0.18.1**, e-fees-api 0.3.4, e-fees-scope 0.2.0 (unchanged this session).
- **`main` (origin, Forgejo)** is now at `c19c8d2` (PR #35 merged + handover sync) plus PR #36's branch on top. PR #36 branch `feat/folder-reconcile-modal`, head `de9c5e0a7512a40cf81fc816694314a50f220988` (UPDATED after review - see below), mergeable:true.
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2).

## Last Session (2026-08-18)
**Summary**: Built the on-disk folder reconcile feature per the approved design (obs:v74ffyd1v6n1go8kf4c7), following PR #35's DB-only project merge/cascade-delete. New backend module `src-tauri/src/commands/folder_reconcile.rs` - 4 Tauri commands: `preview_folder_reconcile`/`execute_folder_reconcile` (walk both project folders, classify every file NEW/CONFLICT/IDENTICAL size-first with a SHA-256 hash only on a size match, then apply per-file copy/overwrite/keep-both/skip resolutions with a mandatory `dry_run: true` pass before any real write) and `preview_trash_project_folder`/`execute_trash_project_folder` (move a deleted project's folder to `.reconcile-backups/{timestamp}/`, opt-in only). Safety invariants: never a silent overwrite (keep-both renames the incoming file; overwrite backs up the existing file first); backups always land outside the 4 status directories; every write re-validates against current reality at execute time rather than trusting a stale preview. Reused `folder_management.rs`'s folder-location/move helpers (widened to `pub(crate)`) instead of duplicating them.

Frontend: new `FolderReconcileModal.svelte` (4-step: resolve both folders by number -> classify -> per-file/bulk resolution -> dry-run summary + confirm), wired as an optional Step 5 on `MergeProjectModal.svelte`'s post-merge success path. `ProjectDetail.svelte` gained an opt-in ("default OFF") "also move the on-disk folder" checkbox on the cascade-delete confirmation, only shown when a folder actually exists; `WarningModal.svelte` extended with an optional checkbox slot (backward compatible - gated to the delete-confirm dialog only).

**Review round 2, same session**: Martin caught a real gap - project folders are created from a template whose files get the project's NUMBER substituted into their names (`rename_template_files_cross_platform` in `template_ops.rs`, confirmed by reading the actual creation code). Path-equality pairing reported a renamed base file (e.g. `"26-97110-var.json"` vs `"26-97104-var.json"` - same logical file, different literal name) as two unrelated NEW files instead of one real conflict, and a plain copy would leave the source's number sitting in the target folder. Fixed with `canonical_dest_relpath()` - reverses the number substitution per path component to compute the correct destination, used for both classification AND the actual write. Plain user files (no number in the name) are unaffected. 9 new tests (5 pure-function, 4 end-to-end) cover it. PR #36 updated in place on the same branch (head `de9c5e0a7`).

Full battery green (final state): `cargo test -p app --lib` **139/139** (114 baseline + 25 folder-reconcile), `cargo clippy` 84/84 warnings zero new (verified via `git stash -u` - NOT plain `git stash`, which misses untracked files), `npm run test:run` **806/806** (796 baseline + 10 API-layer tests), `svelte-check` 0 errors / 185 warnings (baseline unchanged), `npm run lint` 14 pre-existing errors in untouched files only, `npm run build` succeeds.

**Tooling gotcha worth knowing**: svelte-check (4.2.1) intermittently failed to resolve new functions when barrel-re-exported through `src/lib/api/index.ts` ("no exported member", while plain `tsc --noEmit` found zero errors in the same files). Worked around by importing directly from the leaf module (`$lib/api/folderReconcile`) in the two consuming `.svelte` files - which also matches this codebase's existing precedent (`folderManagement.ts` functions were never barrel-exported either). The barrel re-export block is still in `index.ts` (harmless) but nothing relies on it.

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
| KB obs (this session) | observation:vntsnt3o3mo9gnuas2ge (supersedes observation:wdgfn20acnhnjlptzejn) |
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
*Updated: 2026-08-18 (folder-reconcile session, PR #36 incl. review-round-2 project-number-aware file pairing fix; confirmed PR #35 merged)*

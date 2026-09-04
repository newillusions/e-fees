# E-Fees Project Handover

## Current Status
**Corrected 2026-09-04 (full project review)**: the version of this file that was sitting uncommitted in the main checkout claimed PRs #41/#43/#44/#45/#46 were "all open, orchestrator-owned merge + prod apply" - that is **false**. Verified this session via the Forgejo API and a direct read-only query against prod: all five PRs are **merged** (into `main` between 2026-08-19 and 2026-08-29), and prod's `app_state.schema_version = "7"` (updated 2026-08-29T08:19Z, matching PR #46's merge), with 55 of 59 fee rows carrying seeded revision history (only 3 rows still show `rev=0`/`revisions=[]`, matching the mission record's "4 need manual revision numbers" note). **Historical pricing backfill (P0/P1/P2), fee revisions, and the win-ratio report are all live on prod.** Do not trust the previous version of this section.

**Now (2026-09-04)**: one PR is genuinely open - **PR #47** (`feat/fp-template-integration`, head `c8062c1`, mergeable), built in worktree `e-fees-fp-wt` by a concurrently-running agent (`gtm-fp-efees-integration`). Per the KB mission record (updated today 18:36Z by that session): the fp-template rendering path is proven end-to-end on a dev fee (14-page HTML -> 1920x1080pt PDF), code-only (not merged), with two open items - two narrative fields (reference-documents, areas) that e-fees has no data field for yet, and an owner decision on where the renderer runs (the e-fees-api container has no python3/Chrome/template checkout; the render route 503s until `FP_TEMPLATE_ROOT` is set). **This review did not touch that worktree or branch** - see `~/.claude/state/returns/review-e-fees.md` for what else is in flight.

- **Versions**: desktop **0.18.1**, e-fees-api 0.3.4, e-fees-scope 0.2.0 (confirmed via `package.json`/`tauri.conf.json`, unverified whether the two service versions changed since 2026-08-02).
- **`main` (origin, Forgejo)** is at `d769f33` (PR #46 merge). The local `main` branch in the main checkout is 11 commits behind this - has not been updated since `3e1df4a` (a lamp-off sync commit); needs a `git pull`/fast-forward next time someone works from it directly.
- **Prod DB**: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2) - schema_version 7, 75 projects / 59 fees / 31 companies / 34 contacts (queried live 2026-09-04). **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4).
- **Forgejo is on PostgreSQL** (`postgresql17-forgejo`, 10.0.23.27:5432, migrated 2026-08-20).

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects), schema_version 7 |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) - `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) - clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop (PostgreSQL as of 2026-08-20) |
| PR #47 (fp-template integration, IN PROGRESS elsewhere) | https://forge.mms.name/emittiv/fee-prop/pulls/47 (open, mergeable - do not touch, another agent's worktree) |
| PR #46 (fee revisions real) | MERGED d769f33, 2026-08-29 |
| PR #45 (win-ratio report) | MERGED 0775539, 2026-08-23 |
| PR #41/#42/#43/#44 (historical backfill P0-P2) | all MERGED, 2026-08-19/20 |
| KB mission record | `project:6z8cqd43k5jnvz0neu4j` (kept current by the fp-template session; this review only added the docs-hygiene next step) |

## Next Steps
1. **Fast-forward the main checkout's local `main` branch** (currently 11 commits behind `origin/main`) - `git -C /Volumes/base/dev/claude/e-fees checkout main && git pull`. (as of 2026-09-04)
2. **Orchestrator: merge PR #47** once the fp-template integration session finishes (owner decision needed first on where the renderer runs - see mission record `efees-scope-assembly-ui-sequencing`). (as of 2026-09-04)
3. **Follow-up owed by e-fees**: fix the backticked wildcard statement in `v007_fee_revisions.surql` flagged by the migration guard hub message - not verified fixed this session, check before the next migration touches that file. (as of 2026-08-29, unverified whether closed)
4. **Genuine document conflicts need Martin's input** (not code-fixable): 23-97102 Wynn, 25-97105 Shanghai Tang v2, 22-96603 RUA SB5. (as of 2026-08-22, unverified whether resolved since)
5. **ENTTEC Warehouse pro-bono row** - owner ruling received (Won, $0 fee, deliberate); needs the real ENTTEC contact mined from mail before the company+contact+fee rows can be created. (as of 2026-08-22)
6. **Clause-library backlog** - fix 4 divergent clauses, supplement 4 thin, add 7 gap clauses. Gate is Martin's business-content review, not a technical blocker. (as of 2026-08-18)
7. **IDW T5 `.indd` linking** - scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. (as of 2026-08-18)
8. **Stage 3 clause-usage mining** - `mine_clause_usage` binary shipped (PR #18); no evidence it has been run against the production corpus yet. (as of 2026-08-18)
9. **Lulu 26-97104** - waiting on client meeting to lock price; then model Acoustics 55k as a discipline line + regenerate docs. (as of 2026-08-18)
10. **26-96801 fee create** - AED 120k, pending Martin's one-liner per the mission record's current_focus. (as of 2026-08-29)

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete -> archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.
- Investigate the possible `ProjectModal.svelte` single-delete id bug: `getEntityId(project)` -> `extractSurrealId(project.id)` may return the full `"projects:xxx"` string rather than the bare key `delete_project`'s Rust command expects. (still unverified whether it's a real bug)

## Notes
- `kb_detect_project_tags` clobbers monorepo tags - do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project consumer notes) lives in `.claude/rules/judgment.md`.
- `fee` table is SCHEMALESS - arbitrary `data_provenance` sub-fields need no migration, unlike `projects` (SCHEMAFULL).
- New alias-map pattern for company matching: `CLIENT_ALIASES` in `p1_index_load.rs`.
- CI (`.forgejo/workflows/test.yml`) deliberately excludes `src-tauri` (needs the full Tauri/webkit2gtk apt chain the shared runner can't reliably complete) - `cargo check -p app` is the manual equivalent when touching Tauri command wiring.
- **HANDOVER discipline**: write this file only from inside a worktree, never the main checkout - a prior session's uncommitted main-checkout edit (containing stale "PRs still open" claims) sat unpushed for days and nearly got relayed as current status. If `git -C <root> status --short` shows `.claude/HANDOVER.md` modified in the main checkout, that is a signal something went wrong, not a file to trust.

---
*Updated: 2026-09-04 (full project review - corrected stale "PRs open" claims; verified prod migration/backfill state live)*

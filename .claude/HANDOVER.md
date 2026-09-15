# E-Fees Project Handover

## Current Status (as of 2026-09-15)

**Verified this session** (dispatched review-followups agent, acting on an
external code review Martin accepted 2026-09-15) via the Forgejo API and
`git log` against `origin/main` - do not trust the previous version of this
section, which was 11 days stale and had drifted from reality in one place
(see the mission-record correction below).

- **`main` (origin, Forgejo) is at `c7f2164`** (merge of PR #54, 2026-09-13).
  **0 open PRs** existed at that point - every PR from #47 through #54 is
  merged (list + dates below). This session then opened three new PRs acting
  on the review (security fixes, dead-E2E cleanup, this doc fix) - see "This
  session's PRs" below; those are correctly *not* counted in the "0 open"
  baseline above, since that describes the state the review found.
- **Release status**: desktop app is at **v0.18.1, released 2026-08-13**
  (tag commit `47296d5`). Every one of PRs #47-#54 landed on `main` *after*
  that release - `main` currently carries the docbuilder/gotenberg/fp-template
  rendering pipeline (a real feature addition), the payment-terms wording fix,
  PUID/PGID container support, and the CI hardening migration, none of which
  has shipped in a tagged release yet. **Whether to cut 0.18.2 is an owner
  decision**, not something to infer from "main looks ready" - ask Martin
  before running `/release`.
- **Versions**: desktop `0.18.1` (package.json/tauri.conf.json/Cargo.toml, all
  consistent), e-fees-api `0.3.4`, e-fees-scope `0.2.0` (unchanged this
  session).

### Merged PRs #47-#54 (all confirmed via `mcp__forgejo__get_pull_request` / `git log`)

| PR | Merged | Commit | Title |
|----|--------|--------|-------|
| #47 | 2026-09-04 | `a6671d5` | feat(export): fp-template integration - fee record to gated proposal PDF |
| #48 | 2026-09-04 | `102c4bc` | docs: correct stale PR/prod-status claims found during full project review |
| #49 | 2026-09-06 | `edad29a` | feat(export): gotenberg render backend with contained asset flattening, gates before render |
| #50 | 2026-09-06 | `8ecb9fa` | feat(export): docbuilder client for production proposal rendering |
| #51 | 2026-09-12 | `d695468` | docs(terms): invoices submitted with each stage's deliverables (owner ruling 2026-09-09) |
| #52 | 2026-09-09 | `114b1c6` | chore(fp-template): bump pinned ref to 42ac7c84 (payment-terms fix) |
| #53 | 2026-09-12 | `3a2aca8` | feat(docker): PUID/PGID support for e-fees-api and e-fees-scope with SSH-key copy-out |
| #54 | 2026-09-13 | `c7f2164` | ci: migrate to Forgejo Actions hardening standard (13.1) |

**Mission-record correction made this session**: the KB mission record's
`payment-terms-invoice-on-deliverable` next-step asserted "PR #51 stays open,
unmerged, as the doc/test record." That was wrong - PR #51 merged
2026-09-12T17:56:23+04 (`d695468`), confirmed by a direct
`mcp__forgejo__get_pull_request` fetch, not just the list endpoint. Corrected
in the mission record as part of this session's `kb_status_update`.

### This session's PRs (review follow-ups, not yet merged as this is written)

| PR | Branch | What |
|----|--------|------|
| #55 | `fix/security-pass` | Removed a hardcoded scope-service API key + IPs that had been baked into every release build and were live on the public GitHub mirror; fixed SQL-injection-shaped identifier interpolation in e-fees-api and e-fees-scope; restricted CORS from `Any`; constant-time API-key compare; default-deny gate on the desktop agent server's LAN-exposed-without-a-key case. |
| #56 | `fix/dead-e2e-cleanup` | Removed the dead Playwright `e2e/` stack (`playwright.config.ts` intentionally threw on load - browser automation cannot drive a Tauri webview) and its 5 guaranteed-red npm scripts; `e2e-mcp/` (Tauri MCP) is the only E2E path that has ever worked and is now the sole documented one; reconciled 3 divergent config-example files down to the one (`e-fees.config.template`) that's actually kept in sync with `settings.rs`. |
| #57 | `fix/handover-docs-sync` | This file. |

All three branch independently from `c7f2164` (siblings, not stacked) so they
can merge in any order without rebasing on each other.

## Key Context

| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns `emittiv`, db `projects`) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4 (ns `emittiv_dev`, db `projects`) |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) - `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) - clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop (PostgreSQL, migrated 2026-08-20) |
| GitHub mirror | github.com/newillusions/e-fees - **PUBLIC** (verified `gh repo view` this session); release-tag path only, but full git history syncs there - see PR #55 for why that mattered |
| KB mission record | `project:6z8cqd43k5jnvz0neu4j` |

## Next Steps

1. **Orchestrator: review + merge PR #55, #56, #57** (this session's review
   follow-ups) - none deploy anything; PR-only per dispatch contract. (as of
   2026-09-15)
2. **Owner-gated**: the scope-service API key removed in PR #55 was exposed
   on the public GitHub mirror - rotation was explicitly out of scope for
   that PR and needs Martin's call. (as of 2026-09-15)
3. **Follow-up flagged, not fixed, in PR #56**: `test:webdriver` /
   `wdio.conf.js` is a third, separate E2E mechanism not covered by the
   review or by CLAUDE.md's testing directive - worth a look, not blocking.
   (as of 2026-09-15)
4. **0.18.2 release decision** - main carries unreleased features (see
   Current Status above); ask Martin before running `/release`. (as of
   2026-09-15)
5. **Genuine document conflicts need Martin's input** (not code-fixable):
   23-97102 Wynn, 25-97105 Shanghai Tang v2, 22-96603 RUA SB5 - carried
   forward from 2026-08-22, not reverified this session.
6. **Clause-library backlog** - fix 4 divergent clauses, supplement 4 thin,
   add 7 gap clauses. Gate is Martin's business-content review, not a
   technical blocker. (as of 2026-08-18, not reverified this session)
7. **IDW T5 `.indd` linking** - scoped in
   `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. (as of 2026-08-18,
   not reverified this session)

## Open Follow-ups

- Make e-fees-scope integration-test cleanup hard-delete (currently
  soft-delete -> archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.
- Investigate the possible `ProjectModal.svelte` single-delete id bug:
  `getEntityId(project)` -> `extractSurrealId(project.id)` may return the
  full `"projects:xxx"` string rather than the bare key `delete_project`'s
  Rust command expects. (still unverified whether it's a real bug)

## Notes

- `kb_detect_project_tags` clobbers monorepo tags - do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not
  `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project
  consumer notes) lives in `.claude/rules/judgment.md`.
- `fee` table is SCHEMALESS - arbitrary `data_provenance` sub-fields need no
  migration, unlike `projects` (SCHEMAFULL).
- CI (`.forgejo/workflows/test.yml`) deliberately excludes `src-tauri`
  (needs the full Tauri/webkit2gtk apt chain the shared runner can't
  reliably complete) - `cargo check -p app` is the manual equivalent when
  touching Tauri command wiring.
- **HANDOVER discipline**: write this file only from inside a worktree,
  never the main checkout - the main checkout may hold a different
  session's uncommitted edits (it did, at the start of this dispatch). If
  `git -C <root> status --short` shows `.claude/HANDOVER.md` modified in the
  main checkout, that is a signal something is in flight there, not a file
  to trust or overwrite.

---
*Updated: 2026-09-15 (review-followups session - verified 0 open PRs at
review-acceptance time, corrected the PR #51 mission-record error, dated the
0.18.1/unreleased-main split)*

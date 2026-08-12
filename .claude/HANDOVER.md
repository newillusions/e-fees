# E-Fees Project Handover

## Current Status
**v0.17.0 released (2026-08-12)** - desktop installer refreshed after 4.5 months (previous release was v0.16.0, 2026-03-28). Fixes the first-run setup wizard's connection test, which always failed on a fresh install (log-proven on Windows) because `testConnection()` never called `reconnect_database` before checking connection status. PR #28 merged (`833bf7f`); release commit `ef38301` + manifest sync `5dde60b`; `main` is at `5dde60b`.

- **Versions**: desktop **0.17.0**, e-fees-api 0.3.4, e-fees-scope 0.2.0 (unchanged this session).
- **`main` (origin, Forgejo)** is current source of truth - local `main` tracking is fixed (see `.claude/rules/judgment.md`), just confirm `git fetch && git log` before trusting it.
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2). Dev runs a newer SurrealDB point release than prod - don't assume schema/behavior parity between the two without checking both.
- **macOS builds are Apple-Silicon-only** since 2026-06-29 (decision:boujy4d42i8w7zovifts) - Intel dropped from `.github/workflows/build-releases.yml`'s matrix. The macOS job's display name ("Build macOS (Apple Silicon + Intel)") is stale/never renamed; don't read it as a build failure when only 1 macOS asset set appears on a release.

## Last Session
**Date**: 2026-08-12
**Summary**: Fixed `FirstRunSetup.svelte`'s `testConnection()` - it called `check_db_connection` (which only reports whether a DB client already exists, never attempts a connection) without ever calling `reconnect_database` first, so first-run Test always failed even with correct credentials. Added regression test `src/lib/components/FirstRunSetup.test.ts` (RED against the old code, GREEN after the fix; also added a local `Element.prototype.animate` polyfill since jsdom doesn't implement it and this is the first transition-using component with tests in the repo). Full battery green (`npm run test:run` 734/734, `npm run check` 0 errors, `cargo test -p app --lib` 97/0/5-ignored). PR #28 opened, CI green (run #20, 5m19s), squash-merged to main. Cut release v0.17.0 per `.claude/commands/release.md`: version bump (minor), tag pushed to origin+github, GitHub Actions build green on all 3 jobs, Forgejo release id 312 live with 5 assets, `update.json` auto-synced to GitHub by the workflow's own commit step.

**Process note**: `git push github` needed a one-time local fix - `git config credential."https://github.com".helper "!gh auth git-credential"` (plain HTTPS push wasn't picking up `gh`'s stored auth). Full detail + KB observation: `observation:9gstxw3xdgl798lpomg5`.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) - `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) - clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| v0.17.0 release | https://forge.mms.name/emittiv/fee-prop/releases/tag/v0.17.0 |
| KB obs (v0.17.0 release + first-run fix) | observation:9gstxw3xdgl798lpomg5 |

## Next Steps
1. **Clause-library backlog** - fix 4 divergent clauses, supplement 4 thin, add 7 gap clauses. Code/data verified in dev; the remaining gate is Martin's business-content review of 3 client-facing wording changes (payment 30→14d, validity 60d, Defined Role regs paragraph), not a technical blocker. (as of 2026-08-12)
2. **IDW T5 `.indd` linking** - scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`. (as of 2026-08-12)
3. **Stage 3 clause-usage mining** - corpus-ranked clause suggestions are operational (769 positive matches, 27 clause stats live); remaining is one verified real-proposal run end to end. (as of 2026-08-12)
4. **Lulu 26-97104** - waiting on client meeting to lock price; then model Acoustics 55k as a discipline line (see `judgment.md` - buy-ins are discipline lines, not reimbursable costs) + regenerate docs. (as of 2026-08-12)

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete → archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.

## Notes
- `kb_detect_project_tags` clobbers monorepo tags - do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project consumer notes) lives in `.claude/rules/judgment.md` - read it before touching pricing/proposal-export logic or Dockerfiles.
- jsdom doesn't implement `Element.animate` (Web Animations API) - any future component test that renders a Svelte `transition:` needs the same local polyfill used in `FirstRunSetup.test.ts`.

---
*Updated: 2026-08-12 (v0.17.0 release session - first-run connection-test fix; rewrote from a stale 2026-06-29/07-02 handover, several intervening sessions were documented only in KB)*

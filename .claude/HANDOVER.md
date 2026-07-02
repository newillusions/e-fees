# E-Fees Project Handover

## Current Status
**PR #11 merged** into main (merge commit `2709805`, 2026-06-29T17:35:59+04) — container build CI workflow is live. `REGISTRY_TOKEN` was set on `emittiv/fee-prop` before merge.
**PR #9 merged** into main (`4d9e59d`) — clause-selection Stage 1 (e-fees-scope) is live and deployed; verified via `/health` 200 + route registered (401 not 404) 2026-06-29.
**PR #10 merged** into main (`4436a9a`) — typeahead endpoint is live.

- **Versions** (unchanged): desktop 0.16.0, e-fees-api 0.3.4, e-fees-scope 0.2.0.
- **`main` (origin, Forgejo)** is at `2709805` — all three PRs above are in. Local branch `main` tracks a stale `github` remote and is NOT current; resolve against `origin/main` (Forgejo), not local `main`, when checking what's shipped.
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2). Dev runs a newer SurrealDB point release than prod — don't assume schema/behavior parity between the two without checking both.

## Last Session
**Date**: 2026-06-29
**Summary**: Authored container build CI workflow (.forgejo/workflows/build-containers.yml). Matrix job builds e-fees-api and e-fees-scope in parallel on ubuntu-latest with DinD docker daemon (10.0.23.137:2375). Safe push policy: login+push:latest only on main, PRs build-only. Also fixed e-fees-scope/Dockerfile (rust:1.89-slim→rust:slim + COPY Cargo.lock). PR #11 opened, CI run #1 GREEN (5m17s), then merged same day after REGISTRY_TOKEN was set. PR #9 (clause-selection Stage 1) also merged and deployed same day.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) — `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) — clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| KB obs (typeahead) | obs:yyba6std1nis8aayx3se |
| KB obs (CI workflow) | obs:j3rwfa2yz724jxbrrkgq |
| KB obs (clause-selection Stage 1 deploy) | obs:h1va1gwl1m4f7b0eg44c |

## Next Steps
1. **Lulu 26-97104** — waiting on client meeting to lock price; then model Acoustics 55k as a discipline line (see judgment.md — buy-ins are discipline lines, not reimbursable costs) + regenerate docs.
2. **Clause-library backlog** — fix 4 divergent clauses (Defined Role, Fees/Payment Terms, Proposal Validity, Prepared By); supplement 4 thin; add 7 gap clauses (Scope & Services most urgent). Code/data already verified in dev (27/27 tests passing); the remaining gate is Martin's business-content review of 3 client-facing wording changes (payment 30→14d, validity 60d, Defined Role regs paragraph), not a technical blocker.
3. **Clause Stage 2** — pre-fill ClausePicker from `is_default` + conditions (PR #9 has landed, this is now unblocked).
4. **IDW T5 `.indd` linking** — scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`.

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete → archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.

## Notes
- `kb_detect_project_tags` clobbers monorepo tags — do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Typeahead endpoint: route registered before `/projects/{id}` to avoid axum param shadowing.
- Tacit judgment (proposal domain gotchas, deploy traps, cross-project consumer notes) lives in `.claude/rules/judgment.md` — read it before touching pricing/proposal-export logic or Dockerfiles.

---
*Updated: 2026-07-02 (freshness pass — corrected stale PR #9/#11 "open" status to merged; no code changes)*

# E-Fees Project Handover

## Current Status
**PR #11 open** — container build CI workflow. Awaiting orchestrator merge + `REGISTRY_TOKEN` secret setup. Branch `feat/ci-build-containers`, commit `d341517`. CI green (run #1, 5m17s).
**PR #10 merged** into main (4436a9a) — typeahead endpoint is live in main.

- **Versions** (unchanged): desktop 0.16.0, e-fees-api 0.3.4, e-fees-scope 0.2.0.
- **Active branch**: `feat/ci-build-containers` (this session). `main` is at 4436a9a (typeahead PR #10 merged).
- **Dev DB**: `ws://10.0.23.12:8000` ns `emittiv_dev` db `projects` (v3.1.4). Prod: `ws://10.0.23.11:8000` ns `emittiv` db `projects` (v3.1.2).

## Last Session
**Date**: 2026-06-29
**Summary**: Authored container build CI workflow (.forgejo/workflows/build-containers.yml). Matrix job builds e-fees-api and e-fees-scope in parallel on ubuntu-latest with DinD docker daemon (10.0.23.137:2375). Safe push policy: login+push:latest only on main, PRs build-only. Also fixed e-fees-scope/Dockerfile (rust:1.89-slim→rust:slim + COPY Cargo.lock). PR #11 opened, CI run #1 GREEN (5m17s). REGISTRY_TOKEN secret not yet set on fee-prop — orchestrator must add before merge.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 (ns emittiv, db projects) |
| Dev DB | ws://10.0.23.12:8000 v3.1.4, ns emittiv_dev db projects |
| API container | 10.0.21.80:3200 (e-fees-api 0.3.4) — `EFEES_API_KEY` |
| Scope container | 10.0.21.81:3201 (e-fees-scope 0.2.0) — clause DB = prod |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| PR #11 (open) | feat/ci-build-containers → main — container build CI |
| PR #9 (open) | feat/clause-selection-stage1 → main — clause selection Stage 1 |
| KB obs (typeahead) | obs:yyba6std1nis8aayx3se |
| KB obs (CI workflow) | obs:j3rwfa2yz724jxbrrkgq |
| CI run #1 | https://forge.mms.name/emittiv/fee-prop/actions/runs/1 |

## Next Steps
1. **Orchestrator: add REGISTRY_TOKEN secret to emittiv/fee-prop** (write:package scope, same token as ailx + pa repos). Then merge PR #11.
2. **Orchestrator: merge PR #9** (clause-selection Stage 1) + redeploy e-fees-scope (10.0.21.81:3201) — the first main-branch CI run will automatically build+push the image.
3. **Lulu 26-97104** — waiting on client meeting to lock price; then model Acoustics 55k as a discipline line + regenerate docs.
4. **Clause-library backlog** — fix 4 divergent clauses (Defined Role, Fees/Payment Terms, Proposal Validity, Prepared By); supplement 4 thin; add 7 gap clauses (Scope & Services most urgent). Awaiting Martin.
5. **Clause Stage 2** — pre-fill ClausePicker from `is_default` + conditions (after PR #9 lands).
6. **IDW T5 `.indd` linking** — scoped in `docs/plans/2026-06-14-idw-t5-indd-linking-scope.md`.

## Open Follow-ups
- Make e-fees-scope integration-test cleanup hard-delete (currently soft-delete → archived residue accumulates).
- Drop old `ns:emittiv` on 10.0.23.12 once confirmed unneeded.

## Notes
- `kb_detect_project_tags` clobbers monorepo tags — do NOT run on e-fees.
- SurrealDB type-check fn is `type::is_datetime()` (underscore), not `type::is::datetime`.
- Critical query/SurrealValue patterns in CLAUDE.md §Critical query patterns.
- Typeahead endpoint: route registered before `/projects/{id}` to avoid axum param shadowing.

---
*Updated: 2026-06-29*

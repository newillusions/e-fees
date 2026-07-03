# E-Fees Judgment Rules

Tacit know-how not already captured in CLAUDE.md, `development-workflow.md`, or `kb-critical.md` (SurrealDB v3 traps, Tailwind removal, Tauri-MCP-only testing are pinned there - not repeated here).

## Proposal / pricing domain

- When a cost is bought-in or on-charged to the client (e.g. a sub-consultant study), model it as its own **discipline line** in the fee's design-fee distribution - never as `reimbursable_cost` / `pricing.costs`. The IDW workbook (`generate_indesign_workbook`) has no costs/reimbursables sheet; it only emits stages, post-contract items, payment schedule, revisions. Anything in `pricing.costs` is structurally invisible to the merged client `.indd` proposal - it only shows up in the desktop-only `-PRI Pricing.xlsx`. Mark up the buy-in (~×1.15) and add it as a discipline alongside Audio/Video/etc; it then folds into T2/T4 per payment stage. There is no discrete "Acoustics" (or similar) line in the merged tables - a client-facing line item needs a template change, not a data change. (obs:ve11z833mnxb9p8yiqsf)
- Clause-library promotions have two independent gates: technical (tests green, dev DB confirms rows/versions) and editorial (Martin reviewing client-facing wording changes - payment-term days, validity period, regulatory paragraphs). Tests passing does NOT mean cleared to promote to prod; check which gate is actually open before reporting "done." (obs:zh4z9htdeoep72c3udg2)

## Docker / CI

- e-fees-api and e-fees-scope both pull in `roaring@0.11.4` transitively via `surrealdb = "3.0.4"`, which requires rustc 1.90+. `rust:1.89-slim` fails the build with a rustc-version error - and as of the PR #11 fix, that exact tag doesn't even exist on Docker Hub anymore. Use `rust:slim` (floating, tracks latest stable) or pin `rust:1.90-slim` or newer; never re-introduce a `1.89` pin. (obs:8h7yvd3mfb9a928kh72u, PR #11 body)
- e-fees-scope has its **own** `API_KEY` env var, separate from e-fees-api's `EFEES_API_KEY`. When verifying a scope-service deploy without the scope key in hand, a `401` on a protected route (not `404`) is the correct signal that the route is registered and the container is healthy - don't wait for a `200` you can't produce. (obs:h1va1gwl1m4f7b0eg44c)

## Cross-service / consumer integration

- External consumers of e-fees-api (cad-export was the first, and burned a session on it) must normalize the display project number (`YY-CCCNN`, hyphens) to the record-key form (underscores) before calling any `/{id}` route - `GET /projects/26-97101` 404s, `GET /projects/26_97101` 200s. This is already the internal convention (CLAUDE.md §Critical query patterns); the judgment here is to say so explicitly when another project asks how to integrate, since the 404 reads like "wrong endpoint" rather than "wrong id format." (obs:277a9i0g5psmkvxr44uo)
- e-fees pins SurrealDB Rust SDK `3.0.4` and hit the same "JWT TTL never refreshes" class of bug that forced a fix in cad-export - but audited and confirmed **do not port that fix here**. e-fees-api/e-fees-scope connect via pure WebSocket (`Surreal::new::<Ws>`) where prod has `DURATION FOR SESSION NONE`, so the session never expires regardless of JWT TTL; the bug only bites HTTP-transport connections. Re-open this only if e-fees-api/scope ever switch to HTTP transport, or if the desktop app's HTTP fallback path (`src-tauri/src/db/client.rs:43-160`) is confirmed to actually engage in practice. (obs:ciz4qfyka85k5r14hevi)

## Environment

- Dev SurrealDB (10.0.23.12, v3.1.4) is ahead of prod (10.0.23.11, v3.1.2) by a point release. Don't assume schema or query-behavior parity between the two without checking both directly - a query that works in dev isn't proven for prod.
- **RESOLVED 2026-07-03**: local git branch `main` used to track a stale `github` remote instead of `origin` (Forgejo) - `git branch -u origin/main main` fixed the tracking pointer (the `github` remote itself is kept, for releases). Local `main` was still 10 commits behind `origin/main` at fix time (tracking-only change, no merge performed) - if `git branch -vv` ever shows `main` behind or diverged again, re-run the repoint rather than trusting local `main` as current.

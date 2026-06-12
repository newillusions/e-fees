# E-Fees Project Handover

## Current Status
Shipped two export bug fixes + a new `-PRI` API export endpoint (TDD throughout) and **deployed e-fees-api 0.3.4 to production**. Drove the Lulu Island AV proposal (26-97104) to **fully issue-ready** — var.json + IDW + `-PRI` all generated server-side. Everything verified live.

- **Versions**: desktop 0.16.0 (unchanged), **e-fees-api 0.3.4 (DEPLOYED 2026-05-31)**, e-fees-scope 0.2.0 (unchanged)
- **Branch**: main — pushed to Forgejo through `0a62786`. Only `.claude/HANDOVER.md` modified locally.
- **Tests**: e-fees-core 29/29 (3 new fee_template), e-fees-api 25/25 (3 new version-scan), desktop excel_export 8/8, clippy clean. 726 vitest + 89 Rust unit last known green (not re-run).
- **Database**: SurrealDB 3.1.2 surrealkv @ ws://10.0.23.11:8000 prod (ns:emittiv, db:projects) / 10.0.23.12 dev

## Session 2026-05-31 (part 2): -PRI API export endpoint (v0.3.4)
The internal `-PRI` pricing workbook (`generate_fee_template`, umya-spreadsheet) was desktop-only. Lifted it into **e-fees-core** (`export::fee_template`); desktop `excel_export` now re-exports it (8 tests green, no behaviour change). Added **`POST /fees/{id}/export/template`**: scans the proposal folder for the latest `{number}-(PRI|FP)-NN Pricing.xlsx` source, generates the next `-PRI-NN` via SSH temp-file bridge, NC rescan. New `SshOps::read_file` (binary-safe cat) + `list_dir`; pure `latest_pri_source_and_version` version scanner (TDD). Fixed a latent temp-file race in `open_or_create_template` (reads embedded template from in-memory Cursor now). Commit `0a62786`. Generated `26-97104-PRI-02 Pricing.xlsx` (valid, from FP-01 source) live.
**Minor cleanup TODO**: src-tauri/Cargo.toml still lists `umya-spreadsheet` as a now-unused direct dep (desktop uses it via core re-export) — can be dropped.

## Last Session
**Date**: 2026-05-31
**Summary**: Fixed two ailx-reported export bugs (var.json folder + IDW costs), deployed them to prod (had to fix a Docker build blocker first), and finalized the Lulu Island AV proposal data.

### Bug fixes (TDD, 4 commits pushed to Forgejo)
- `ffefdcd` `fix(api)` — **BUG 1**: `export_fee_json` built the proposal path from `project.name` (legal name) instead of `project.name_short`, writing to a stray folder every regen. Extracted a pure `ProposalPaths` builder keyed on `name_short`. Test: `proposal_paths_use_name_short_not_legal_name`.
- `ad8ba19` `fix(core)` — **BUG 2**: IDW workbook (`generate_indesign_workbook`) omitted reimbursable costs. Added a **T5 Reimbursable Costs** sheet from `fee.reimbursable_costs` (fallback `pricing.costs`). Test: `test_workbook_includes_reimbursable_costs_sheet`. **NOTE: needs a matching table linked in the .indd before costs auto-merge — one-time manual InDesign step, not done.**
- `45da2f6` `chore` — bump e-fees-api 0.3.2→0.3.3 + fix stale OpenAPI doc version.
- `b61038d` `fix(docker)` — **build blocker**: the Dockerfile regenerated a minimal workspace and skipped `Cargo.lock`, so the build resolved `diskann-0.52.0` fresh, which fails under `rust:1.90-slim`. Fixed by `COPY Cargo.lock`. (KB: `observation:zxdxu99okpi0jkig9fuq`.)

### Deploy (e-fees-api 0.3.3 → 10.0.21.80:3200) — VERIFIED LIVE
Built+pushed on AI server via SSH, deployed via `unraid_docker(action=update, confirm=true)`. `/health` returns `version: 0.3.3`, surrealdb dep ok. BUG 1 confirmed fixed in prod: re-export landed in canonical `26-97104 Lulu-Island-AV/` folder, stray `LULU ISLAND DEVELOPMENT` folder gone.

### Lulu Island AV proposal (fee:26_97104_0) — issue-ready bar -PRI
- Fee structure approved + unchanged: AV base 250k + accel 37.5k + acoustics 55k (as discipline line) = 342,500 ex-VAT / 359,625 incl. Design dist 305k.
- Client city corrected Dubai→Abu Dhabi (`company:k1ivcfv3buuoyzqzpfeg`).
- Contact phone set (`contacts:6pv05af5kzv9c0z6a8mx` Admin AUH) = `+971 2 632 8681`.
- var.json re-exported, 23 fields, all correct (Project/Client City = Abu Dhabi, Contact Phone real).
- **`-PRI Pricing.xlsx` DONE** — generated server-side via the new v0.3.4 endpoint → `26-97104-PRI-02 Pricing.xlsx` (valid, sourced from FP-01). Proposal is now fully issue-ready on the e-fees side.

### Env note (this machine)
`emittiv-container-utils` path dep resolves to `/Volumes/base/dev/container-utils/rust`, but the clone is at `/Volumes/base/dev/claude/container-utils`. Created symlink `dev/container-utils → claude/container-utils` so the workspace compiles. Do NOT edit the Cargo.toml path (CI relies on it). KB: `observation:3uberl62bn0t07w5f0k8`.

## Key Context
| Resource | Value |
|----------|-------|
| Production DB | ws://10.0.23.11:8000 v3.1.2 surrealkv (ns: emittiv, db: projects) |
| Dev DB | ws://10.0.23.12:8000 |
| API Container | 10.0.21.80:3200 (**e-fees-api v0.3.3**, `/health` = version) |
| Scope Container | 10.0.21.81:3201 (e-fees-scope v0.2.0) |
| Forgejo | forge.mms.name/emittiv/fee-prop |
| Deploy | build via `ssh unraid-ai` (skill `deploy-containers`), deploy via `unraid_docker(update)` — NOT raw docker |
| NC proposal folders | on **Primary** 10.0.20.12 (`ssh unraid`), `/mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/<number> <name_short>/` |
| API key var | `EFEES_API_KEY` in creds.env |
| RecordId key extraction | `record_key_string(&id.key)` — NOT `.key()` |
| Project record-key form | underscore (`26_97104`), NOT display number (`26-97104`) |

## Next Steps
1. **Lulu Island**: fully issue-ready on the e-fees side (var.json + IDW + -PRI all generated). Martin to issue.
2. **IDW T5 .indd linking** (carryover): link a Reimbursable Costs table in the .indd template to the new T5 sheet so costs auto-merge. Until then, on-charged buy-ins go in as discipline lines (current workaround).
3. **Open P5 hub decisions** (both need a user call, not urgent):
   - shared-directory v0.4.0 record-level signin (`message:ieo33kf58mc9bbsbqslw`, dev) — needs `e_fees_aggregator` credential from dev.
   - Phase 7 dev-DB namespace merge (`message:rgmxu21umljoo4ndb3vm`, orchestrator) — pick Option A vs B.

## Critical Rules
1. **`type::record('table', $key)` for record-id parameterization** — never `table:$key` directly.
2. **`query_bind_map` for multi-param CREATE/UPDATE** in desktop client; raw `state.db.query(...).bind(...)` for axum services.
3. **Project/fee route `{id}` is the SurrealDB record key (underscore form)**.
4. **SCHEMAFULL tables (company/contacts/projects/country/currency/activity_log/scope_revision) hard-error on undefined-field writes on 3.1.2.** Write only defined fields. `fee` + scope tables are SCHEMALESS.
5. **export paths use `project.name_short`** (canonical folder `{number} {name_short}`), never `project.name`.
6. **Docker image build needs `Cargo.lock` copied** (Dockerfile), else diskann float breaks the build.

---
*Updated: 2026-05-31*

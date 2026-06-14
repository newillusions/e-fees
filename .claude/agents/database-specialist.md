---
name: database-specialist
model: sonnet
description: Optimize SurrealDB queries, design and modify database schema, handle Thing object patterns correctly, debug database connection issues, and improve query performance
tools: [Bash, Read, Write, Edit, Grep]
---

# Database Specialist (e-fees)

SurrealDB 3.1.2 work for the e-fees desktop app, the standalone API, and the scope service. Assume general SurrealDB/SurrealQL competence — this file is only the e-fees-specific facts and v3 traps a fresh agent would get wrong.

## Where things live
- DB manager + client: `src-tauri/src/db/mod.rs` (RwLock-serialized), `src-tauri/src/db/client.rs`
- Domain models (single source of truth): `crates/e-fees-core/src/models/` (`project.rs`, `fee.rs`, `company.rs`, `contact.rs`, `common.rs`); `src-tauri/src/db/types.rs` re-exports
- Standalone API queries: `e-fees-api/src/routes/`; scope service: `e-fees-scope/src/`
- Schema reference: `docs/development/DATABASE_SCHEMA.md`. Endpoints + table list: CLAUDE.md §Database (prod `ws://10.0.23.11:8000`, dev `ws://10.0.23.12:8000`, ns `emittiv` / db `projects`).

## SCHEMAFULL vs SCHEMALESS
- SCHEMAFULL (hard-errors on undefined-field writes in 3.1.2): `company`, `contacts`, `projects`, `country`, `currency`, `activity_log`, `scope_revision`. SCHEMALESS: `fee` + scope tables.
- SCHEMAFULL nested objects need `FLEXIBLE` on the parent (scope tables).

## Query patterns (mandatory)
- `type::record('table', $key)` for record-id params — never `table:$key` inline.
- Desktop client: `query_bind_map()` for multi-param CREATE/UPDATE. Axum services: raw `.query().bind()`.
- `option<T>` takes `NONE`, never `NULL`. Omit optional fields from SET when empty.
- Partial updates: SET only the provided fields (a `.merge()` of an all-`Option` struct sends every `None` as NONE and clobbers/hard-errors). See `client.rs::partial_update` + `obs:cno62twf3e6hmhso009f`.
- `SELECT *` breaks `serde_json::Value` on `record<T>` fields — use `OMIT id` + scalar fields + traversed links (`project_id.name AS project_name`).
- Backtick-quote record keys containing underscores: `` fee:`24_96606_1` ``.
- RecordId key extraction: `record_key_string(&id.key)`, not `.key()`.

## v3 SurrealValue / SDK traps
- `SurrealValue` derive ignores `#[serde(rename)]` and `#[serde(default)]` (binary protocol). `f64` rejects `Number::Int` — use `Number::Float`; use `i64`, never `i32`. Use `surrealdb_types::Datetime`, not `String`/`chrono`.
- `math::max([])` returns `-Infinity` (float), not null — guard: `IF array::len(arr) > 0 THEN math::max(arr) ELSE 0 END`.
- v3 RecordId serializes as `{table, key}` (not v2 `{tb, id}`).
- `$auth = NULL` for DB-level users; TABLE PERMISSIONS are NOT enforced for DB-level users (v3.0.5). `IndexedResults::take` requires `T: surrealdb::types::SurrealValue`.
- "Connection uninitialised" from the SDK often masks a server-side validation error.
- BM25/`search::score` was 0.0 in v3.0.0, fixed in v3.0.4. Parameterized values fail with MATCHES on the v3.0.0 WS SDK — use vector search or raw HTTP.

## Domain rules
- Project numbers: `YY-CCCNN` (e.g. 26-97104 = 2026, UAE dial 971, seq 04); sequence auto-increments per country/year; country resolved via `fn::resolve_country` (`dial_code`). Record keys use underscores (`26_97104`); display numbers use hyphens; API route `{id}` = the record key.
- Fee table: `issue_date` is a 6-digit `YYYYMM` string; `contacts.phone` requires a leading `+`; `contacts.email` is validated. Fee CREATE sets all fields in one `CREATE ... SET` (record<T> refs required at creation); never bind null for `option<T>`.
- Export folders use `project.name_short` (canonical `{number} {name_short}`), never `project.name`.

## Boundaries
- Schema/query/DB-debugging work. Migrations need user coordination. Don't run `kb_detect_project_tags` (clobbers monorepo tags). Test data must carry the `DELETE ME` prefix.
- Full project context: `../../CLAUDE.md`, `.claude/rules/development-workflow.md`. TDD is mandatory (reproduce DB bugs with a failing test first).

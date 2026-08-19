-- v006_backfill_metadata_fields.surql
-- Historical fee-data backfill, Phase P0 — schema migration.
-- Owner-approved DDL (revised 2026-08-18) from
-- /Volumes/base/dev/.claude/research/2026-08-08-pricing-ingestion/03-ingestion-plan.md
-- section 2 "P0 — Schema migration". D1/D2/D3 applied. Rollback:
-- v006_backfill_metadata_fields_rollback.surql.
--
-- Renamed from 006-backfill-metadata-fields.surql (2026-08-19, migration-
-- guard onboarding) to the guard's discovery convention
-- (~/.claude/scripts/apply-migration.sh: `v<NNN>_<slug>.surql` /
-- `v<NNN>_<slug>_rollback.surql`) and given the trailing schema_version
-- stamp the guard's classifier requires (see bottom of file) - content
-- otherwise unchanged from the merged PR #39 version. Migrations from here
-- on (007+) use this same `vNNN_slug.surql` naming; 001-005 keep their
-- original hyphenated names as historical/dead files that will never be
-- re-run through the guard (see 000-bootstrap-schema-version-tracking.surql
-- for why, and why this migration's predecessor version is 5, not the count
-- of files that actually landed).
--
-- Run on: emittiv/projects DB, dev (10.0.23.12:8000, ns emittiv_dev) FIRST,
-- verified there, then prod (10.0.23.11:8000, ns emittiv) only after Martin
-- confirms this exact field list - schema changes on live prod are
-- "major change" class and need explicit per-target confirmation, not
-- blanket authorization. The orchestrator owns the prod application via
-- ~/.claude/scripts/apply-migration.sh (obs:q2lbpycedhr8ppggv56u) - note for
-- that guard: every `DEFINE FIELD OVERWRITE` below targets a field that does
-- NOT yet exist on prod (this migration net-adds fields; it does not
-- re-define any field prod already has), so the guard's "not-proven-additive"
-- path (triggered by an OVERWRITE of an EXISTING table) is not expected to
-- fire here for that reason - it DOES still classify NOT_ADDITIVE overall,
-- because every statement is `DEFINE FIELD OVERWRITE` (not `IF NOT EXISTS`)
-- and several carry ASSERT clauses outside the guard's enumerated safe-clause
-- set (migration_classify.py) - expected and accepted, not a defect; the
-- migration pays the whole-database recoverability proof instead of riding
-- the pure-whitelist path. `OVERWRITE` (not `IF NOT EXISTS`) is used
-- throughout specifically so a partial hand-apply can be safely re-run.
--
-- `projects` is SCHEMAFULL: every field must be explicitly defined; an
-- undefined field hard-errors on write (confirmed live on dev: attempting to
-- write `disciplines` before this migration returns
-- "Found field 'disciplines', but no such field exists for table 'projects'").
--
-- Every field is `option<T>` (absent value is NONE, never NULL) so the
-- backfill can populate incrementally without breaking existing writers
-- (desktop app / e-fees-api) or existing rows - none of the fields below is
-- read by the app today (ProjectModal.svelte binds to exactly 6 fields via
-- an explicit validator array, per the plan's evaluator finding).
--
-- REBUILD-SAFETY PREREQUISITE (added 2026-08-19, found by the guard's own
-- dry-run): `projects`'s base definition predates this repo's migration
-- chain (same situation pa-core-rs documents in schema/v000_baseline_email.
-- surql for `email`). apply-migration.sh's rollback-proof cycle runs this
-- file alone against a brand-new throwaway namespace, so if `projects`
-- isn't already defined there, `data_provenance`'s FLEXIBLE clause below
-- hits a table SurrealDB auto-created SCHEMALESS - confirmed live:
-- "FLEXIBLE can only be used in SCHEMAFULL tables". `IF NOT EXISTS` makes
-- this a no-op against dev/prod, which already have `projects` defined
-- exactly this way (verified via INFO FOR DB on both, 2026-08-19).
DEFINE TABLE IF NOT EXISTS projects TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

-- D1: two-field open-vocabulary type/category (not a hard enum - the
-- vocabulary is still being discovered, an enum would reject legitimate new
-- values).
DEFINE FIELD OVERWRITE project_type ON projects TYPE option<string>;
DEFINE FIELD OVERWRITE project_category ON projects TYPE option<string>;

-- D2: disciplines as a list of structured entries (not a flat array<string>).
-- The five standard disciplines are fixed but optional; "Sub" is a
-- subcontractor slot, not a discipline name - kind=Sub + sub_company must
-- never reach a client-facing export (see the plan's §5 client-visibility
-- test requirement, a P1/P3 deliverable, not enforced by this DDL).
DEFINE FIELD OVERWRITE disciplines ON projects TYPE option<array<object>>;
DEFINE FIELD OVERWRITE disciplines[*].kind ON projects
    TYPE string
    ASSERT $value IN ['Lighting', 'Video', 'Audio', 'SFX', 'Show Control', 'Sub'];
DEFINE FIELD OVERWRITE disciplines[*].sub_discipline ON projects TYPE option<string>;
DEFINE FIELD OVERWRITE disciplines[*].sub_company ON projects TYPE option<record<company>>;

-- D3: stage/outcome split (not a single Won/Lost/Open/Cancelled/No-Response
-- enum). `status` (existing app-workflow field) is DELIBERATELY untouched -
-- `stage` is a new, parallel DB-only field. Retiring `status` in favour of
-- `stage` is a larger, separate, app-touching change, out of scope here.
DEFINE FIELD OVERWRITE stage ON projects
    TYPE option<string>
    ASSERT $value = NONE OR $value IN
        ['RFP', 'Design', 'Construction', 'Completed', 'Superseded', 'Cancelled'];
DEFINE FIELD OVERWRITE outcome ON projects
    TYPE option<string>
    ASSERT $value = NONE OR $value IN ['Won', 'Lost', 'No Response', 'Cancelled'];
    -- NONE = open/pending - "Open" is not a value (D3). REVERSIBLE: this is
    -- a current state, not a permanent verdict - every change must be
    -- logged via `activity_log` (P2's job, not this migration's).
DEFINE FIELD OVERWRITE outcome_changed_at ON projects TYPE option<datetime>;
DEFINE FIELD OVERWRITE successor ON projects TYPE option<record<projects>>;
    -- Forward pointer, populated only when stage=Superseded. Lineage
    -- matching uses the `folder` field, never `name` (P2 detail).

-- Untouched by any decision - carried forward from the 2026-08-08 draft
-- verbatim.
DEFINE FIELD OVERWRITE gfa_sqm         ON projects TYPE option<float>;
DEFINE FIELD OVERWRITE plot_area_sqm   ON projects TYPE option<float>;
DEFINE FIELD OVERWRITE scope_summary   ON projects TYPE option<string>;
DEFINE FIELD OVERWRITE loss_reason     ON projects TYPE option<string>;
DEFINE FIELD OVERWRITE source_document ON projects TYPE option<string>;
DEFINE FIELD OVERWRITE data_provenance ON projects TYPE option<object> FLEXIBLE;

-- Schema-version stamp (guard requirement, migration_classify.py's
-- SCHEMA_VERSION_STAMP rule - structural template match, not a substring
-- search). Idempotent re-apply: whichever branch fires, the result is the
-- same row at value='6'.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '6', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '6', updated_at = time::now())
END;

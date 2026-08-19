-- v006_backfill_metadata_fields_rollback.surql
-- Rollback for v006_backfill_metadata_fields.surql (P0 historical-backfill
-- schema migration). Removes every field the forward migration defined.
-- Child object-array fields are removed before their parent.
--
-- Renamed from 006-backfill-metadata-fields-rollback.surql (2026-08-19,
-- migration-guard onboarding) to the guard's `_rollback.surql` sibling
-- convention; trailing schema_version stamp-revert added (see bottom of
-- file) - the guard's rollback-proof cycle asserts schema_version reads
-- back to the predecessor version (5) after this file runs, every time it
-- proves the migration, not just here.
--
-- Safe to re-run: `REMOVE FIELD IF EXISTS` is idempotent.
--
-- IMPORTANT (found live on dev, SurrealDB 3.1.4, during P0 rollback proof):
-- defining `disciplines[*].kind` (etc.) implicitly creates an intermediate
-- wildcard object field that `INFO FOR TABLE` echoes as `disciplines.*`.
-- Removing the three `disciplines[*].<child>` fields and the `disciplines`
-- parent does NOT remove this intermediate field - it must be removed
-- explicitly, using the SAME bracket-star syntax the forward migration used
-- to define it (`disciplines[*]`, not the dot-star form `INFO FOR TABLE`
-- displays - `REMOVE FIELD \`disciplines.*\`` silently no-ops, matching
-- nothing, and returns OK regardless because of `IF EXISTS`). Verified this
-- rollback leaves `INFO FOR TABLE projects` fields identical to the
-- pre-migration baseline (16 keys: area/city/country/folder/name/name_short/
-- number(+3 sub)/status/time(+2 sub)) - not re-verified against prod's
-- 3.1.2, which may or may not share this exact wildcard-field behavior.
--
-- Run on: emittiv/projects DB, same target (dev then prod) as the forward
-- migration.

REMOVE FIELD IF EXISTS project_type ON projects;
REMOVE FIELD IF EXISTS project_category ON projects;

REMOVE FIELD IF EXISTS disciplines[*].sub_company ON projects;
REMOVE FIELD IF EXISTS disciplines[*].sub_discipline ON projects;
REMOVE FIELD IF EXISTS disciplines[*].kind ON projects;
REMOVE FIELD IF EXISTS disciplines[*] ON projects;
REMOVE FIELD IF EXISTS disciplines ON projects;

REMOVE FIELD IF EXISTS stage ON projects;
REMOVE FIELD IF EXISTS outcome ON projects;
REMOVE FIELD IF EXISTS outcome_changed_at ON projects;
REMOVE FIELD IF EXISTS successor ON projects;

REMOVE FIELD IF EXISTS gfa_sqm ON projects;
REMOVE FIELD IF EXISTS plot_area_sqm ON projects;
REMOVE FIELD IF EXISTS scope_summary ON projects;
REMOVE FIELD IF EXISTS loss_reason ON projects;
REMOVE FIELD IF EXISTS source_document ON projects;
REMOVE FIELD IF EXISTS data_provenance ON projects;

-- Schema-version stamp-revert (guard requirement - see
-- v006_backfill_metadata_fields.surql's matching stamp for the structural
-- template the classifier matches). Idempotent re-apply.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '5', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '5', updated_at = time::now())
END;

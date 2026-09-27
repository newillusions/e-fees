-- v008_activity_log_action_merge.surql
-- Widens `activity_log.action`'s ASSERT to admit 'merge', and fixes the
-- table-level backfill-noise gap while we're touching this table's schema.
--
-- BACKGROUND (found 2026-09-27, dispatched activity-feed/logging-gap task):
-- `logMerge()` (src/lib/services/activityLogger.ts) has written
-- `action: 'merge'` since project-merge/cascade-delete shipped (PR #35,
-- 2026-08-18) - called from MergeProjectModal.svelte on every real merge.
-- Every one of those writes has silently FAILED: the schema's own ASSERT
-- (`$value INSIDE ['create', 'update', 'delete', 'status_change']`) rejects
-- 'merge', and `logActivity()` is deliberately fire-and-forget (catches and
-- only warns - see activityLogger.ts's own doc-comment, which already
-- flagged this exact risk without confirming it live). Confirmed live this
-- session: prod `activity_log` (10.0.23.11, ns emittiv) holds 74 rows total
-- - 70 from the 2026-08-19 P2 backfill (user
-- 'backfill:ingestion-P2-stage-outcome') and 4 'system' rows from
-- 2025-12-09/10 (create/update/update/status_change - a one-off dev smoke
-- test, not real usage) - zero 'merge' rows despite the feature being live
-- for 5+ weeks, and zero real create/update/delete/status_change rows from
-- ANY desktop write path since Dec 2025 (a separate, application-layer gap -
-- see PR body / RETURN payload for the other logging-coverage fixes, which
-- are app-code changes, not schema changes, and don't need a migration).
--
-- This migration ONLY widens the ASSERT (additive: every value that passed
-- before still passes; 'merge' is now also accepted). It does not touch
-- `entity_type`, any other field, or existing rows.
--
-- Run on: emittiv/projects DB, dev (10.0.23.12:8000, ns emittiv_dev) FIRST,
-- then prod (10.0.23.11:8000, ns emittiv) - orchestrator-owned via
-- ~/.claude/scripts/apply-migration.sh, per this repo's standing migration
-- workflow (docs/development/DATABASE_SCHEMA.md).
--
-- NOTE found while proving this migration live (2026-09-27): the dev HOST
-- (10.0.23.12) actually carries TWO namespaces - the guard-tracked
-- `emittiv_dev` (has `app_state`/schema_version, what this migration and
-- apply-migration.sh target) AND an older, untracked `emittiv` namespace
-- with no `app_state` table (same one v007_fee_revisions.surql's header
-- documents finding). `db/tests.rs`'s `dev_db_config()` - the config every
-- `#[ignore]`d live-DB test in this file actually connects with - points at
-- the UNTRACKED `emittiv` namespace, not `emittiv_dev`. This migration was
-- applied to BOTH this session (emittiv_dev via the guard; emittiv via a
-- direct hand-applied `DEFINE FIELD OVERWRITE`, since it has no
-- schema_version to stamp) so the new regression tests below could actually
-- run against what they connect to. A future migration touching
-- `activity_log` should do the same, or the guard-tracked apply alone will
-- silently not cover what this test file exercises.
--
-- Applied by this dispatch: DEV ONLY, both dev-host namespaces above (HARD
-- CONSTRAINT: no prod DB writes from this session). Prod (10.0.23.11, ns
-- emittiv) is UNCHANGED and still rejects 'merge' - orchestrator-owned next
-- step, same as every other pending migration in this repo's history.

DEFINE FIELD OVERWRITE action ON activity_log
    TYPE string
    ASSERT $value INSIDE ['create', 'update', 'delete', 'status_change', 'merge']
    PERMISSIONS FULL;

-- Schema-version stamp (guard requirement, migration_classify.py's
-- SCHEMA_VERSION_STAMP rule - structural template match, not a substring
-- search). Idempotent re-apply: whichever branch fires, the result is the
-- same row at value='8'.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '8', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '8', updated_at = time::now())
END;

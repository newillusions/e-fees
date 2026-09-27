-- v008_activity_log_action_merge_rollback.surql
-- Rollback for v008_activity_log_action_merge.surql.
--
-- Restores the original (narrower) ASSERT list. NOTE: any 'merge' rows
-- written while the widened ASSERT was live are NOT removed by this
-- rollback and would then violate the restored ASSERT on their next write -
-- they are untouched reads until someone updates them, since SurrealDB only
-- re-validates a field on write, not retroactively on schema change.

DEFINE FIELD OVERWRITE action ON activity_log
    TYPE string
    ASSERT $value INSIDE ['create', 'update', 'delete', 'status_change']
    PERMISSIONS FULL;

-- Schema-version stamp-revert (guard requirement). Idempotent re-apply.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '7', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '7', updated_at = time::now())
END;

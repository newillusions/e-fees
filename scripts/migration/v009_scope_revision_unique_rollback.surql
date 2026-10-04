-- v009_scope_revision_unique_rollback.surql
-- Rollback for v009_scope_revision_unique.surql.
--
-- Order matters: the index goes back to NON-unique first, then rows that
-- v009 renumbered get their original (duplicate) revision number back, then
-- the audit field is removed. `scope_assembly.current_revision` values that
-- v009 raised are NOT lowered (a higher counter is harmless and lowering it
-- would re-open the duplicate-number hazard).

DEFINE INDEX OVERWRITE idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision;

UPDATE scope_revision SET revision = renumbered_from, renumbered_from = NONE WHERE renumbered_from != NONE;

REMOVE FIELD IF EXISTS renumbered_from ON scope_revision;

-- Schema-version stamp-revert (guard requirement). Idempotent re-apply.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '8', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '8', updated_at = time::now())
END;

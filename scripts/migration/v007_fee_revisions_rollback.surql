-- v007_fee_revisions_rollback.surql
-- Rollback for v007_fee_revisions.surql.
--
-- IMPORTANT ASYMMETRY, stated plainly: the forward migration did not
-- introduce new schema - it formalized field/index definitions that
-- already existed live, undocumented, since before this migration chain
-- began, AND fixed a write-validation corruption on the `revisions`
-- wildcard sub-field via a REMOVE+DEFINE round-trip on the parent field
-- (see the forward file's ROOT-CAUSE section - read it before running this
-- rollback for real). This rollback therefore does NOT restore a prior
-- working state; it REMOVES the `rev`/`revisions` definitions and the
-- `fee_project_rev` index entirely, which is a strictly worse state than
-- either "before v007" (fields existed, just untracked - and, per the
-- forward file's finding, may or may not have been in the corrupted
-- write-validation state) or "after v007" (fields exist, tracked, and
-- confirmed writable). It exists only to satisfy the migration guard's
-- structural requirement that every migration have a rollback sibling that
-- round-trips cleanly on a throwaway namespace - do NOT run this against
-- dev or prod as a real rollback unless deliberately reverting the whole
-- fee-revisions feature. `fee` is SCHEMALESS, so `REMOVE FIELD` only
-- removes the field's enforcement/VALUE-clause definition, not the
-- underlying stored `revisions[]`/`rev` data on existing rows - but `rev`
-- would stop being auto-computed and the unique index would stop
-- protecting against duplicate (project_id, rev) writes.
--
-- The `revisions.*` line below is confirmed (this session) to be a no-op
-- against a live target - `REMOVE FIELD` on this wildcard spelling matches
-- nothing regardless of `IF EXISTS`, the same way `disciplines[*]`'s
-- rollback in v006 documents. Kept for documentation symmetry with the
-- forward file, not because it does anything.
--
-- Safe to re-run: `REMOVE FIELD IF EXISTS` / `REMOVE INDEX IF EXISTS` are
-- idempotent.

REMOVE INDEX IF EXISTS fee_project_rev ON fee;
REMOVE FIELD IF EXISTS `revisions.*` ON fee;
REMOVE FIELD IF EXISTS revisions ON fee;
REMOVE FIELD IF EXISTS rev ON fee;

-- Schema-version stamp-revert (guard requirement). Idempotent re-apply.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '6', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '6', updated_at = time::now())
END;

# e-fees schema migrations

Applied through the workspace migration guard,
`~/.claude/scripts/apply-migration.sh` (`obs:q2lbpycedhr8ppggv56u`) - the
same tool `pa-core-rs` uses. Full workflow, version-stamp discipline, and
apply command: `docs/development/DATABASE_SCHEMA.md` §"Migration workflow".

## Quick reference

- Forward: `vNNN_slug.surql` · Rollback: `vNNN_slug_rollback.surql` - both
  required, the guard refuses to apply a migration with no rollback sibling.
- Every migration ends with the `app_state.schema_version` stamp idiom
  (copy the trailing block from `v006_backfill_metadata_fields.surql`,
  replace the version number in both branches - and in the rollback,
  `version - 1`).
- `001-create-venue-table.surql` … `005-country-normalization.surql`
  predate the guard and keep their original names - historical, never
  re-run. `000-bootstrap-schema-version-tracking.surql` is the one-time,
  hand-applied bootstrap that created `app_state` and stamped the baseline
  at `'5'` (see that file's header for exactly which of 001-005 actually
  reached each environment - not all five did).
- Apply order: dev first (`--host 10.0.23.12:8000 --ns emittiv_dev`,
  `--dry-run` then for real), confirm, then prod
  (`--host 10.0.23.11:8000 --ns emittiv`) - orchestrator-owned, not run by
  this project instance.

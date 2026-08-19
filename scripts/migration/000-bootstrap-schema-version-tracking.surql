-- 000-bootstrap-schema-version-tracking.surql
-- ONE-TIME bootstrap for the workspace migration guard
-- (~/.claude/scripts/apply-migration.sh, obs:q2lbpycedhr8ppggv56u).
--
-- NOT applied through apply-migration.sh. Apply by hand, once, directly via
-- surql.sh:
--   ~/.claude/scripts/surql.sh --host <HOST:PORT> --ns emittiv --db projects \
--     --creds EFEES_SURREALDB "$(cat 000-bootstrap-schema-version-tracking.surql)"
--
-- WHY THIS CANNOT GO THROUGH THE GUARD (structural, not a convenience
-- choice): apply-migration.sh's monotonic-step gate reads the CURRENT
-- version via `SELECT key, value FROM app_state WHERE key = 'schema_version'`
-- before it will apply anything. When `app_state` does not exist yet, that
-- query errors (SurrealDB: "table 'app_state' does not exist" under this
-- database's strict-mode behaviour - confirmed live against prod
-- 2026-08-19) rather than returning an empty/zero result, so the guard dies
-- before it can apply ANY version, including a hypothetical "version 1".
-- There is no predecessor state to step from - this is the same chicken/egg
-- case pa-core-rs resolved identically: its schema/README.md's version-stamp
-- discipline "starts from v045" and v000-v044 (including v026, which first
-- DEFINEs `app_state` there) were all applied by hand before the guard
-- existed. This file is e-fees's equivalent one-time step.
--
-- WHY THE BASELINE VALUE IS '5', NOT '0' (a real finding, not a guess -
-- verified live against BOTH dev 10.0.23.12/emittiv_dev and prod
-- 10.0.23.11/emittiv on 2026-08-19, `INFO FOR DB` / `INFO FOR TABLE`
-- compared directly, not assumed from the file numbering):
--   * 001-create-venue-table.surql, 002's Step 1 (projects.venue_id), and
--     004-create-venues-from-projects.surql were NEVER applied to EITHER
--     environment - no `venue` table exists on dev or prod, and `projects`
--     carries no `venue_id` field on either. This was evidently an abandoned
--     parallel "venue" domain-model restructure from 2026-02 that never
--     shipped - DATABASE_SCHEMA.md and CLAUDE.md's key-tables list have
--     never mentioned `venue`.
--   * 002's Step 2 (the new project-status ASSERT enum) DID land on both -
--     `projects.status`'s live ASSERT list matches the new lifecycle
--     (Lead/RFP/Submitted/Awarded/Design/Construction/Completed/Lost/No
--     Response/Cancelled/On Hold/Superseded) on both dev and prod, and the
--     old values (Active/Revised) are absent from both.
--   * 003-data-migration.surql DID land on both - `fee.status` samples on
--     prod are exactly {Accepted, Draft, Rejected, Sent, Superseded} (no
--     Awarded/Completed/Lost/Revised, the old lifecycle), matching 003's
--     described post-migration state.
--   * 005-country-normalization.surql DID land on both - `country.iso2`
--     field + `idx_country_iso2` index present on both, populated.
--   * dev and prod's live table sets are IDENTICAL (14 tables each,
--     byte-for-byte same DEFINE TABLE lines from INFO FOR DB) - so this is
--     not a case of dev/prod being at two different real states; there is
--     one consistent pre-tracking baseline shared by both.
-- Decision: since (a) 006-backfill-metadata-fields.surql already exists,
-- already merged, and already numbered "006" against this exact file
-- sequence (PR #39, commit eae22fabaf5), and (b) the only two migrations
-- that actually landed (003, 005) plus the untouched carry-forward fields
-- are IDENTICAL between dev and prod, stamping the baseline at '5' draws
-- the pre-tracking/post-tracking line at the same point the already-shipped
-- "006" filename already draws it, without renumbering merged work and
-- without papering over any dev/prod inconsistency (there isn't one to
-- paper over - both environments verified identical). The abandoned venue
-- restructure (001, 002-step-1, 004) is simply dead history that never
-- reached either database; it does not need a version slot.
--
-- `app_state` table shape matches pa-core-rs's schema/v026_app_state.surql
-- exactly (same fields, same index) - reusing an existing, already-proven
-- workspace convention rather than inventing a new one.

DEFINE TABLE app_state SCHEMAFULL;

DEFINE FIELD key ON app_state TYPE string;
DEFINE FIELD value ON app_state TYPE string;
DEFINE FIELD updated_at ON app_state TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_app_state_key ON app_state FIELDS key UNIQUE;

CREATE app_state SET key = 'schema_version', value = '5', updated_at = time::now();

-- v007_fee_revisions.surql
-- Formalizes the LIVE `fee.rev` / `fee.revisions` / `fee.revisions.*` field
-- definitions and the `fee_project_rev` unique index as a tracked
-- migration, AND fixes a live SurrealDB write-validation corruption this
-- migration's own first draft triggered on the dev namespace (see the
-- ROOT-CAUSE section below - read it before touching this file again).
--
-- BACKGROUND: these fields already existed on both dev (10.0.23.12) and
-- prod (10.0.23.11) - verified live this session via `INFO FOR TABLE fee`
-- on dev - but were never captured by any tracked migration; the only
-- prior record of their shape was DATABASE_SCHEMA.md's own audit banner,
-- which explicitly flags its DEFINE FIELD/DEFINE INDEX blocks as
-- "illustrative... not verified against a live INFO FOR TABLE dump" - and
-- which, critically, showed the WRONG, unsafe form of the VALUE clause:
-- `VALUE math::max($value.revisions[*].revision_number)`. The project's own
-- pinned kb-critical rule ("Surrealdb V3 Math Max Empty Array") documents
-- why that form is a live bug: `math::max([])` returns `-Infinity` (a
-- float), not NONE/0, which cannot coerce into the `int`-typed `rev` field.
-- The ACTUAL live definition already guards this with an IF/ELSE - the DDL
-- below matches the live form verbatim, not the stale doc.
--
-- This migration is part of the e-fees revision audit + fix (2026-08-28):
-- the underlying application bug was that EVERY fee-construction call site
-- wrote `revisions: vec![]`, so this VALUE-computed `rev` field always
-- resolved to 0 for every fee row ever created (confirmed live: 100% of
-- fee rows on dev had rev=0, revisions=[], at time of writing) - the
-- `fee_project_rev` UNIQUE index then rejected any second fee row for a
-- project outright. The application fix (Revision::new + every
-- FeeCreate/FeeUpdate call site) is what actually changes behaviour going
-- forward; `backfill_fee_revisions_seed` (crates/e-fees-core/src/bin/)
-- retroactively repairs existing rows. This migration's job is to make the
-- pre-existing field shape reproducible/auditable AND, per the discovery
-- below, to leave the target able to actually WRITE to `revisions` at all.
--
-- ROOT-CAUSE, found live on dev 2026-08-28 (a session all its own - do not
-- rediscover this the hard way): this migration's FIRST draft used
-- `DEFINE FIELD OVERWRITE revisions ...` and `DEFINE FIELD OVERWRITE
-- \`revisions.*\` ...` against the tracked dev namespace (emittiv_dev). It
-- applied cleanly (guard's rollback-proof round-trip and the real apply
-- both reported success, `INFO FOR TABLE fee` looked byte-identical
-- before/after), but every subsequent write of a non-empty `revisions[]`
-- array - CREATE or UPDATE, parameterized or literal, native-typed bind or
-- pre-serialized JSON, `SET`, `MERGE`, or `+=` - failed with:
--   "Couldn't coerce value for field `revisions.*`... Expected `object`
--    but found `NONE`"
-- Isolated by direct probing (not guessed): a brand-new, completely
-- unrelated throwaway table with the identical
-- `array<object>` + `.* -> object` wildcard shape reproduced the SAME
-- error on emittiv_dev, while the REAL PRE-EXISTING `fee` table on a
-- second, untracked namespace on the same host (emittiv, no `app_state` -
-- clearly predates this migration chain) has kept working throughout,
-- with byte-identical `INFO FOR TABLE fee` DDL text in both places. This
-- is the SAME class of bug this repo already hit and documented for
-- `disciplines[*]`/`disciplines.*` in
-- `v006_backfill_metadata_fields_rollback.surql`'s header: SurrealDB
-- tracks an implicit wildcard-array element field (`x.*`, however it
-- prints) as something DISTINCT from what `INFO FOR TABLE` displays, and
-- `REMOVE FIELD` on either the dot-star or bracket-star spelling of that
-- wildcard silently matches NOTHING (`REMOVE FIELD IF EXISTS` returns OK
-- either way) - confirmed directly: after removing it, `INFO FOR TABLE
-- fee` still showed `revisions.*` defined, unchanged. So `OVERWRITE`-ing
-- `revisions` (or its wildcard) does not truly reset the write-validator's
-- internal state for that wildcard - it just leaves stale/corrupted state
-- behind while the displayed DDL text looks correct.
--
-- THE FIX THAT WORKED, verified live against dev (proved on a real
-- pre-existing fee row, `22_96601_1`: UPDATE started succeeding, backfill
-- then applied cleanly to all 46 dev fee rows, 0 errors): REMOVE the
-- PARENT `revisions` field (this DOES actually remove it - only the
-- wildcard sub-field spelling is un-removable), then DEFINE it fresh. That
-- alone made writes to `revisions[]` succeed again, on the SAME orphaned
-- wildcard sub-field the parent field is implicitly tied to - removing and
-- redefining only the parent is sufficient to un-stick it. This migration
-- therefore uses REMOVE + DEFINE for `revisions` (not OVERWRITE) as the
-- durable, safe-to-repeat pattern - safe on an already-corrupted target
-- (fixes it) AND on a pristine one (prod, presumed never OVERWRITE'd - the
-- remove+define round-trip reproduces the same working state). `rev` gets
-- the same defensive treatment since its VALUE clause depends on
-- `revisions.*`, even though it was never observed broken on its own.
--
-- `fee` is SCHEMALESS in practice (CLAUDE.md "Critical query patterns") -
-- these fields are individually defined and enforced regardless of
-- table-level schema mode, so no `DEFINE TABLE` clause is needed here
-- (unlike v006's `projects` migration, which needed one because `projects`
-- is SCHEMAFULL).
--
-- Run on: emittiv/projects DB, dev (10.0.23.12:8000, ns emittiv_dev) FIRST
-- - already applied and independently re-verified this session (0 of 46
-- fee rows left at rev=0 after the matching backfill ran) - then prod
-- (10.0.23.11:8000, ns emittiv) - orchestrator-owned per the guard's own
-- workflow (docs/development/DATABASE_SCHEMA.md "Migration workflow").
-- Prod has NOT been touched by any of this - its `revisions.*` field may
-- already be in the same never-corrupted state as the untracked `emittiv`
-- dev-host namespace (unconfirmed either way), but this migration's
-- remove+define approach is written to be correct regardless.

REMOVE FIELD IF EXISTS revisions ON fee;
DEFINE FIELD revisions ON fee TYPE array<object> DEFAULT []
    PERMISSIONS FOR select, create, update WHERE FULL;

-- Defensive only - REMOVE FIELD on this wildcard spelling is a confirmed
-- no-op (see ROOT-CAUSE above); IF NOT EXISTS makes this line a harmless
-- no-op everywhere it already exists, and a real definition on a genuinely
-- fresh environment where it doesn't.
DEFINE FIELD IF NOT EXISTS `revisions.*` ON fee TYPE object PERMISSIONS FULL;

REMOVE FIELD IF EXISTS rev ON fee;
DEFINE FIELD rev ON fee TYPE int
    VALUE (IF array::len(revisions.*.revision_number) > 0
           THEN math::max(revisions.*.revision_number)
           ELSE 0 END)
    PERMISSIONS FOR select WHERE FULL, FOR create, update FULL;

DEFINE INDEX OVERWRITE fee_project_rev ON fee FIELDS project_id, rev UNIQUE;

-- Schema-version stamp (guard requirement, migration_classify.py's
-- SCHEMA_VERSION_STAMP rule - structural template match, not a substring
-- search). Idempotent re-apply: whichever branch fires, the result is the
-- same row at value='7'.
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF array::len($existing) > 0 THEN
  (UPDATE $existing[0].id SET value = '7', updated_at = time::now())
ELSE
  (CREATE app_state SET key = 'schema_version', value = '7', updated_at = time::now())
END;

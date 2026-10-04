-- v009_scope_revision_unique.surql
-- Makes (fee_id, revision) UNIQUE on e-fees-scope's `scope_revision` table.
--
-- WHY: save_scope_builder used to DELETE + CREATE the `scope_assembly`
-- record, resetting `current_revision` to 0, so the next create_revision
-- re-issued revision numbers 1, 2, ... for a fee that already had them. The
-- index on (fee_id, revision) was non-unique, so the repeats were stored
-- silently. The application fix (e-fees-scope: update in place, never reset
-- current_revision, next number = max(counter, highest stored) + 1) stops
-- new duplicates; this migration repairs existing data and then enforces
-- uniqueness so a regression fails loudly instead of corrupting history.
--
-- MIGRATION SAFETY (existing duplicates): creating a UNIQUE index over rows
-- that already collide fails, so duplicates are renumbered FIRST,
-- deterministically, and only then is the index replaced:
--   * per fee, per revision number, the earliest row (saved_at, then record
--     id) KEEPS its number - history references stay valid;
--   * every later row with the same number is moved to (current highest
--     number for that fee) + 1, in (revision, saved_at, id) order;
--   * the original number is kept on the row in `renumbered_from`, and the
--     closing SELECT lists every renumbered row so the apply output is the
--     report;
--   * each `scope_assembly.current_revision` is raised to its fee's highest
--     stored revision when it was behind (never lowered);
--   * if any duplicate survives (e.g. a concurrent write), the index is not
--     replaced and schema_version is NOT stamped 9 (see the stamp below).
-- Idempotent: a second run finds no duplicates and changes nothing.
--
-- Run on: emittiv/projects DB (the e-fees-scope tables), dev
-- (10.0.23.12:8000, ns emittiv_dev) FIRST, then prod (10.0.23.11:8000,
-- ns emittiv) - orchestrator-owned via ~/.claude/scripts/apply-migration.sh
-- per docs/development/DATABASE_SCHEMA.md. NOT applied by the PR that adds
-- this file.

-- On a real database both tables already exist (SCHEMAFULL) and these two
-- statements do nothing. They exist for apply-migration.sh's rollback-proof,
-- which runs this file in a fresh namespace holding only app_state: without
-- them the SELECTs below fail with "table does not exist". They must stay
-- SCHEMALESS: an empty SCHEMAFULL table makes the DEFINE INDEX below fail
-- ("field fee_id does not exist").
DEFINE TABLE IF NOT EXISTS scope_assembly SCHEMALESS;
DEFINE TABLE IF NOT EXISTS scope_revision SCHEMALESS;

DEFINE FIELD OVERWRITE renumbered_from ON scope_revision TYPE option<int> PERMISSIONS FULL;

-- Renumber: one flat pass over every row in (fee, revision, saved_at, id)
-- order. A row is a duplicate when an earlier row of the same fee holds the
-- same number; the earliest row of each number is never touched, so it is
-- always found by the later ones even after they start moving away.
--
-- Engine notes (found against the embedded SurrealDB 3.0.4 engine, the
-- SDK version this repo pins; the dev/prod servers are 3.1.x): the loop is
-- deliberately single-level over a LET-bound list (an inline subquery in a
-- FOR header, or a FOR nested in a FOR with the inner list built from the
-- outer variable, mis-evaluates there), and the new number is computed into
-- a LET first (a subquery inside UPDATE ... SET sees the target row's own
-- `revision`).
LET $rows = (SELECT id, fee_id, revision, saved_at FROM scope_revision ORDER BY fee_id ASC, revision ASC, saved_at ASC, id ASC);
FOR $r IN $rows {
    LET $earlier = (SELECT VALUE id FROM scope_revision WHERE fee_id = $r.fee_id AND revision = $r.revision AND (saved_at < $r.saved_at OR (saved_at = $r.saved_at AND id < $r.id)));
    IF array::len($earlier) > 0 {
        LET $next = (SELECT VALUE revision FROM scope_revision WHERE fee_id = $r.fee_id ORDER BY revision DESC LIMIT 1)[0] + 1;
        UPDATE $r.id SET renumbered_from = $r.revision, revision = $next;
    };
};

LET $assemblies = (SELECT VALUE id FROM scope_assembly);
FOR $asm IN $assemblies {
    LET $fee = (SELECT VALUE fee_id FROM ONLY $asm);
    LET $cur = (SELECT VALUE current_revision FROM ONLY $asm);
    LET $top = (SELECT VALUE revision FROM scope_revision WHERE fee_id = $fee ORDER BY revision DESC LIMIT 1)[0] ?? 0;
    IF $top > $cur {
        UPDATE $asm SET current_revision = $top;
    };
};

LET $still_duplicated = (SELECT * FROM (SELECT fee_id, revision, count() AS n FROM scope_revision GROUP BY fee_id, revision) WHERE n > 1);
IF array::len($still_duplicated) > 0 {
    THROW "v009: duplicate (fee_id, revision) rows remain after renumbering - unique index NOT created";
};

DEFINE INDEX OVERWRITE idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision UNIQUE;

-- Schema-version stamp, conditional on the outcome. THROW does not halt a
-- script outside a transaction (later statements still run), and a
-- transaction cannot be used here (the unique index build does not see the
-- renumbering done earlier in the same transaction). So the stamp itself
-- checks that the index really is UNIQUE and refuses to claim version 9
-- otherwise. (Deliberately not the classifier's canonical stamp idiom:
-- this migration is NOT_ADDITIVE anyway.)
LET $idx = (INFO FOR TABLE scope_revision).indexes.idx_scope_rev_fee ?? '';
LET $existing = (SELECT id FROM app_state WHERE key = 'schema_version' LIMIT 1);
IF !string::contains($idx, 'UNIQUE') {
    THROW "v009: idx_scope_rev_fee is not UNIQUE - schema_version NOT stamped";
} ELSE IF array::len($existing) > 0 {
    UPDATE $existing[0].id SET value = '9', updated_at = time::now();
} ELSE {
    CREATE app_state SET key = 'schema_version', value = '9', updated_at = time::now();
};

-- Report: every row renumbered by this or an earlier run (empty if no fee ever had duplicates).
SELECT fee_id, renumbered_from, revision, saved_at FROM scope_revision WHERE renumbered_from != NONE ORDER BY fee_id, revision;

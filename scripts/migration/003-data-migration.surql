-- 003-data-migration.surql
-- Domain Model Restructure: Phase 2 — Migrate existing data to new status values
--
-- EXECUTION ORDER:
--   1. Remove existing ASSERT constraints (STEP 0 below)
--   2. Run STEP 1 + STEP 2 (the UPDATE statements)
--   3. Apply new ASSERT constraints (STEP 3 below)
--
-- PRE-MIGRATION COUNTS (verified 2026-02-25):
--   Projects: Active=2, Awarded=2, Cancelled=3, Completed=11, Lost=36, RFP=3, Revised=4
--   Fees:     Awarded=3, Completed=4, Lost=21, Revised=3
--
-- POST-MIGRATION COUNTS (verified 2026-02-25):
--   Projects: Design=2, Awarded=2, Cancelled=3, Completed=11, Lost=36, RFP=3, Superseded=4
--   Fees:     Accepted=7, No Response=21, Superseded=3

-- ============================================================================
-- STEP 0: Remove old ASSERT constraints to allow new status values
-- Must run BEFORE the UPDATE statements below
-- ============================================================================

DEFINE FIELD OVERWRITE status ON projects TYPE string
  PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE FIELD OVERWRITE status ON fee TYPE string DEFAULT 'Draft'
  PERMISSIONS FULL;

-- ============================================================================
-- STEP 1: Map project statuses to new lifecycle
-- ============================================================================

-- Draft → Lead (no Draft records exist, but handle for safety)
UPDATE projects SET status = 'Lead' WHERE status = 'Draft';

-- Active → Design (these are projects in active design work)
UPDATE projects SET status = 'Design' WHERE status = 'Active';

-- Revised → Superseded (confirmed: all 4 records are superseded engagements)
UPDATE projects SET status = 'Superseded' WHERE status = 'Revised';

-- ============================================================================
-- STEP 2: Map fee statuses to new lifecycle
-- ============================================================================

-- Awarded → Accepted
UPDATE fee SET status = 'Accepted' WHERE status = 'Awarded';

-- Completed → Accepted (fee was accepted; delivery tracked at project level)
UPDATE fee SET status = 'Accepted' WHERE status = 'Completed';

-- Revised → Superseded
UPDATE fee SET status = 'Superseded' WHERE status = 'Revised';

-- Active → Sent (no Active records exist, but handle for safety)
UPDATE fee SET status = 'Sent' WHERE status = 'Active';

-- On Hold → Draft (no On Hold records exist, but handle for safety)
UPDATE fee SET status = 'Draft' WHERE status = 'On Hold';

-- Cancelled → Rejected (no Cancelled records exist, but handle for safety)
UPDATE fee SET status = 'Rejected' WHERE status = 'Cancelled';

-- Lost → No Response (majority went cold; review individually post-migration)
UPDATE fee SET status = 'No Response' WHERE status = 'Lost';

-- ============================================================================
-- STEP 3: Apply new ASSERT constraints
-- Run AFTER all UPDATE statements above succeed
-- ============================================================================

DEFINE FIELD OVERWRITE status ON projects TYPE string
  ASSERT $value INSIDE [
    'Lead', 'RFP', 'Submitted', 'Awarded', 'Design', 'Construction', 'Completed',
    'Lost', 'No Response', 'Cancelled', 'On Hold', 'Superseded'
  ]
  PERMISSIONS FOR select, create, update WHERE FULL;

DEFINE FIELD OVERWRITE status ON fee TYPE string DEFAULT 'Draft'
  ASSERT $value INSIDE [
    'Draft', 'Sent', 'Negotiation', 'Accepted', 'Rejected',
    'No Response', 'Superseded'
  ]
  PERMISSIONS FULL;

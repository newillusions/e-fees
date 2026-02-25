-- 002-update-project-fee-schemas.surql
-- Domain Model Restructure: Phase 1 — Update project and fee schemas
--
-- EXECUTION ORDER:
--   1. Run ONLY the venue_id field definition (this task, Task 2)
--   2. Run data migration to remap statuses (Task 3)
--   3. THEN apply the status ASSERT changes below
--
-- Current project statuses in use: Active(2), Awarded(2), Cancelled(3),
--   Completed(11), Lost(36), Revised(4), RFP(3)
-- Current fee statuses in use: Awarded(3), Completed(4), Lost(21), Revised(3)

-- ============================================================================
-- STEP 1: Add venue_id to projects (safe — new optional field)
-- Execute immediately
-- ============================================================================

DEFINE FIELD venue_id ON projects TYPE option<record<venue>>
  PERMISSIONS FOR select, create, update WHERE FULL;

-- ============================================================================
-- STEP 2: Update project status ASSERT to new lifecycle
-- DO NOT EXECUTE until Task 3 data migration completes
-- Mapping: Active -> Design, Revised -> Superseded
-- New additions: Lead, Submitted, Design, Construction, No Response, Superseded
-- Removals: Draft, Active, Revised
-- ============================================================================

-- DEFINE FIELD status ON projects TYPE string
--   ASSERT $value INSIDE [
--     'Lead', 'RFP', 'Submitted', 'Awarded', 'Design', 'Construction', 'Completed',
--     'Lost', 'No Response', 'Cancelled', 'On Hold', 'Superseded'
--   ]
--   PERMISSIONS FOR select, create, update WHERE FULL;

-- ============================================================================
-- STEP 3: Update fee status ASSERT to new lifecycle
-- DO NOT EXECUTE until Task 3 data migration completes
-- Mapping: Awarded -> Accepted, Completed -> Accepted, Lost -> Rejected,
--          Revised -> Superseded
-- New additions: Accepted, Rejected, No Response, Superseded
-- Removals: Awarded, Completed, Lost, Cancelled, On Hold, Revised
-- ============================================================================

-- DEFINE FIELD status ON fee TYPE string DEFAULT 'Draft'
--   ASSERT $value INSIDE [
--     'Draft', 'Sent', 'Negotiation', 'Accepted', 'Rejected',
--     'No Response', 'Superseded'
--   ]
--   PERMISSIONS FULL;

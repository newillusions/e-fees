-- Migration Script: Update fee table status enum constraint
-- Remove 'Active', 'Under Review', and 'Clarification' from valid status values
-- 
-- This script:
-- 1. Updates any existing records with removed statuses to 'Draft'
-- 2. Updates the field definition to use the new enum constraint
-- 
-- Date: 2025-07-21

-- Step 1: Check for existing records with statuses that will be removed
SELECT count() as active_count FROM fee WHERE status = 'Active';
SELECT count() as under_review_count FROM fee WHERE status = 'Under Review';  
SELECT count() as clarification_count FROM fee WHERE status = 'Clarification';

-- Step 2: Update existing records with removed statuses to 'Draft'
-- (safer to change to Draft than to guess what the user intended)
UPDATE fee SET status = 'Draft' WHERE status = 'Active';
UPDATE fee SET status = 'Draft' WHERE status = 'Under Review';
UPDATE fee SET status = 'Draft' WHERE status = 'Clarification';

-- Step 3: Update the field definition with new enum constraint
DEFINE FIELD status ON fee TYPE string ASSERT $value INSIDE ['Draft', 'Prepared', 'Sent', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'] DEFAULT 'Draft';

-- Step 4: Verification - check that all records have valid statuses
SELECT status, count() as count FROM fee GROUP BY status ORDER BY status;

-- Step 5: Test the constraint by trying to insert an invalid status (should fail)
-- CREATE fee:test_invalid SET status = 'Active', name = 'Test';  -- This should fail
-- DELETE fee:test_invalid;  -- Clean up if it somehow succeeded
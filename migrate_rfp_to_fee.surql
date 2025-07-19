-- Migration Script: rfp table to fee table
-- This script will:
-- 1. Create the new fee table with the same schema as rfp
-- 2. Copy all data from rfp to fee with new ID format (fee: prefix)
-- 3. Verify the data was copied correctly
-- 4. Optionally remove the old rfp table

-- Step 1: Create the new fee table with the same schema
DEFINE TABLE fee SCHEMAFULL;

-- Copy the field definitions from the rfp table
DEFINE FIELD name ON fee TYPE string ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD number ON fee TYPE string ASSERT $value != NONE;
DEFINE FIELD project_id ON fee TYPE record<projects> ASSERT $value != NONE;
DEFINE FIELD company_id ON fee TYPE record<company> ASSERT $value != NONE;
DEFINE FIELD contact_id ON fee TYPE record<contacts> ASSERT $value != NONE;
DEFINE FIELD status ON fee TYPE string ASSERT $value INSIDE ['Draft', 'Active', 'Sent', 'Awarded', 'Lost', 'Cancelled'] DEFAULT 'Draft';
DEFINE FIELD stage ON fee TYPE string ASSERT $value INSIDE ['Draft', 'Prepared', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost'] DEFAULT 'Draft';
DEFINE FIELD issue_date ON fee TYPE string ASSERT $value != NONE;
DEFINE FIELD activity ON fee TYPE option<string>;
DEFINE FIELD package ON fee TYPE option<string>;
DEFINE FIELD strap_line ON fee TYPE option<string> DEFAULT 'sensory design studio';
DEFINE FIELD staff_name ON fee TYPE option<string>;
DEFINE FIELD staff_email ON fee TYPE option<string>;
DEFINE FIELD staff_phone ON fee TYPE option<string>;
DEFINE FIELD staff_position ON fee TYPE option<string>;
DEFINE FIELD rev ON fee TYPE int DEFAULT 1 VALUE math::max($value.revisions[*].revision_number);
DEFINE FIELD revisions ON fee TYPE array<object> DEFAULT [];
DEFINE FIELD time ON fee TYPE object VALUE { created_at: time::now(), updated_at: time::now() };

-- Create index on project + revision (same as rfp table)
DEFINE INDEX fee_project_rev ON fee FIELDS project_id, rev UNIQUE;

-- Step 2: Copy all data from rfp to fee
-- Note: We need to change the ID format from rfp:xxx to fee:xxx

-- Note: SurrealDB INSERT syntax for copying data
INSERT INTO fee SELECT * FROM rfp;

-- Step 3: Verify the migration
-- Count records in both tables
SELECT count() AS rfp_count FROM rfp;
SELECT count() AS fee_count FROM fee;

-- Sample comparison - show first few records from each table
SELECT * FROM rfp LIMIT 3;
SELECT * FROM fee LIMIT 3;

-- Step 4: Optional - Remove the old rfp table (UNCOMMENT WHEN READY)
-- WARNING: Only run this after verifying all data is correctly migrated!
-- REMOVE TABLE rfp;

-- Step 5: Show the new table info
INFO FOR TABLE fee;
# RFP Table to FEE Table Migration Plan

## Overview
Rename the `rfp` table to `fee` to better reflect that it contains fee proposals (responses to RFPs), not the actual RFP requests themselves.

**CRITICAL**: Keep original `rfp` table intact throughout the process for data verification and rollback capability.

## Migration Strategy

### Phase 1: Database Migration (Keep Original Table)

#### Step 1: Create New `fee` Table
- Copy exact structure from `rfp` table
- Include all field definitions, constraints, and default values
- Ensure identical schema except for table name

#### Step 2: Copy All Data
- Transfer all 30 RFP records from `rfp` to `fee` table
- Preserve all relationships and foreign keys
- Maintain exact data integrity

#### Step 3: Create Indexes and Constraints
- Replicate all indexes from `rfp` table to `fee` table:
  - `fee_company` (company_id)
  - `fee_contact` (contact_id) 
  - `fee_created` (time.created_at)
  - `fee_project` (project_id)
  - `fee_project_rev` (project_id, rev) UNIQUE
  - `fee_status` (status)

#### Step 4: Data Verification
- Compare record counts (should be identical: 30 records)
- Verify all field values match exactly
- Check foreign key relationships are preserved
- Validate indexes work correctly

### Phase 2: Backend Code Updates

#### Step 5: Update Rust Code (`src-tauri/src/db/mod.rs`)
**Current `rfp` references to change to `fee`:**
- Line ~700: `CREATE rfp SET` → `CREATE fee SET`
- Line ~740: `UPDATE rfp` → `UPDATE fee` 
- Line ~780: `DELETE rfp` → `DELETE fee`
- All SELECT queries: `FROM rfp` → `FROM fee`
- All indexes references: `rfp_*` → `fee_*`

#### Step 6: Update Command Functions
- `get_rfps()` → `get_fees()` or keep same name but use `fee` table
- `create_rfp()` → update to use `fee` table
- `update_rfp()` → update to use `fee` table
- `delete_rfp()` → update to use `fee` table

### Phase 3: Frontend Code Updates

#### Step 7: Update API Layer (`src/lib/api.ts`)
- Change API endpoints if needed
- Update error messages
- Ensure TypeScript types remain consistent

#### Step 8: Update Store References
- `rfpsStore` → can keep same name, just update underlying queries
- `rfpsActions` → update to use new backend endpoints

### Phase 4: Testing and Verification

#### Step 9: Comprehensive Testing
- Test RFP/Fee CRUD operations
- Verify filtering and search work correctly
- Test relationships with projects, companies, contacts
- Ensure UI displays correctly
- Test all detail views and modals

#### Step 10: Performance Verification
- Compare query performance between old and new tables
- Verify indexes are working correctly
- Test with realistic data loads

### Phase 5: Final Migration (Future)

#### Step 11: Production Deployment
- Deploy code changes to use `fee` table
- Monitor for any issues
- Keep `rfp` table as backup for 30 days

#### Step 12: Cleanup (After Verification Period)
- Archive `rfp` table data
- Remove `rfp` table (only after 100% confidence)

## Database Migration Scripts

### Script 1: Create Fee Table Structure
```sql
-- Create fee table with identical structure to rfp
CREATE TABLE fee AS SELECT * FROM rfp LIMIT 0;

-- Copy all field definitions
DEFINE FIELD activity ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD company_id ON fee TYPE record<company> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD contact_id ON fee TYPE record<contacts> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD issue_date ON fee TYPE string ASSERT string::len($value) = 6 AND string::is::numeric($value) PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON fee TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD number ON fee TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD package ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD project_id ON fee TYPE record<projects> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD rev ON fee TYPE int VALUE math::max(revisions[*].revision_number) OR 0 PERMISSIONS FOR select WHERE FULL, FOR create, update FULL;
DEFINE FIELD revisions ON fee TYPE array<object> DEFAULT [] PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD status ON fee TYPE string DEFAULT 'Draft' ASSERT $value INSIDE ['Draft', 'Prepared', 'Active', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'] PERMISSIONS FULL;
DEFINE FIELD staff_email ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_name ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_phone ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD staff_position ON fee TYPE string PERMISSIONS FULL;
DEFINE FIELD strap_line ON fee TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD time ON fee TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON fee TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON fee TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;
```

### Script 2: Copy Data
```sql
-- Copy all data from rfp to fee
INSERT INTO fee SELECT * FROM rfp;
```

### Script 3: Create Indexes
```sql
-- Create all indexes on fee table
DEFINE INDEX fee_company ON fee FIELDS company_id;
DEFINE INDEX fee_contact ON fee FIELDS contact_id;
DEFINE INDEX fee_created ON fee FIELDS time.created_at;
DEFINE INDEX fee_project ON fee FIELDS project_id;
DEFINE INDEX fee_project_rev ON fee FIELDS project_id, rev UNIQUE;
DEFINE INDEX fee_status ON fee FIELDS status;
```

### Script 4: Verification Queries
```sql
-- Compare record counts
SELECT count() FROM rfp;
SELECT count() FROM fee;

-- Compare sample data
SELECT id, name, number, status FROM rfp ORDER BY time.created_at LIMIT 5;
SELECT id, name, number, status FROM fee ORDER BY time.created_at LIMIT 5;

-- Verify specific records match
SELECT * FROM rfp WHERE number = '22-96601-FP';
SELECT * FROM fee WHERE number = '22-96601-FP';
```

## Rollback Plan

If issues are discovered:
1. **Code rollback**: Deploy previous version using `rfp` table
2. **Data verification**: Original `rfp` table remains untouched
3. **No data loss**: All original data preserved in `rfp` table

## Benefits of This Approach

1. **Zero Risk**: Original table remains completely intact
2. **Verification**: Can compare data side-by-side during entire process
3. **Easy Rollback**: Simply switch code back to use `rfp` table
4. **Better Naming**: `fee` table name accurately reflects content (fee proposals)
5. **Clean Migration**: No ALTER table operations that could cause issues

## File Changes Required

### Backend Files:
- `src-tauri/src/db/mod.rs` - Update all SQL queries
- `src-tauri/src/commands/mod.rs` - Update command implementations

### Frontend Files:
- `src/types/index.ts` - Update if interface names change
- `src/lib/api.ts` - Update API calls if needed
- `src/lib/stores.ts` - Update store queries

### Documentation:
- `DATABASE_SCHEMA.md` - Update to reflect new table name
- `CLAUDE.md` - Update references from RFP to Fee table

## Timeline

- **Phase 1 (Database)**: 1-2 hours
- **Phase 2 (Backend)**: 2-3 hours  
- **Phase 3 (Frontend)**: 1-2 hours
- **Phase 4 (Testing)**: 2-3 hours
- **Total**: ~6-10 hours with thorough testing

## Success Criteria

1. ✅ `fee` table created with identical structure
2. ✅ All 30 records copied with 100% data integrity
3. ✅ All indexes working correctly
4. ✅ All CRUD operations functional
5. ✅ UI displays correctly without errors
6. ✅ Foreign key relationships preserved
7. ✅ Original `rfp` table untouched as backup

---

**Status**: Ready to proceed with database migration
**Next Step**: Create `fee` table structure and copy data
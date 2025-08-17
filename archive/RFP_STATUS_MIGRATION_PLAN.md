# RFP Status Field Migration Plan

## Overview
This document outlines the plan to merge the `stage` field into the `status` field for the RFP table, eliminating redundancy while preserving all unique values and ensuring no data loss.

## Current State Analysis

### Status Field (6 values)
- Draft
- Active  
- Sent
- Awarded
- Lost
- Cancelled

### Stage Field (8 values)
- Draft *(duplicate)*
- Prepared
- Sent *(duplicate)*
- Under Review
- Clarification
- Negotiation
- Awarded *(duplicate)*
- Lost *(duplicate)*

### Overlapping Values
- Draft, Sent, Awarded, Lost appear in both fields
- These represent 50% of status values and 50% of stage values

## Proposed Merged Status Field

### New Consolidated Values (10 unique statuses)
1. **Draft** - Initial state, proposal being created
2. **Prepared** - Proposal completed, ready to send
3. **Active** - General active state (when specific stage unknown)
4. **Sent** - Proposal sent to client
5. **Under Review** - Client reviewing proposal
6. **Clarification** - Client requested clarifications
7. **Negotiation** - In negotiation phase
8. **Awarded** - Project awarded
9. **Lost** - Project not awarded
10. **Cancelled** - Proposal cancelled

## Migration Steps

### 1. Database Schema Update
```sql
-- Step 1: Update the status field constraint to include all values
ALTER TABLE rfp 
MODIFY FIELD status TYPE string 
ASSERT $value INSIDE ['Draft', 'Prepared', 'Active', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost', 'Cancelled'] 
DEFAULT 'Draft';
```

### 2. Data Migration Query
```sql
-- Step 2: Migrate stage values to status where they provide more detail
UPDATE rfp SET status = stage 
WHERE stage IN ['Prepared', 'Under Review', 'Clarification', 'Negotiation'] 
AND status IN ['Active', 'Sent'];

-- Step 3: Handle edge cases where stage and status conflict
-- If stage is more specific than status, use stage
UPDATE rfp SET status = stage
WHERE (status = 'Draft' AND stage != 'Draft')
   OR (status = 'Active' AND stage NOT IN ['Draft', 'Active'])
   OR (status = 'Sent' AND stage IN ['Under Review', 'Clarification', 'Negotiation']);
```

### 3. Remove Stage Field
```sql
-- Step 4: Remove the stage field after migration
REMOVE FIELD stage ON rfp;
```

## Code Changes Required

### 1. TypeScript Types (`src/types/index.ts`)
```typescript
// Update Rfp interface
export interface Rfp {
  // ... other fields
  status: 'Draft' | 'Prepared' | 'Active' | 'Sent' | 'Under Review' | 'Clarification' | 'Negotiation' | 'Awarded' | 'Lost' | 'Cancelled';
  // stage: remove this line
  // ... rest of fields
}
```

### 2. Rust Structs (`src-tauri/src/db/mod.rs`)
- Update `Rfp`, `RfpCreate`, and `RfpUpdate` structs to remove stage field
- Update status field comment to include all new values

### 3. Frontend Components
- **RfpModal.svelte**: Remove stage dropdown, update status options
- **Proposals.svelte**: Remove stage filter
- **ProposalDetail.svelte**: Remove stage display
- **StatusBadge.svelte**: Add styling for new status values

### 4. API Layer (`src/lib/api.ts`)
- Remove stage from RFP creation/update payloads

## Implementation Order

1. **Phase 1: Add New Status Values**
   - Update database schema to accept new status values
   - Deploy schema change without removing stage yet

2. **Phase 2: Update Application Code**
   - Update TypeScript types
   - Update Rust structs
   - Update UI components
   - Deploy application updates

3. **Phase 3: Migrate Data**
   - Run migration queries to consolidate fields
   - Verify data integrity

4. **Phase 4: Remove Stage Field**
   - Remove stage field from database
   - Final cleanup of any remaining references

## Rollback Plan

If issues arise:
1. Keep backup of original data
2. Stage field can be restored from backup
3. Application can be rolled back to previous version

## Testing Checklist

- [ ] All existing RFPs display correct status after migration
- [ ] New RFPs can be created without stage field
- [ ] Status filtering works with all new values
- [ ] Status badge displays correctly for all values
- [ ] No errors in console or logs
- [ ] Data export includes only status field

## Status Value Mapping Logic

### Detailed Migration Rules
| Current Status | Current Stage | New Status | Notes |
|---------------|---------------|------------|-------|
| Draft | Draft | Draft | No change |
| Draft | Prepared | Prepared | Use more specific stage |
| Draft | * | Use stage value | Stage is more specific |
| Active | Draft | Active | Keep status |
| Active | Prepared | Prepared | Use more specific stage |
| Active | Sent | Sent | Use more specific stage |
| Active | Under Review | Under Review | Use more specific stage |
| Active | * | Use stage value | Stage is more specific |
| Sent | Sent | Sent | No change |
| Sent | Under Review | Under Review | Natural progression |
| Sent | Clarification | Clarification | Natural progression |
| Sent | Negotiation | Negotiation | Natural progression |
| * | * | Use stage if not Draft | Default to stage when more specific |

## Benefits

1. **Eliminates Redundancy**: No more confusion between status and stage
2. **Preserves Information**: All unique states are maintained
3. **Simplifies UI**: One field instead of two
4. **Maintains History**: All current states map logically to new structure
5. **Future-Proof**: Can add new statuses as needed

## Notes

- The 'Active' status is retained as a general state when specific stage is unknown
- Migration prioritizes the more specific value when status and stage differ
- All 10 unique values across both fields are preserved
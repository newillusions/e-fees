# RFP Stage Field Migration - Implementation Checklist

## Phase 1: Database Preparation
- [ ] Backup current RFP data
- [ ] Test migration script on development database
- [ ] Run migration script sections 1-3 (update schema, migrate data)
- [ ] Verify data integrity with verification queries

## Phase 2: Backend Code Updates

### Rust Code Updates (`src-tauri/src/db/mod.rs`)
- [ ] Line 410: Remove `stage: String,` from `RfpCreate` struct
- [ ] Line 431: Remove `stage: String,` from `RfpUpdate` struct  
- [ ] Line 453: Remove `stage: String,` from `Rfp` struct
- [ ] Line 453: Update comment to list all 10 status values
- [ ] Line 704-705: Remove stage from CREATE query
- [ ] Line 742-743: Remove stage from UPDATE query

### Rust Commands (`src-tauri/src/commands/mod.rs`)
- [ ] Check and remove any stage field references

## Phase 3: Frontend Code Updates

### TypeScript Types (`src/types/index.ts`)
- [ ] Line 83: Remove `stage` field from `Rfp` interface
- [ ] Line 82: Update status type to include all 10 values:
  ```typescript
  status: 'Draft' | 'Prepared' | 'Active' | 'Sent' | 'Under Review' | 'Clarification' | 'Negotiation' | 'Awarded' | 'Lost' | 'Cancelled';
  ```

### API Layer (`src/lib/api.ts`)
- [ ] Remove stage from RFP creation payload
- [ ] Remove stage from RFP update payload

### Component Updates

#### RfpModal.svelte
- [ ] Line 22: Remove `stage: 'Draft' as const,` from formData
- [ ] Line 59: Remove `const stageOptions = [...]`
- [ ] Line 58: Update statusOptions to include all 10 values
- [ ] Line 231: Remove `stage: rfp.stage || 'Draft',`
- [ ] Line 268: Remove `stage: 'Draft',`
- [ ] Line 378: Remove `stage: formData.stage,`
- [ ] Lines 681-731: Remove entire Stage dropdown div
- [ ] Update grid from `grid-cols-3` to `grid-cols-2` for Rev/Status row

#### Proposals.svelte  
- [ ] Line 22: Remove `stage: '',` from filters
- [ ] Line 31: Remove `stage: (proposal) => proposal.stage,` from filterConfig
- [ ] Line 55: Remove `const uniqueStages = ...`
- [ ] Lines 356-364: Remove entire Stage filter dropdown

#### ProposalDetail.svelte
- [ ] Line 130: Remove stage from stats calculation
- [ ] Line 144: Remove stage from details fields

#### StatusBadge.svelte
- [ ] Add styling for new status values in proposal case:
  ```typescript
  case 'Prepared':
    return `${baseClasses} bg-purple-500/10 text-purple-400`;
  case 'Under Review':
    return `${baseClasses} bg-indigo-500/10 text-indigo-400`;
  case 'Clarification':
    return `${baseClasses} bg-orange-500/10 text-orange-400`;
  case 'Negotiation':
    return `${baseClasses} bg-yellow-500/10 text-yellow-400`;
  case 'Active':
    return `${baseClasses} bg-cyan-500/10 text-cyan-400`;
  case 'Cancelled':
    return `${baseClasses} bg-red-500/10 text-red-400`;
  ```

#### Other Components
- [ ] ContactDetail.svelte (Line 241): Remove stage display
- [ ] CompanyDetail.svelte (Line 250): Remove stage display  
- [ ] ProjectDetail.svelte (Line 203): Remove stage display
- [ ] ProposalModal.svelte: Remove all stage references (if still exists)

## Phase 4: Final Database Update
- [ ] Run migration script section 5 (remove stage field)
- [ ] Verify all functionality works without stage field

## Phase 5: Testing
- [ ] Create new RFP - verify no stage field
- [ ] Edit existing RFP - verify status shows correctly
- [ ] Filter by all new status values
- [ ] Check status badge colors for all values
- [ ] Export RFP data - verify no stage column
- [ ] Detail views show only status, not stage

## Phase 6: Documentation Updates
- [ ] Update DATABASE_SCHEMA.md to remove stage field
- [ ] Update CLAUDE.md if it mentions RFP fields
- [ ] Update any API documentation
- [ ] Archive migration plan and checklist

## Rollback Preparation
- [ ] Keep database backup for 30 days
- [ ] Tag current git commit before changes
- [ ] Document rollback SQL if needed

## Notes
- Total estimated changes: ~25 code locations
- Migration affects 37 existing RFP records
- No changes needed to project, company, or contact tables
- UI will be cleaner with single status field
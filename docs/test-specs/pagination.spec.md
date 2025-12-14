# Test Specification: Pagination / Lazy Loading

**Feature**: Paginated data loading with background fetching
**Date**: December 8, 2025
**Status**: Specification Complete - Ready for Test Implementation

---

## Overview

This document specifies the test requirements for implementing pagination and lazy loading in the E-Fees application. The feature addresses critical scalability issues identified in SITREP_2025-12-06.md where the current `loadAllData()` pattern loads all records at once, causing potential memory exhaustion with large datasets.

**Feature Context:**
- Current behavior: All records loaded simultaneously via `loadAllData()` in `src/lib/stores.ts:318`
- Target behavior: Initial load of 50 records with incremental background/scroll loading
- Affected entities: Projects, Companies, Contacts, Fees (Proposals)
- Related records: Fee → Company, Fee → Project, Contact → Company lookups

**Technical Stack:**
- Frontend: Svelte 5 stores with CRUD utilities (`src/lib/utils/crud.ts`)
- Backend: Rust/Tauri commands (`src-tauri/src/db/mod.rs`)
- Database: SurrealDB with WebSocket connection
- Existing patterns: `CrudApi<T>` interface, `PaginatedResult<T>` type

---

## Requirements Summary

| ID | Requirement | Priority |
|----|-------------|----------|
| REQ-01 | Default page size of 50 records | Critical |
| REQ-02 | Background OR scroll-triggered loading for remaining records | Critical |
| REQ-03 | On-demand loading of related records not yet in memory | High |
| REQ-04 | Append new data WITHOUT duplicates | Critical |
| REQ-05 | Maintain sort order when appending | Critical |
| REQ-06 | Memory efficient - don't reload already-loaded records | High |

---

## Test Cases

### TC-001: Initial Page Load

**Category**: Integration
**Priority**: Critical

**Preconditions**:
- Database contains 150+ records for entity type being tested
- Application is freshly started or store is cleared

**Input**:
- Navigate to entity list page (Projects/Companies/Contacts/Proposals)

**Expected Output**:
- Store contains exactly 50 records (or total count if < 50)
- Loading indicator shown during fetch, hidden after
- Records displayed in default sort order
- Total count indicator shows "50 of N" where N is total records

**Assertions**:
- [ ] `store.items.length === 50` (or total if less)
- [ ] `store.pagination.hasMore === true`
- [ ] `store.pagination.currentPage === 1`
- [ ] Items sorted by default sort field (time.created_at DESC)
- [ ] Loading state transitions: false → true → false

---

### TC-002: Background Loading - Automatic Fetch

**Category**: Integration
**Priority**: Critical

**Preconditions**:
- Database contains 200 records
- Initial 50 records have loaded successfully

**Input**:
- Complete initial load, wait for background loader (2 second delay)

**Expected Output**:
- Additional records fetched in batches of 50
- Store grows: 50 → 100 → 150 → 200
- UI remains responsive during background loading

**Assertions**:
- [ ] After background load completes, `store.items.length === 200`
- [ ] No duplicate records (unique IDs)
- [ ] Sort order maintained throughout
- [ ] UI frame rate > 30fps during loading

---

### TC-003: Scroll-Triggered Loading (Infinite Scroll)

**Category**: Integration
**Priority**: Critical

**Preconditions**:
- Database contains 200 records
- Initial 50 records loaded
- Scroll container is scrollable

**Input**:
- Scroll to 80% of container height (threshold)

**Expected Output**:
- Page 2 (records 51-100) loads when scroll threshold reached
- Loading spinner appears at bottom of list
- New records appended seamlessly

**Assertions**:
- [ ] `store.items.length === 100` after first scroll trigger
- [ ] Scroll position maintained (no jump)
- [ ] Loading indicator visible during fetch
- [ ] `hasMore === false` when all loaded

---

### TC-004: Duplicate Prevention During Pagination

**Category**: Unit
**Priority**: Critical

**Preconditions**:
- Store has 50 records with IDs 1-50
- Server returns records with IDs 25-75 (overlapping)

**Input**:
- Append page 2 response to store

**Expected Output**:
- Store contains 75 unique records (not 100)
- Records 25-50 not duplicated

**Assertions**:
- [ ] `store.items.length === 75`
- [ ] `new Set(store.items.map(i => extractSurrealId(i.id))).size === 75`
- [ ] Each ID appears exactly once

---

### TC-005: Sort Order Maintenance

**Category**: Unit
**Priority**: Critical

**Preconditions**:
- 150 records exist
- Default sort by `time.created_at DESC`

**Input**:
- Load page 1 (newest 50)
- Load page 2 (next 50)
- Load page 3 (oldest 50)

**Expected Output**:
- All 150 records in correct chronological order (newest first)

**Assertions**:
- [ ] `store.items[0].time.created_at > store.items[49].time.created_at`
- [ ] `store.items[49].time.created_at > store.items[99].time.created_at`
- [ ] Sort is stable (equal timestamps maintain relative order)

---

### TC-006: On-Demand Related Record Loading - Fee to Company

**Category**: Integration
**Priority**: Critical

**Preconditions**:
- Fee record references `company:XYZ`
- `company:XYZ` NOT in companies store

**Input**:
- User clicks on fee record to view details

**Expected Output**:
- System detects Company XYZ not in memory
- Company XYZ fetched on-demand
- Fee displays with company name (not raw ID)

**Assertions**:
- [ ] Single fetch request made for company
- [ ] Company added to `companiesStore`
- [ ] Fee modal shows `company.name` not `company_id`
- [ ] Subsequent views use cached company (no re-fetch)

---

### TC-007: On-Demand Loading - Contact to Company

**Category**: Integration
**Priority**: High

**Preconditions**:
- Contact references company not yet loaded

**Input**:
- Open ContactModal for contact

**Expected Output**:
- Company information displayed (not raw ID)

**Assertions**:
- [ ] Company name resolved in modal
- [ ] Company dropdown populated
- [ ] No error if company fetch delayed

---

### TC-008: Memory Efficiency - No Reload

**Category**: Unit
**Priority**: High

**Preconditions**:
- 50 records loaded
- Network monitoring active

**Input**:
- Navigate away from page
- Navigate back

**Expected Output**:
- No network request for already-loaded data
- Store retains data across navigation

**Assertions**:
- [ ] Zero fetch requests on return navigation
- [ ] `store.items.length` unchanged
- [ ] Only fetch if staleness threshold exceeded

---

### TC-009: Concurrent Load Prevention

**Category**: Unit
**Priority**: High

**Preconditions**:
- Page load in progress (`isLoadingData === true`)

**Input**:
- Trigger another page load (rapid scroll)

**Expected Output**:
- Second load request ignored/queued
- No duplicate data

**Assertions**:
- [ ] Only one active request at a time
- [ ] `isLoadingData` flag prevents concurrent calls
- [ ] No race conditions

---

### TC-010: Projects List Pagination

**Category**: E2E
**Priority**: Critical

**Preconditions**:
- 150 projects in database (PAGTEST_ prefixed)

**Test Steps**:
1. Navigate to Projects page (`/#/projects`)
2. Verify initial 50 projects loaded
3. Trigger pagination
4. Verify projects 51-100 appended

**Assertions**:
- [ ] Project number format (YY-CCCNN) displayed correctly
- [ ] Sorting by project number works
- [ ] Search filters paginated results

---

### TC-011: Companies List Pagination

**Category**: E2E
**Priority**: Critical

**Preconditions**:
- 100 companies in database (PAGTEST_ prefixed)

**Test Steps**:
1. Navigate to Companies page (`/#/companies`)
2. Load initial 50
3. Paginate to load remaining

**Assertions**:
- [ ] Companies with contact counts display correctly
- [ ] `companiesWithContactsStore` works with paginated data

---

### TC-012: Contacts List Pagination

**Category**: E2E
**Priority**: Critical

**Preconditions**:
- 200 contacts in database (PAGTEST_ prefixed)

**Test Steps**:
1. Navigate to Contacts page (`/#/contacts`)
2. Load initial 50
3. Verify company names resolved
4. Paginate to load more

**Assertions**:
- [ ] Contact `full_name` displayed (computed field)
- [ ] Company name resolved (not company ID)
- [ ] On-demand company loading works

---

### TC-013: Proposals/Fees List Pagination

**Category**: E2E
**Priority**: Critical

**Preconditions**:
- 150 fees exist with relationships (PAGTEST_ prefixed)

**Test Steps**:
1. Navigate to Proposals page (`/#/proposals`)
2. Load initial 50 fees
3. Verify project, company, contact names displayed
4. Paginate to load more

**Assertions**:
- [ ] All three relationships resolved
- [ ] On-demand loading for missing related records
- [ ] Fee status filter works with pagination

---

## Edge Cases

### EC-001: Records Added During Pagination

**Scenario**: New record created while pagination in progress
**Expected Behavior**:
- Dedup by ID prevents duplicates
- UI indicates "new data available" if total count changed
- Refresh reloads from page 1

---

### EC-002: Records Deleted During Pagination

**Scenario**: Record deleted between page loads
**Expected Behavior**:
- Total count updates after each page load
- No errors if referencing deleted record
- Use cursor-based pagination for consistency (future enhancement)

---

### EC-003: Concurrent Load Requests

**Scenario**: Rapid scrolling triggers multiple load requests
**Expected Behavior**:
- Request deduplication (one active request per page)
- Use `isLoadingData` flag (existing at line 315)
- Queue subsequent requests

---

### EC-004: Exact Page Size Multiple

**Scenario**: Database has exactly 100 records, page size 50
**Expected Behavior**:
- Page 2 returns exactly 50 records
- No empty page 3 request
- `hasMore === false` after page 2

---

### EC-005: Empty Page Response

**Scenario**: All records deleted between page loads
**Expected Behavior**:
- Graceful handling of empty array
- "No more records" indicator
- No infinite loop

---

### EC-006: Filter Changes Mid-Pagination

**Scenario**: User applies filter after loading partial data
**Expected Behavior**:
- Reset to page 1 when filter applied
- Server-side filtering for consistent results
- Clear previously loaded data on filter change

---

### EC-007: Sort Changes Mid-Pagination

**Scenario**: User changes sort order after partial load
**Expected Behavior**:
- Reset pagination state
- Re-fetch from page 1 with new sort
- Show loading indicator

---

### EC-008: Related Record Not Found

**Scenario**: Fee references deleted company
**Expected Behavior**:
- Show placeholder: "[Deleted]" or "Company not found"
- Log warning but don't crash
- Allow editing to select different company

---

## Error Cases

### ERR-001: Network Failure During Pagination

**Trigger**: Network disconnects during page fetch
**Expected Error**: "Failed to load more data"
**Expected Behavior**:
- Retry button available
- Already-loaded data preserved
- Loading state reset

---

### ERR-002: Database Connection Lost

**Trigger**: SurrealDB connection drops
**Expected Error**: "Database connection lost"
**Expected Behavior**:
- ConnectionStatus shows disconnected
- Auto-retry when connection restored
- No data loss in store

---

### ERR-003: Invalid Page Request

**Trigger**: Request page 100 when only 50 records exist
**Expected Error**: None (empty response)
**Expected Behavior**:
- Return empty array
- Set `hasMore = false`
- Log info message

---

### ERR-004: Malformed Response

**Trigger**: Backend returns invalid JSON
**Expected Error**: "Invalid response from server"
**Expected Behavior**:
- Graceful error handling
- Existing data preserved
- Retry available

---

## Data Requirements

### Test Data Pattern

All test data MUST include identifier for cleanup:

```typescript
const TEST_PREFIX = 'PAGTEST_';

// Projects
const testProject = {
  name: `${TEST_PREFIX}Project_${timestamp}`,
  name_short: `PT${sequence}`,
  status: 'Draft',
  // DELETE ME marker in name for cleanup
};

// Companies
const testCompany = {
  name: `${TEST_PREFIX}Company_${timestamp}`,
  abbreviation: `PT${sequence}`,
};

// Contacts
const testContact = {
  first_name: TEST_PREFIX,
  last_name: `Contact_${timestamp}`,
  email: `${TEST_PREFIX.toLowerCase()}${timestamp}@test.local`,
};

// Fees
const testFee = {
  name: `${TEST_PREFIX}Fee_${timestamp}`,
  status: 'Draft',
};
```

### Cleanup Query

```sql
-- Run after tests complete
DELETE FROM projects WHERE name STARTS WITH 'PAGTEST_';
DELETE FROM company WHERE name STARTS WITH 'PAGTEST_';
DELETE FROM contacts WHERE first_name = 'PAGTEST_';
DELETE FROM fee WHERE name STARTS WITH 'PAGTEST_';
```

---

## Acceptance Criteria

### Functional
- [ ] TC-001 through TC-013 all pass
- [ ] All edge cases (EC-001 to EC-008) handled gracefully
- [ ] All error cases (ERR-001 to ERR-004) return appropriate errors

### Performance
- [ ] Initial load < 500ms for 50 records
- [ ] Page load < 300ms per page
- [ ] Related record fetch < 200ms
- [ ] No memory leaks after 10 page loads
- [ ] UI FPS > 30 during background loading

### Data Integrity
- [ ] Zero duplicate records ever
- [ ] Sort order always maintained
- [ ] All test data cleaned up after test run

---

## Implementation Files

### Backend (Rust/Tauri)
- `src-tauri/src/db/mod.rs` - Add paginated query methods
- `src-tauri/src/commands/mod.rs` - Add Tauri commands for pagination

### Frontend (Svelte/TypeScript)
- `src/lib/stores.ts` - Add pagination state management
- `src/lib/utils/crud.ts` - Extend with pagination support
- `src/lib/api.ts` - Add paginated API methods

### New Types Required
```typescript
interface PaginationState {
  currentPage: number;
  pageSize: number;
  totalRecords: number;
  hasMore: boolean;
  loadedIds: Set<string>;
  isLoading: boolean;
}

interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}
```

---

**Specification Version**: 1.0
**Author**: Test Planner Agent
**Next Step**: Implement tests from this specification (testing-specialist)

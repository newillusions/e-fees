# Pagination UI Integration Test Specification

## Overview

This specification defines tests for integrating pagination/lazy loading into the UI components. Tests follow TDD methodology - written FIRST, expected to FAIL until implementation is complete.

## Test Categories

### TC-UI-001: Projects Page Pagination Integration

**Preconditions:**
- Database contains 150+ projects
- User navigates to Projects page

**Test Steps:**
1. Load Projects page
2. Verify initial load fetches only first 50 records
3. Verify loading indicator shown during initial load
4. Verify paginated store is used (not legacy store)

**Expected Results:**
- Only 50 projects rendered initially
- Loading spinner visible during fetch
- `paginatedProjectsStore` used instead of `projectsStore`
- Total count displayed correctly (e.g., "Showing 50 of 150")

### TC-UI-002: Infinite Scroll Trigger

**Preconditions:**
- Projects page loaded with 50+ more records available
- User has scrolled list

**Test Steps:**
1. Scroll project list to 80% threshold
2. Verify next page fetch triggered
3. Verify loading indicator at bottom of list
4. Verify new records appended

**Expected Results:**
- API call made when scroll threshold reached
- Loading indicator shown at list bottom
- New records appear below existing
- No duplicate records
- Scroll position maintained

### TC-UI-003: Loading States

**Test Cases:**

#### TC-UI-003a: Initial Load Skeleton
- Show skeleton/placeholder during first page load
- Hide skeleton when data arrives

#### TC-UI-003b: Next Page Loading
- Show "Loading more..." indicator at list bottom
- Indicator disappears when new data loaded

#### TC-UI-003c: All Data Loaded
- Show "All projects loaded" when hasMore=false
- Remove infinite scroll listener

### TC-UI-004: Related Record Resolution

**Preconditions:**
- Fee proposal list loaded
- Fee references company not in paginated companies store

**Test Steps:**
1. Display fee in list
2. Verify company name displayed (not raw ID)
3. Verify on-demand fetch triggered for missing company

**Expected Results:**
- Company name displayed correctly
- Single API call for company data
- Company cached for subsequent views

### TC-UI-005: Filter Interaction with Pagination

**Test Steps:**
1. Load page with pagination
2. Apply search filter
3. Verify pagination resets
4. Verify filtered results paginated correctly

**Expected Results:**
- Filter change resets to page 1
- Filtered results respect pagination
- Previous loaded data cleared on filter change

### TC-UI-006: Error Handling

**Test Cases:**

#### TC-UI-006a: Initial Load Failure
- Show error message on API failure
- Provide retry button

#### TC-UI-006b: Next Page Failure
- Preserve existing data
- Show error toast
- Allow retry

### TC-UI-007: Empty States

**Test Cases:**

#### TC-UI-007a: No Data
- Show "No projects yet" empty state
- Display create action

#### TC-UI-007b: No Filtered Results
- Show "No results found" message
- Maintain pagination state

## Performance Requirements

- Initial page load: < 500ms perceived
- Next page load: < 300ms
- Scroll detection: No jank (60fps maintained)
- Memory: No accumulation of duplicate records

## Test Data Requirements

All test data MUST use `PAGTEST_` prefix for safe identification and cleanup.

## Implementation Checklist

### Component Changes Required:

1. **Projects.svelte**
   - [ ] Import `paginatedProjectsStore` instead of `projectsStore`
   - [ ] Add scroll container with `createScrollTrigger()`
   - [ ] Add loading indicator for initial load
   - [ ] Add "loading more" indicator at bottom
   - [ ] Update ResultsCounter to show pagination info

2. **Companies.svelte**
   - [ ] Same pagination integration
   - [ ] On-demand contact count resolution

3. **Contacts.svelte**
   - [ ] Same pagination integration
   - [ ] On-demand company name resolution

4. **Proposals.svelte**
   - [ ] Same pagination integration
   - [ ] On-demand project/company/contact resolution

### New Components Required:

1. **PaginatedList.svelte** (optional wrapper)
   - Handles scroll detection
   - Manages loading states
   - Provides slot for list items

2. **LoadingMore.svelte**
   - Bottom-of-list loading indicator
   - "All loaded" state

## Files to Create

```
src/routes/Projects.test.ts           # Component tests
src/routes/Companies.test.ts          # Component tests
src/routes/Contacts.test.ts           # Component tests
src/routes/Proposals.test.ts          # Component tests
src/lib/components/PaginatedList.test.ts  # Wrapper component tests
```

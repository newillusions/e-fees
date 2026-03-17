# God Function Splits — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split 3 oversized functions into focused, testable modules. Reduce `crud.ts` from 1,165 lines to ~150 + 4 focused files. Extract search providers and project number parsing as pure functions.

**Architecture:** Extract pure logic into separate files, keep orchestrators thin. Follow existing patterns in `src/lib/utils/`.

**Tech Stack:** TypeScript, Svelte 5, Vitest

---

## Pre-Flight

- [ ] `npm test` — 633/633 pass
- [ ] `npm run check` — 0 errors

---

## Task 1: Extract search providers from GlobalSearchModal

**Files:**
- Create: `src/lib/utils/searchProviders.ts`
- Create: `src/lib/utils/__tests__/searchProviders.test.ts`
- Modify: `src/lib/components/GlobalSearchModal.svelte`

### Step 1: Write failing tests

- [ ] Create `src/lib/utils/__tests__/searchProviders.test.ts` with tests for:
  - `searchProjects()` — matches on name, project_number; respects MAX limit; returns `SearchResult[]`
  - `searchCompanies()` — matches on name, country; respects MAX
  - `searchContacts()` — matches on name, email, company name via lookup
  - `searchFees()` — matches on description, company name, project name via lookups; sorted by date desc

- [ ] Run tests — verify they FAIL (module doesn't exist yet)

### Step 2: Implement search providers

- [ ] Create `src/lib/utils/searchProviders.ts` with 4 exported functions:

```typescript
export function searchProjects(projects: Project[], query: string, max: number): SearchResult[]
export function searchCompanies(companies: Company[], query: string, max: number): SearchResult[]
export function searchContacts(contacts: Contact[], companyLookup: Map<string, string>, query: string, max: number): SearchResult[]
export function searchFees(fees: Fee[], companyLookup: Map<string, string>, projectLookup: Map<string, string>, query: string, max: number): SearchResult[]
```

- [ ] Extract the filter+map logic from `performSearch()` lines 84–176 into these functions
- [ ] Run tests — verify they PASS

### Step 3: Refactor GlobalSearchModal

- [ ] Replace the 4 inline search blocks in `performSearch()` with calls to the extracted functions
- [ ] `performSearch()` becomes: build lookups → call 4 providers → concat results (~15 lines)
- [ ] Run `npm test` — all 633+ pass
- [ ] Commit: `refactor(ui): extract search providers from GlobalSearchModal`

---

## Task 2: Extract project number parsing from NewProjectModal

**Files:**
- Create: `src/lib/utils/projectNumber.ts`
- Create: `src/lib/utils/__tests__/projectNumber.test.ts`
- Modify: `src/lib/components/NewProjectModal.svelte`

### Step 1: Write failing tests

- [ ] Create `src/lib/utils/__tests__/projectNumber.test.ts` with tests for:
  - `parseProjectNumber("25-97105")` → `{ year: "25", countryCode: "971", seq: "05" }`
  - `parseProjectNumber("26-96606")` → `{ year: "26", countryCode: "966", seq: "06" }`
  - Edge cases: invalid format, empty string
  - `buildProjectPayload(formData, projectNumber)` → complete `Project` object with timestamp

- [ ] Run tests — verify they FAIL

### Step 2: Implement pure functions

- [ ] Create `src/lib/utils/projectNumber.ts`:

```typescript
export function parseProjectNumber(raw: string): ProjectNumber
export function buildProjectPayload(formData: Record<string, unknown>, projectNumber: ProjectNumber): Omit<Project, 'id'>
```

- [ ] Extract parsing logic from `handleCreate()` lines 248–283
- [ ] Run tests — verify they PASS

### Step 3: Refactor NewProjectModal

- [ ] Replace inline parsing/building in `handleCreate()` with calls to extracted functions
- [ ] `handleCreate()` becomes: parse → build → create → folder lifecycle (~25 lines)
- [ ] Run `npm test` — all pass
- [ ] Commit: `refactor(ui): extract project number parsing from NewProjectModal`

---

## Task 3: Split crud.ts — Extract types

**Files:**
- Create: `src/lib/utils/crudTypes.ts`
- Modify: `src/lib/utils/crud.ts`

- [ ] Move all interfaces and type exports to `crudTypes.ts`: `CrudState`, `CrudApi`, `CrudActions`, `CrudStore`, `FilterConfig`, `SortConfig`, etc.
- [ ] Update `crud.ts` to import from `crudTypes.ts`
- [ ] Update all consumers that import types from `crud.ts` to import from `crudTypes.ts` (or re-export from `crud.ts` for backwards compat)
- [ ] Run `npm test` — all pass
- [ ] Commit: `refactor(utils): extract CRUD types to crudTypes.ts`

---

## Task 4: Split crud.ts — Extract pipeline (pure functions)

**Files:**
- Create: `src/lib/utils/crudPipeline.ts`
- Create: `src/lib/utils/__tests__/crudPipeline.test.ts`
- Modify: `src/lib/utils/crud.ts`

### Step 1: Write failing tests

- [ ] Test `applyFiltersAndSearch()` — filters by field values, searches by text query, combined filter+search
- [ ] Test `applySorting()` — sorts by string/number/date fields, ascending/descending

- [ ] Run tests — FAIL

### Step 2: Implement

- [ ] Move `applyFiltersAndSearch()` and `applySorting()` to `crudPipeline.ts`
- [ ] These are pure functions — no store access, easy to test
- [ ] Update `crud.ts` to import from `crudPipeline.ts`

- [ ] Run tests — PASS
- [ ] Commit: `refactor(utils): extract CRUD pipeline to crudPipeline.ts`

---

## Task 5: Split crud.ts — Extract optimistic update logic

**Files:**
- Create: `src/lib/utils/crudOptimistic.ts`
- Create: `src/lib/utils/__tests__/crudOptimistic.test.ts`
- Modify: `src/lib/utils/crud.ts`

### Step 1: Write failing tests

- [ ] Test `withOptimisticCreate()` — adds item to state, rolls back on failure
- [ ] Test `withOptimisticUpdate()` — replaces item, rolls back on failure
- [ ] Test `withOptimisticDelete()` — removes item, rolls back on failure

- [ ] Run tests — FAIL

### Step 2: Implement

- [ ] Extract the repeated optimistic pattern from `create`, `update`, `delete` into a generic wrapper
- [ ] The pattern: snapshot state → apply optimistic change → await API → commit or rollback
- [ ] ~200 lines of duplication collapses to ~80 lines of generic logic
- [ ] Update `crud.ts` to use the new optimistic wrappers

- [ ] Run tests — PASS
- [ ] Commit: `refactor(utils): extract CRUD optimistic logic to crudOptimistic.ts`

---

## Task 6: Split crud.ts — Extract query actions

**Files:**
- Create: `src/lib/utils/crudQueryActions.ts`
- Modify: `src/lib/utils/crud.ts`

- [ ] Move `search()`, `applyFilters()`, `sort()`, `resetFilters()` to `crudQueryActions.ts`
- [ ] These are action factories that mutate `filteredItems` only
- [ ] Update `crud.ts` to import and wire them into the actions object
- [ ] Run `npm test` — all pass
- [ ] Commit: `refactor(utils): extract CRUD query actions to crudQueryActions.ts`

---

## Task 7: Extract independent utilities from crud.ts

**Files:**
- Create: `src/lib/utils/modalState.ts` (from `useModalState()`, lines 854–888)
- Create: `src/lib/utils/operationState.ts` (from `useOperationState()` + `withLoadingState()`, lines 921–994)
- Modify: `src/lib/utils/crud.ts`
- Modify: all consumers that import `useModalState`, `useOperationState`, `withLoadingState` from `crud.ts`

- [ ] Move `useModalState()` to `modalState.ts`
- [ ] Move `useOperationState()` + `withLoadingState()` to `operationState.ts`
- [ ] Update imports in all consumers (grep for `useModalState`, `useOperationState`, `withLoadingState` imports from `crud`)
- [ ] Keep re-exports in `crud.ts` temporarily if needed for backwards compat
- [ ] Run `npm test` — all pass
- [ ] Commit: `refactor(utils): extract modalState and operationState from crud.ts`

---

## Final Verification

- [ ] `npm test` — all tests pass
- [ ] `npm run check` — 0 TypeScript errors
- [ ] Verify `crud.ts` is now ~150 lines (down from 1,165)
- [ ] Verify `performSearch()` in GlobalSearchModal is ~15 lines (down from 114)
- [ ] Verify `handleCreate()` in NewProjectModal is ~25 lines (down from 74)
- [ ] No broken imports across codebase
- [ ] Commit: `refactor: complete god function splits — crud.ts, GlobalSearchModal, NewProjectModal`

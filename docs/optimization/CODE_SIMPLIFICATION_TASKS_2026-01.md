# E-Fees Code Simplification Tasks
**Generated**: 2026-01-18
**Status**: Ready for Implementation

---

## Previous Work Summary

### Completed (August 2025)
- [x] CrudModal system created (60% modal reduction)
- [x] FormField component for dynamic fields
- [x] ContactModal/CompanyModal migrated to CrudModal
- [x] 2,300+ lines eliminated (9.2% reduction)
- [x] 224 tests passing

### Remaining from December 2025 Plan
The 8-week comprehensive refactoring plan was created but not executed. Key remaining phases:
- Phase 3: API Layer Refactoring
- Phase 4: CRUD Utilities Unification
- Phase 5: Database Operations Consolidation
- Phase 6: Command Layer Optimization

---

## Task Categories

### Legend
- **P1** = High Priority (do first)
- **P2** = Medium Priority
- **P3** = Low Priority (polish)
- **Est** = Estimated effort

---

## P1: HIGH PRIORITY TASKS

### 1. Split api.ts into Domain Modules
**File**: `src/lib/api.ts` (2,156 lines)
**Est**: 2-3 hours

- [ ] Create `src/lib/api/` directory structure
- [ ] Extract `api/connection.ts` - DB connection methods
- [ ] Extract `api/projects.ts` - Project CRUD
- [ ] Extract `api/companies.ts` - Company CRUD
- [ ] Extract `api/contacts.ts` - Contact CRUD
- [ ] Extract `api/fees.ts` - Fee/RFP CRUD
- [ ] Extract `api/filesystem.ts` - File operations
- [ ] Extract `api/settings.ts` - Settings management
- [ ] Create `api/index.ts` with re-exports
- [ ] Reduce verbose JSDoc comments (20-40 lines → 3-5 lines)
- [ ] Standardize error handling (all throw OR all return null)
- [ ] Remove unnecessary variable assignments

**Target**: Each file < 300 lines

---

### 2. Split db/mod.rs into Focused Modules
**File**: `src-tauri/src/db/mod.rs` (1,600+ lines)
**Est**: 3-4 hours

- [ ] Extract `db/entities.rs` - All entity structs (Project, Company, Contact, Fee, ActivityLog)
- [ ] Extract `db/client.rs` - Generic DatabaseClient abstraction
- [ ] Create trait-based dispatch to eliminate 400+ lines of match arm duplication
- [ ] Extract `db/pagination.rs` - Generic `paginate<T>()` method
- [ ] Extract `db/queries.rs` - Query builders (eliminate inline SQL string building)
- [ ] Keep `db/mod.rs` as DatabaseManager + connection logic only

**Target**: Each file < 400 lines

---

### 3. Consolidate Duplicate Type Definitions
**Files**: `commands/mod.rs`, `commands/companies.rs`, `commands/contacts.rs`, `commands/settings.rs`
**Est**: 30 minutes

- [ ] Create `src-tauri/src/commands/types.rs`
- [ ] Move `CompanyUpdate` to types.rs (remove from mod.rs:86-95, companies.rs:28-37)
- [ ] Move `ContactUpdate` to types.rs (remove from mod.rs:102-111, contacts.rs:18-27)
- [ ] Move `ProjectUpdate` to types.rs
- [ ] Move `AppSettings` to types.rs (remove from mod.rs, settings.rs:37-52)
- [ ] Update all imports

---

### 4. Fix Nested Ternary in crud.ts
**File**: `src/lib/utils/crud.ts` (lines 937-944)
**Est**: 15 minutes

- [ ] Replace nested ternary:
  ```typescript
  // Before
  actions[loadingType === 'loading' ? 'setLoading' : loadingType === 'saving' ? 'setSaving' : 'setDeleting'](true);

  // After
  const actionMap = { loading: 'setLoading', saving: 'setSaving', deleting: 'setDeleting' };
  actions[actionMap[loadingType]](true);
  ```

---

### 5. Fix Awkward Store State Reading
**File**: `src/lib/utils/crud.ts` (lines 240-243, 770-772)
**Est**: 15 minutes

- [ ] Replace subscribe/unsubscribe pattern with `get()`:
  ```typescript
  // Before
  let stateValue: CrudState<T>;
  const unsubscribe = currentState.subscribe(state => stateValue = state);
  unsubscribe();

  // After
  import { get } from 'svelte/store';
  const stateValue = get(currentState);
  ```

---

### 6. Migrate ProjectModal to CrudModal Pattern
**File**: `src/lib/components/ProjectModal.svelte` (533 lines)
**Est**: 1-2 hours

- [ ] Evaluate if status change workflow allows CrudModal migration
- [ ] If yes: Create `src/lib/config/forms/project.ts` field configuration
- [ ] If no: Document why ProjectModal differs and extract ID matching logic to utility
- [ ] Extract lines 266-298 (32-line ID comparison) to `src/lib/utils/surrealdb.ts`:
  ```typescript
  export function matchSurrealIds(id1: unknown, id2: unknown): boolean
  ```

---

### 7. Create Generic Pagination in Rust
**File**: `src-tauri/src/db/mod.rs` (lines 1421-1593)
**Est**: 1 hour

- [ ] Create generic `paginate<T>()` method:
  ```rust
  pub async fn paginate<T: DeserializeOwned>(
      &self,
      table: &str,
      page: usize,
      page_size: usize,
      order_by: &str
  ) -> Result<PaginatedResponse<T>, Error>
  ```
- [ ] Replace `get_projects_page`, `get_companies_page`, `get_contacts_page`, `get_fees_page`
- [ ] Eliminate 200+ lines of duplicated pagination code

---

### 8. Consolidate Dual Store Pattern
**File**: `src/lib/stores.ts` (lines 45-63)
**Est**: 1-2 hours

- [ ] Remove internal/external store duplication
- [ ] Consolidate 4 repeated patterns (projects, companies, contacts, fees)
- [ ] Fix N+1 query pattern in `companiesWithContactsStore` (lines 166-174)
  - Current: O(n*m) on every update
  - Solution: Cache or move join to backend

---

## P2: MEDIUM PRIORITY TASKS

### 9. Create Entity Logger Factory
**File**: `src/lib/services/activityLogger.ts` (lines 177-260)
**Est**: 30 minutes

- [ ] Create `createEntityLogger(entityType)` factory function
- [ ] Replace 4 identical logger objects (84 lines → ~20 lines)

---

### 10. Simplify adapters.ts
**File**: `src/lib/stores/adapters.ts` (211 lines)
**Est**: 45 minutes

- [ ] Replace class wrappers with object literals:
  ```typescript
  // Before: class with no state
  class ProjectsCrudApiClass implements CrudApi<Project> { ... }

  // After: simple object
  export const projectsApi: CrudApi<Project> = {
    getAll: () => getProjects(),
    create: (data) => createProjectWithTemplate(data),
    ...
  };
  ```
- [ ] Create `assertResult<T>(result, message)` helper for null checks
- [ ] Remove redundant `await` in `return await` patterns

---

### 11. Extract Reusable Search Handler
**Files**: `ContactModal.svelte`, `ProposalModalNew.svelte`, `GlobalSearchModal.svelte`
**Est**: 45 minutes

- [ ] Create `src/lib/utils/searchHelpers.ts`:
  ```typescript
  export function createStoreSearchHandler<T>(
    store: Readable<T[]>,
    filterFields: (keyof T)[],
    mapFn: (item: T) => { value: string; label: string }
  )
  ```
- [ ] Replace 3 identical search handler patterns

---

### 12. Split write_fee_to_json Function
**File**: `src-tauri/src/commands/mod.rs` (lines 979-1199, 220 lines)
**Est**: 45 minutes

- [ ] Extract `find_fee_record()` - database lookup
- [ ] Extract `build_json_data()` - JSON construction
- [ ] Extract `write_to_file()` - file I/O
- [ ] Main function becomes orchestrator (~30 lines)

---

### 13. Fix Activity Logger Types
**File**: `src/lib/stores.ts` (lines 203, 214, 222, etc.)
**Est**: 30 minutes

- [ ] Fix activity logger interface to accept proper entity types
- [ ] Eliminate 15+ occurrences of `as unknown as Record<string, unknown>` double-cast

---

### 14. Extract lib.rs Setup Functions
**File**: `src-tauri/src/lib.rs` (lines 128-256)
**Est**: 30 minutes

- [ ] Extract `setup_database()` function
- [ ] Extract `setup_plugins()` function
- [ ] Extract `setup_window()` function
- [ ] Main setup closure becomes ~30 lines

---

### 15. Fix search.ts Type Safety
**File**: `src/lib/utils/search.ts` (lines 118-120, 301-303)
**Est**: 30 minutes

- [ ] Create `extractProjectNumber(project: Project): string` helper
- [ ] Define proper types for SurrealDB Thing objects
- [ ] Eliminate `as any` casts

---

### 16. Replace Global CSS Override
**File**: `src/lib/components/ProposalModalNew.svelte` (lines 258-263)
**Est**: 10 minutes

- [ ] Remove `:global(.proposal-modal) { z-index: 65 !important; }`
- [ ] Pass `zIndex={65}` prop to CrudModal

---

### 17. Use svelte:window for Keyboard Events
**Files**: `Layout.svelte` (lines 70-76), `Navigation.svelte` (lines 81-84)
**Est**: 15 minutes

- [ ] Replace `onMount`/`onDestroy` event listener pattern
- [ ] Use `<svelte:window on:keydown={handleKeydown} />`

---

## P3: LOW PRIORITY TASKS (Polish)

### 18. Remove Dead Code
**Est**: 15 minutes total

- [ ] `Navigation.svelte`: Remove commented import (line 6) and code block (lines 70-77)
- [ ] `ContactModal.svelte`: Remove unused `showCompanyModal` state and handlers (lines 26, 191-197)
- [ ] `lib.rs`: Remove commented window positioning code (lines 182-194)
- [ ] `crud.ts`: Remove `compareSurrealIdsLocal` wrapper (line 975-977)

---

### 19. Remove Redundant Comments
**Est**: 20 minutes

- [ ] All modal files: Remove 4-6 line header comments that restate filename
- [ ] `CompanyModal.svelte`, `ContactModal.svelte`: Remove empty style blocks
- [ ] `api.ts`: Reduce 20-40 line JSDoc blocks to 3-5 lines

---

### 20. Fix Minor Code Style Issues
**Est**: 15 minutes

- [ ] `contacts.rs` line 139: Rename `contactUpdate` to `contact_update` (Rust convention)
- [ ] `system.rs` line 340: Use `std::env::temp_dir()` instead of hardcoded `/tmp/`
- [ ] `filters.ts` lines 166-171: Return new object instead of mutating parameter
- [ ] `pagination.ts` line 268: Extract magic number to `const PAGE_LOAD_DELAY_MS = 100`

---

### 21. Add Missing Return Types
**Est**: 15 minutes

- [ ] `stores.ts` lines 408, 430-433, 436, 441: Add `Promise<void>` return types
- [ ] `crud.ts` lines 1074-1083, 1088-1100: Convert arrow function exports to function declarations

---

### 22. Fix ContactModal Array Mutation
**File**: `src/lib/components/ContactModal.svelte` (line 131)
**Est**: 10 minutes

- [ ] Define `companyField` directly in fields array instead of using `push()` at module load

---

## Task Summary

| Priority | Count | Estimated Total |
|----------|-------|-----------------|
| P1 High | 8 | 9-12 hours |
| P2 Medium | 9 | 5-6 hours |
| P3 Low | 5 | 1-2 hours |
| **Total** | **22** | **15-20 hours** |

---

## Recommended Execution Order

### Phase 1: Quick Wins (2 hours)
1. Task 4: Fix nested ternary
2. Task 5: Fix store state reading
3. Task 3: Consolidate duplicate types
4. Task 18: Remove dead code

### Phase 2: API Refactoring (4 hours)
5. Task 1: Split api.ts
6. Task 10: Simplify adapters.ts
7. Task 13: Fix activity logger types

### Phase 3: Rust Backend (5 hours)
8. Task 2: Split db/mod.rs
9. Task 7: Create generic pagination
10. Task 12: Split write_fee_to_json
11. Task 14: Extract lib.rs setup functions

### Phase 4: Frontend Stores & Components (4 hours)
12. Task 8: Consolidate dual store pattern
13. Task 6: ProjectModal evaluation
14. Task 9: Entity logger factory
15. Task 11: Reusable search handler

### Phase 5: Polish (2 hours)
16. Remaining P2 and P3 tasks

---

## Files Impact Summary

### Files to Create
- `src/lib/api/` directory (8 files)
- `src-tauri/src/db/entities.rs`
- `src-tauri/src/db/client.rs`
- `src-tauri/src/db/pagination.rs`
- `src-tauri/src/commands/types.rs`
- `src/lib/utils/searchHelpers.ts`

### Files to Significantly Modify
- `src/lib/api.ts` → Split into modules
- `src-tauri/src/db/mod.rs` → Split into modules
- `src/lib/utils/crud.ts` → Fix patterns
- `src/lib/stores.ts` → Consolidate stores
- `src/lib/stores/adapters.ts` → Simplify to objects

### Files to Minimally Modify
- All Svelte modal files (remove comments, fix patterns)
- Rust command files (update imports)
- Various utility files (minor fixes)

---

## Success Metrics

- [ ] `api.ts` split into 8 files, each < 300 lines
- [ ] `db/mod.rs` split into 5 files, each < 400 lines
- [ ] Zero duplicate type definitions in commands/
- [ ] Zero nested ternaries in codebase
- [ ] All 224+ tests passing
- [ ] TypeScript errors reduced (target: 0)

---

*Next Step*: Start with Phase 1 Quick Wins

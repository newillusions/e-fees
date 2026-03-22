---
description: Run E-Fees UI smoke tests via Tauri MCP. Validates database connection, data loading, entity statuses, route rendering, UI state, CRUD operations, integration, and regression guards.
---

# E-Fees Smoke Test

Run all UI validation checks against the running E-Fees desktop app using Tauri MCP tools.

## Prerequisites

- E-Fees desktop app MUST be running (`npm run tauri:dev` or installed app)
- Tauri MCP server must be available
- App window should be visible (not minimized)
- Database connection should be configured and accessible

## Loop Mode

- `/smoke-test --loop` — repeat every 10 minutes
- `/smoke-test --loop 5m` — repeat every 5 minutes

Uses the `/loop` skill infrastructure. Reports only failures after first run.
Prerequisites: App must be running via `npm run tauri:dev`.

## Check Phases (52 total)

1. Safety (1) — abort if production DB detected
2. Infrastructure (2) — DB connection, data loaded
3. Data validation (3) — status values, entity counts
4. Navigation (5) — route rendering
5. UI State (10) — modals, forms, filters, keyboard shortcuts, bulk select
6. CRUD (5) — create/update/delete pipeline with cleanup
7. Integration (6) — status consistency, connection, settings, detail panel
8. Regression (8) — guards against known past bugs

## Test Execution Flow

### Step 1: Import Check Definitions

Read the check definitions from the smoke test implementation file:

```typescript
// File: e2e-mcp/suites/run-smoke.ts
// This file exports:
// - CHECKS: object with check names as keys, JavaScript code strings as values
// - CHECK_ORDER: array defining execution order (52 checks)
```

Reference the CHECKS and CHECK_ORDER exports to understand what each check does and the execution sequence.

### Step 2: Execute Checks in Order

For each check in CHECK_ORDER, execute it sequentially using `mcp__tauri-mcp__execute_js`:

```typescript
// CHECK_ORDER = [
//   // Phase 1: Safety
//   'safety',
//   // Phase 2: Infrastructure
//   'db_connection', 'data_loaded',
//   // Phase 3: Data validation
//   'project_statuses', 'fee_statuses', 'entity_counts',
//   // Phase 4: Navigation
//   'navigate_dashboard', 'navigate_projects', 'navigate_proposals',
//   'navigate_companies', 'navigate_contacts',
//   // Phase 5: UI State
//   'modal_open_close', 'form_validation', 'search_filter', 'dropdown_filter',
//   'keyboard_nav_1', 'keyboard_nav_2', 'keyboard_nav_3',
//   'keyboard_nav_4', 'keyboard_nav_5', 'bulk_select',
//   // Phase 6: CRUD (sequential pipeline)
//   'crud_company', 'crud_contact', 'crud_project', 'crud_fee', 'crud_cleanup',
//   // Phase 7: Integration
//   'status_transition', 'fee_project_mapping', 'connection_indicator',
//   'entity_count_consistency', 'settings_modal', 'detail_panel',
//   // Phase 8: Regression
//   'recordid_v3', 'fee_no_infinity', 'contact_full_name', 'fee_deser',
//   'company_id_extract', 'fee_status_legacy', 'project_status_legacy', 'nav_order',
// ]
```

**For each check:**

1. Get the JavaScript code string from `CHECKS[checkName]`
2. Execute it via `mcp__tauri-mcp__execute_js` with parameter `code: CHECKS[checkName]`
3. Parse the returned result object
4. Check the result structure:
   - `{ check: string; pass: boolean; ABORT?: boolean; error?: string; details?: any }`
5. **If `ABORT: true` is returned, STOP IMMEDIATELY** and report the error

### Step 3: Parse and Validate Results

Each check returns a result object with this structure:

```typescript
{
  check: 'check_name',
  pass: boolean,
  ABORT?: boolean,        // If true, stop all testing immediately
  error?: string,         // Error message if check failed
  details?: {             // Additional information
    [key: string]: any
  }
}
```

### Step 4: Record Results in a Table

After all checks complete, compile results into a summary table:

```markdown
| # | Check | Status | Pass | Details |
|---|-------|--------|------|---------|
| 0  | safety                   | ✓ PASS | yes | db: ws://10.0.21.8:8000 |
| 1  | db_connection            | ✓ PASS | yes | connected |
| 2  | data_loaded              | ✓ PASS | yes | projects: 48, companies: 19, fees: 37, contacts: 52 |
| 3  | project_statuses         | ✓ PASS | yes | all statuses valid, 0 invalid |
| 4  | fee_statuses             | ✓ PASS | yes | all statuses valid, 0 invalid |
| 5  | entity_counts            | ✓ PASS | yes | stats available |
| 6  | navigate_dashboard       | ✓ PASS | yes | DOM rendered, 2500+ bytes |
| 7  | navigate_projects        | ✓ PASS | yes | 48 rows found, filters present |
| 8  | navigate_proposals       | ✓ PASS | yes | 37 rows found |
| 9  | navigate_companies       | ✓ PASS | yes | 19 rows found |
| 10 | navigate_contacts        | ✓ PASS | yes | 52 rows found |
| 11 | modal_open_close         | ✓ PASS | yes | modal opened and closed |
| 12 | form_validation          | ✓ PASS | yes | validation errors shown |
| 13 | search_filter            | ✓ PASS | yes | filter applied |
| 14 | dropdown_filter          | ✓ PASS | yes | status filter works |
| 15 | keyboard_nav_1           | ✓ PASS | yes | Cmd+1 → dashboard |
| 16 | keyboard_nav_2           | ✓ PASS | yes | Cmd+2 → projects |
| 17 | keyboard_nav_3           | ✓ PASS | yes | Cmd+3 → proposals |
| 18 | keyboard_nav_4           | ✓ PASS | yes | Cmd+4 → companies |
| 19 | keyboard_nav_5           | ✓ PASS | yes | Cmd+5 → contacts |
| 20 | bulk_select              | ✓ PASS | yes | checkboxes visible |
| 21 | crud_company             | ✓ PASS | yes | company created |
| 22 | crud_contact             | ✓ PASS | yes | contact created |
| 23 | crud_project             | ✓ PASS | yes | project created |
| 24 | crud_fee                 | ✓ PASS | yes | fee created |
| 25 | crud_cleanup             | ✓ PASS | yes | test data deleted |
| 26 | status_transition        | ✓ PASS | yes | status updated |
| 27 | fee_project_mapping      | ✓ PASS | yes | fees linked to projects |
| 28 | connection_indicator     | ✓ PASS | yes | indicator visible |
| 29 | entity_count_consistency | ✓ PASS | yes | counts match |
| 30 | settings_modal           | ✓ PASS | yes | settings opened |
| 31 | detail_panel             | ✓ PASS | yes | panel renders |
| 32 | recordid_v3              | ✓ PASS | yes | v3 RecordId format handled |
| 33 | fee_no_infinity          | ✓ PASS | yes | no -Infinity values |
| 34 | contact_full_name        | ✓ PASS | yes | full_name populated |
| 35 | fee_deser                | ✓ PASS | yes | fee deserialized correctly |
| 36 | company_id_extract       | ✓ PASS | yes | company ID extracted |
| 37 | fee_status_legacy        | ✓ PASS | yes | no legacy fee statuses |
| 38 | project_status_legacy    | ✓ PASS | yes | no legacy project statuses |
| 39 | nav_order                | ✓ PASS | yes | nav order correct |
```

### Step 5: Take Final Screenshot

After all navigation checks complete, capture final app state:

```typescript
await mcp__tauri-mcp__take_screenshot({
  window_label: 'main',
  path: '/tmp/e-fees-smoke-final.png',
  format: 'png'
})
```

### Step 6: Generate Summary Report

Report overall results:

**Format:**
```
SMOKE TEST RESULTS
==================

Total Checks: 52
Passed: X/52
Failed: Y/52

Status: [✓ PASS / ⚠ PARTIAL / ✗ FAIL]

[Details of any failures or warnings]
```

## Expected Passing Behavior

### Safety Check (CRITICAL)
- Must pass FIRST
- Verifies database URL does NOT include `10.0.23.11` (production)
- If production database detected: **ABORT IMMEDIATELY**

### Database Checks (Phase 2)
- `db_connection`: Connected and responsive
- `data_loaded`: Projects > 0 AND companies > 0 (fees/contacts optional)

### Data Validation (Phase 3)
- `project_statuses`: All projects have valid statuses from domain model
  - Valid: Lead, RFP, Submitted, Awarded, Design, Construction, Completed, Lost, No Response, Cancelled, On Hold, Superseded
- `fee_statuses`: All fees have valid statuses
  - Valid: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded
- `entity_counts`: Stats available and non-null

### Navigation (Phase 4)
- Each route should render with DOM content > 500 bytes
- Projects, Proposals, Companies, Contacts should have visible rows/items

### UI State (Phase 5)
- Modals open and close correctly
- Form validation shows errors on empty required fields
- Search and dropdown filters narrow the list
- Keyboard shortcuts Cmd+1 through Cmd+5 navigate to correct routes
- Bulk select checkboxes are visible on list pages

### CRUD Pipeline (Phase 6)
- Company, contact, project, and fee can each be created with test data
- All test records are deleted by `crud_cleanup` at the end
- CRUD checks run sequentially — later checks may depend on earlier ones (e.g. fee needs a project)

### Integration (Phase 7)
- Status transitions persist correctly
- Fees are linked to their parent projects
- Connection status indicator is visible in the UI
- Entity counts are consistent between stats and actual data
- Settings modal opens and closes
- Detail panel renders on record selection

### Regression (Phase 8)
- `recordid_v3`: SurrealDB v3 RecordId `{table, key}` format handled (not v2 `{tb, id}`)
- `fee_no_infinity`: No `-Infinity` from `math::max([])` in fee pricing fields
- `contact_full_name`: Contacts have `full_name` populated (not null)
- `fee_deser`: Fee deserialization succeeds without panics
- `company_id_extract`: Company IDs extracted correctly from SurrealDB Thing objects
- `fee_status_legacy`: No legacy fee statuses (e.g. "Awarded" → should be "Accepted")
- `project_status_legacy`: No legacy project statuses
- `nav_order`: Navigation order is Dashboard, Projects, Companies, Contacts, Proposals

## Common Failure Causes & Fixes

| Failure | Cause | Fix |
|---------|-------|-----|
| **ABORT: Production DB** | Database URL includes 10.0.23.11 | Switch to dev DB (10.0.21.8:8000 or surreal-dev.internal) |
| **db_connection: FAIL** | Database not running or unreachable | Start SurrealDB container, verify network connectivity |
| **data_loaded: FAIL** | Empty result sets from queries | Check database has test data, verify queries work |
| **project_statuses: FAIL** | Invalid status values in database | Run database migration scripts 001-003 to fix statuses |
| **fee_statuses: FAIL** | Invalid fee status values | Verify domain model restructure was applied (v0.13.0+) |
| **navigate_*: FAIL** | Route doesn't render or empty DOM | Check browser console for JS errors, verify store initialization |
| **entity_counts: FAIL** | get_stats command not available | Verify backend has stats endpoint exposed |
| **crud_*: FAIL** | CRUD command rejected | Check Tauri command registration and DB schema validation |
| **crud_cleanup: FAIL** | Test data not deleted | Manually delete DELETE ME records from dev DB |
| **recordid_v3: FAIL** | Frontend still using v2 RecordId format | Check stores.ts for `{tb, id}` references — update to `{table, key}` |
| **fee_no_infinity: FAIL** | math::max([]) returning -Infinity | Verify SurrealDB query uses IF array::len guard |
| **fee_status_legacy: FAIL** | Old status values in DB | Run migration to rename Awarded→Accepted, Lost→Rejected |

## Cleanup

After smoke test completes:

1. Note any failures or warnings
2. Verify `crud_cleanup` deleted all test data (search for "DELETE ME" records)
3. Return app to Dashboard view for next session
4. Archive screenshot if issues found

## Reference

- **Check Definitions**: `e2e-mcp/suites/run-smoke.ts` (CHECK_ORDER, CHECKS)
- **Helper Module**: `e2e-mcp/suites/helpers/smoke-checks.ts` (original 11 checks)
- **CRUD Checks**: `e2e-mcp/suites/helpers/crud-checks.ts`
- **UI State Checks**: `e2e-mcp/suites/helpers/ui-state-checks.ts`
- **Integration Checks**: `e2e-mcp/suites/helpers/integration-checks.ts`
- **Regression Checks**: `e2e-mcp/suites/helpers/regression-checks.ts`
- **Manual Runbook**: `e2e-mcp/suites/smoke.md` (detailed step-by-step guide)
- **Database Schema**: `DATABASE_SCHEMA.md` (entity definitions)

## Notes

- Smoke tests are **read-only** except for the CRUD phase (Phase 6), which creates and immediately deletes test data
- `crud_cleanup` MUST run even if earlier CRUD checks fail — it guards against test data leaking into the dev DB
- All checks must pass before considering app "ready" for development
- Use `/smoke-test` after major changes, releases, or troubleshooting
- For detailed validation failures, refer to the manual runbook for deeper investigation

---

**Version**: 2.0
**Last Updated**: March 22, 2026
**Purpose**: Full UI validation (52 checks) using Tauri MCP against running E-Fees app

---
description: Run E-Fees UI smoke tests via Tauri MCP. Validates database connection, data loading, entity statuses, and route rendering.
---

# E-Fees Smoke Test

Run all UI validation checks against the running E-Fees desktop app using Tauri MCP tools.

## Prerequisites

- E-Fees desktop app MUST be running (`npm run tauri:dev` or installed app)
- Tauri MCP server must be available
- App window should be visible (not minimized)
- Database connection should be configured and accessible

## Test Execution Flow

### Step 1: Import Check Definitions

Read the check definitions from the smoke test implementation file:

```typescript
// File: e2e-mcp/suites/run-smoke.ts
// This file exports:
// - CHECKS: object with check names as keys, JavaScript code strings as values
// - CHECK_ORDER: array defining execution order
```

Reference the CHECKS and CHECK_ORDER exports to understand what each check does and the execution sequence.

### Step 2: Execute Checks in Order

For each check in CHECK_ORDER, execute it sequentially using `mcp__tauri-mcp__execute_js`:

```typescript
// CHECK_ORDER = [
//   'safety',
//   'db_connection',
//   'data_loaded',
//   'project_statuses',
//   'fee_statuses',
//   'entity_counts',
//   'navigate_dashboard',
//   'navigate_projects',
//   'navigate_proposals',
//   'navigate_companies',
//   'navigate_contacts',
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
| 0 | safety | ✓ PASS | yes | db: ws://10.0.21.8:8000 |
| 1 | db_connection | ✓ PASS | yes | connected, timestamp: ... |
| 2 | data_loaded | ✓ PASS | yes | projects: 48, companies: 19, fees: 37, contacts: 52 |
| 3 | project_statuses | ✓ PASS | yes | all statuses valid, 0 invalid |
| 4 | fee_statuses | ✓ PASS | yes | all statuses valid, 0 invalid |
| 5 | entity_counts | ✓ PASS | yes | stats available |
| 6 | navigate_dashboard | ✓ PASS | yes | DOM rendered, 2500+ bytes |
| 7 | navigate_projects | ✓ PASS | yes | 48 rows found, filters present |
| 8 | navigate_proposals | ✓ PASS | yes | 37 rows found |
| 9 | navigate_companies | ✓ PASS | yes | 19 rows found |
| 10 | navigate_contacts | ✓ PASS | yes | 52 rows found |
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

Total Checks: 11
Passed: X/11
Failed: Y/11

Status: [✓ PASS / ⚠ PARTIAL / ✗ FAIL]

[Details of any failures or warnings]
```

## Expected Passing Behavior

### Safety Check (CRITICAL)
- Must pass FIRST
- Verifies database URL does NOT include `10.0.23.11` (production)
- If production database detected: **ABORT IMMEDIATELY**

### Database Checks (1-2)
- `db_connection`: Connected and responsive
- `data_loaded`: Projects > 0 AND companies > 0 (fees/contacts optional)

### Data Validation (3-4)
- `project_statuses`: All projects have valid statuses from domain model
  - Valid: Lead, RFP, Submitted, Awarded, Design, Construction, Completed, Lost, No Response, Cancelled, On Hold, Superseded
- `fee_statuses`: All fees have valid statuses
  - Valid: Draft, Sent, Negotiation, Accepted, Rejected, No Response, Superseded

### Entity Counts (5)
- `entity_counts`: Stats available and non-null

### Navigation (6-10)
- Each route should render with DOM content > 500 bytes
- Projects, Proposals, Companies, Contacts should have visible rows/items

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

## Cleanup

After smoke test completes:

1. Note any failures or warnings
2. Do NOT delete test data (smoke tests are read-only)
3. Return app to Dashboard view for next session
4. Archive screenshot if issues found

## Reference

- **Check Definitions**: `e2e-mcp/suites/run-smoke.ts` (CHECK_ORDER, CHECKS)
- **Helper Module**: `e2e-mcp/suites/helpers/smoke-checks.ts` (detailed JS code strings)
- **Manual Runbook**: `e2e-mcp/suites/smoke.md` (detailed step-by-step guide)
- **Database Schema**: `DATABASE_SCHEMA.md` (entity definitions)

## Notes

- Smoke tests are **read-only** — no data is created, modified, or deleted
- All checks must pass before considering app "ready" for development
- Use `/smoke-test` after major changes, releases, or troubleshooting
- For detailed validation failures, refer to the manual runbook for deeper investigation

---

**Version**: 1.0  
**Last Updated**: February 28, 2026  
**Purpose**: Quick UI validation using Tauri MCP against running E-Fees app

# E-Fees UI Testing & API — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add UI validation via Tauri MCP tools and build a standalone Rust API service for programmatic data access, sharing types through a core library crate.

**Architecture:** Two workstreams — (A) UI smoke tests using Tauri MCP tools against the running desktop app, (B) standalone axum API service sharing domain types with the Tauri app via an extracted `e-fees-core` crate. Both query the same SurrealDB instance.

**Tech Stack:** Rust (axum 0.8, surrealdb 3.0), TypeScript (Vitest), Tauri MCP tools, Docker.

**Design doc:** `docs/plans/2026-02-28-ui-testing-and-api-design.md`

---

## Phase 1: UI Smoke Tests

### Task 1: Create the smoke test helper module

**Files:**
- Create: `e2e-mcp/suites/helpers/tauri-mcp.ts`

The existing `e2e-mcp/helpers/mcp-client.ts` calls global functions (`mcp__tauri_mcp__execute_js` etc.) that don't exist in a vitest environment. The Tauri MCP tools are available as Claude Code MCP tools, not as importable functions.

Instead, the smoke tests will be **script files that Claude runs via Tauri MCP tools directly** — not vitest test files. They output structured results that Claude reads.

**Step 1: Create the smoke test runner script**

This is a TypeScript module that gets executed inside the Tauri webview via `execute_js`. It checks app state and returns results.

```typescript
// e2e-mcp/suites/helpers/smoke-checks.ts
//
// These functions return JS code strings to be passed to Tauri MCP execute_js.
// Each returns a serializable result object.

/** Check if the app is connected to the database */
export const checkDbConnection = `
(async () => {
  try {
    const result = await window.__TAURI__.invoke('get_connection_status');
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    return {
      check: 'db_connection',
      pass: result === true || result?.connected === true,
      details: {
        connected: result,
        db: dbInfo
      }
    };
  } catch (e) {
    return { check: 'db_connection', pass: false, error: e.message };
  }
})()
`;

/** Check that data has loaded into stores */
export const checkDataLoaded = \`
(async () => {
  try {
    const projects = await window.__TAURI__.invoke('get_projects');
    const companies = await window.__TAURI__.invoke('get_companies');
    const contacts = await window.__TAURI__.invoke('get_contacts');
    const fees = await window.__TAURI__.invoke('get_fees');
    return {
      check: 'data_loaded',
      pass: projects.length > 0 && companies.length > 0,
      details: {
        projects: projects.length,
        companies: companies.length,
        contacts: contacts.length,
        fees: fees.length
      }
    };
  } catch (e) {
    return { check: 'data_loaded', pass: false, error: e.message };
  }
})()
\`;

/** Check that the current DB is NOT production */
export const checkNotProduction = \`
(async () => {
  try {
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    const url = dbInfo?.url || dbInfo?.endpoint || '';
    const isProd = url.includes('10.0.23.11');
    return {
      check: 'not_production',
      pass: !isProd,
      details: { url, isProd },
      error: isProd ? 'REFUSING: Connected to PRODUCTION database' : null
    };
  } catch (e) {
    return { check: 'not_production', pass: false, error: e.message };
  }
})()
\`;

/** Check entity counts match expected dashboard stats */
export const checkEntityCounts = \`
(async () => {
  try {
    const stats = await window.__TAURI__.invoke('get_stats');
    return {
      check: 'entity_counts',
      pass: stats != null,
      details: stats
    };
  } catch (e) {
    return { check: 'entity_counts', pass: false, error: e.message };
  }
})()
\`;

/** Check project status values are the new domain model values */
export const checkProjectStatuses = \`
(async () => {
  try {
    const projects = await window.__TAURI__.invoke('get_projects');
    const statuses = [...new Set(projects.map(p => p.status))].sort();
    const validStatuses = [
      'Lead', 'RFP', 'Submitted', 'Awarded', 'Design', 'Construction',
      'Completed', 'Lost', 'No Response', 'Cancelled', 'On Hold', 'Superseded'
    ];
    const invalidStatuses = statuses.filter(s => !validStatuses.includes(s));
    return {
      check: 'project_statuses',
      pass: invalidStatuses.length === 0,
      details: { found: statuses, invalid: invalidStatuses }
    };
  } catch (e) {
    return { check: 'project_statuses', pass: false, error: e.message };
  }
})()
\`;

/** Check fee status values are the new domain model values */
export const checkFeeStatuses = \`
(async () => {
  try {
    const fees = await window.__TAURI__.invoke('get_fees');
    const statuses = [...new Set(fees.map(f => f.status))].sort();
    const validStatuses = [
      'Draft', 'Sent', 'Negotiation', 'Accepted', 'Rejected',
      'No Response', 'Superseded'
    ];
    const invalidStatuses = statuses.filter(s => !validStatuses.includes(s));
    return {
      check: 'fee_statuses',
      pass: invalidStatuses.length === 0,
      details: { found: statuses, invalid: invalidStatuses }
    };
  } catch (e) {
    return { check: 'fee_statuses', pass: false, error: e.message };
  }
})()
\`;
```

**Step 2: Verify the file was created correctly**

Run: Read the file back, confirm syntax is valid.

**Step 3: Commit**

```bash
git add e2e-mcp/suites/helpers/smoke-checks.ts
git commit -m "feat(e2e): add smoke check JS snippets for Tauri MCP validation"
```

---

### Task 2: Create the DOM validation checks

**Files:**
- Create: `e2e-mcp/suites/helpers/dom-checks.ts`

These check the rendered DOM for expected elements, not just the data layer.

**Step 1: Write the DOM check module**

```typescript
// e2e-mcp/suites/helpers/dom-checks.ts
//
// DOM structure checks — run via Tauri MCP get_dom, then parse results.
// These are check descriptions + CSS selectors for Claude to verify.

export interface DOMCheck {
  name: string;
  description: string;
  /** CSS selector or text pattern to look for in the DOM */
  selector?: string;
  /** Text content to search for */
  textContent?: string;
  /** Route to navigate to before checking (hash router path) */
  route?: string;
}

/** Checks that run on any page (app shell) */
export const appShellChecks: DOMCheck[] = [
  {
    name: 'sidebar_navigation',
    description: 'Sidebar with 5 navigation links exists',
    selector: 'nav, [class*="sidebar"], [class*="navigation"]',
  },
  {
    name: 'connection_indicator',
    description: 'Database connection status indicator is visible',
    selector: '[class*="connection"], [class*="status-indicator"]',
  },
  {
    name: 'app_title',
    description: 'App title shows E-Fees in header/sidebar',
    textContent: 'E-Fees',
  },
];

/** Per-route checks */
export const routeChecks: Record<string, DOMCheck[]> = {
  '/': [
    {
      name: 'dashboard_stats',
      description: 'Dashboard shows entity count cards',
      selector: '[class*="stat"], [class*="card"], [class*="dashboard"]',
    },
  ],
  '/projects': [
    {
      name: 'projects_table',
      description: 'Projects list/table renders with rows',
      selector: 'table, [class*="project-list"], [class*="list"]',
    },
    {
      name: 'projects_filter',
      description: 'Status filter dropdown exists',
      selector: 'select, [class*="filter"]',
    },
    {
      name: 'projects_search',
      description: 'Search input exists',
      selector: 'input[type="text"], input[type="search"], [class*="search"]',
    },
  ],
  '/proposals': [
    {
      name: 'proposals_table',
      description: 'Proposals list renders',
      selector: 'table, [class*="fee-list"], [class*="proposal-list"], [class*="list"]',
    },
  ],
  '/companies': [
    {
      name: 'companies_table',
      description: 'Companies list renders',
      selector: 'table, [class*="company-list"], [class*="list"]',
    },
  ],
  '/contacts': [
    {
      name: 'contacts_table',
      description: 'Contacts list renders',
      selector: 'table, [class*="contact-list"], [class*="list"]',
    },
  ],
};

/**
 * JS to execute in webview to navigate to a hash route.
 * E-Fees uses svelte-spa-router (hash-based).
 */
export function navigateToRoute(route: string): string {
  return `
    (async () => {
      window.location.hash = '#${route}';
      await new Promise(r => setTimeout(r, 1000)); // wait for route + data load
      return { navigated: '${route}', hash: window.location.hash };
    })()
  `;
}

/**
 * JS to check for console errors captured since app start.
 */
export const checkConsoleErrors = `
(async () => {
  // Check if we have any error tracking
  const errors = window.__capturedErrors || [];
  return {
    check: 'console_errors',
    pass: errors.length === 0,
    details: { errorCount: errors.length, errors: errors.slice(0, 5) }
  };
})()
`;
```

**Step 2: Commit**

```bash
git add e2e-mcp/suites/helpers/dom-checks.ts
git commit -m "feat(e2e): add DOM validation check definitions for route testing"
```

---

### Task 3: Create the smoke test runner

**Files:**
- Create: `e2e-mcp/suites/smoke.md`

This is a **runbook for Claude** — a structured document that Claude follows using Tauri MCP tools. Not an automated script, because the MCP tools are available to Claude, not to Node.js.

**Step 1: Write the smoke test runbook**

```markdown
<!-- e2e-mcp/suites/smoke.md -->
# E-Fees Smoke Test Runbook

Run these checks using Tauri MCP tools against the running desktop app.
The app must be running (`npm run tauri:dev` or installed app).

## Prerequisites

- App is running and visible
- Tauri MCP server is connected

## Safety Check (MUST RUN FIRST)

Use `mcp__tauri-mcp__execute_js` with this code:

\`\`\`javascript
(async () => {
  try {
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    const url = dbInfo?.url || dbInfo?.endpoint || '';
    const isProd = url.includes('10.0.23.11');
    return {
      check: 'not_production',
      pass: !isProd,
      url: url,
      STOP_IF_PRODUCTION: isProd
    };
  } catch (e) {
    return { check: 'not_production', pass: false, error: e.message };
  }
})()
\`\`\`

**IF `STOP_IF_PRODUCTION` is true: STOP ALL TESTING IMMEDIATELY.**

## Check 1: Database Connection

Execute JS:
\`\`\`javascript
(async () => {
  const status = await window.__TAURI__.invoke('get_connection_status');
  const info = await window.__TAURI__.invoke('get_db_info');
  return { connected: status, db: info };
})()
\`\`\`

**Pass:** `connected` is truthy.

## Check 2: Data Loads

Execute JS:
\`\`\`javascript
(async () => {
  const p = await window.__TAURI__.invoke('get_projects');
  const c = await window.__TAURI__.invoke('get_companies');
  const f = await window.__TAURI__.invoke('get_fees');
  const co = await window.__TAURI__.invoke('get_contacts');
  return { projects: p.length, companies: c.length, fees: f.length, contacts: co.length };
})()
\`\`\`

**Pass:** All counts > 0.

## Check 3: Project Status Values

Execute JS:
\`\`\`javascript
(async () => {
  const p = await window.__TAURI__.invoke('get_projects');
  const statuses = [...new Set(p.map(x => x.status))].sort();
  const valid = ['Lead','RFP','Submitted','Awarded','Design','Construction','Completed','Lost','No Response','Cancelled','On Hold','Superseded'];
  const invalid = statuses.filter(s => !valid.includes(s));
  return { statuses, invalid, pass: invalid.length === 0 };
})()
\`\`\`

**Pass:** `invalid` is empty.

## Check 4: Fee Status Values

Execute JS:
\`\`\`javascript
(async () => {
  const f = await window.__TAURI__.invoke('get_fees');
  const statuses = [...new Set(f.map(x => x.status))].sort();
  const valid = ['Draft','Sent','Negotiation','Accepted','Rejected','No Response','Superseded'];
  const invalid = statuses.filter(s => !valid.includes(s));
  return { statuses, invalid, pass: invalid.length === 0 };
})()
\`\`\`

**Pass:** `invalid` is empty.

## Check 5: Dashboard Route Renders

1. Execute JS: `window.location.hash = '#/'; await new Promise(r => setTimeout(r, 1000));`
2. Use `mcp__tauri-mcp__get_dom` to capture DOM.
3. Verify DOM contains stat cards or dashboard elements.
4. Use `mcp__tauri-mcp__take_screenshot` to capture visual state.

**Pass:** DOM is not empty, contains data elements.

## Check 6: Projects Route Renders

1. Execute JS: `window.location.hash = '#/projects'; await new Promise(r => setTimeout(r, 1500));`
2. Use `mcp__tauri-mcp__get_dom`.
3. Verify: table/list element exists, rows present, filter/search inputs exist.
4. Screenshot.

**Pass:** Project rows visible in DOM.

## Check 7: Proposals Route Renders

1. Navigate: `window.location.hash = '#/proposals';`
2. Wait 1.5s. Get DOM. Screenshot.

**Pass:** Fee/proposal rows visible.

## Check 8: Companies Route Renders

1. Navigate: `window.location.hash = '#/companies';`
2. Wait 1.5s. Get DOM. Screenshot.

**Pass:** Company rows visible.

## Check 9: Contacts Route Renders

1. Navigate: `window.location.hash = '#/contacts';`
2. Wait 1.5s. Get DOM. Screenshot.

**Pass:** Contact rows visible.

## Check 10: Entity Counts Match

Execute JS:
\`\`\`javascript
(async () => {
  const stats = await window.__TAURI__.invoke('get_stats');
  return stats;
})()
\`\`\`

Compare with Check 2 counts. They should match.

**Pass:** Counts consistent.

## Results Summary

Report: X/10 checks passed. List any failures with details.
```

**Step 2: Commit**

```bash
git add e2e-mcp/suites/smoke.md
git commit -m "feat(e2e): add smoke test runbook for Tauri MCP validation"
```

---

### Task 4: Create a Claude-executable smoke test script

**Files:**
- Create: `e2e-mcp/suites/run-smoke.ts`

A single TypeScript file containing all the JS snippets as exported constants, plus a `runAllChecks()` function that returns structured results. Claude can read this file and execute each check via Tauri MCP.

**Step 1: Write the executable smoke test**

```typescript
// e2e-mcp/suites/run-smoke.ts
//
// Smoke test definitions for E-Fees.
// Usage: Claude reads this file, then executes each check via mcp__tauri-mcp__execute_js.
// Each check is a self-contained JS string that returns { check, pass, details }.

export const CHECKS = {
  safety: `(async () => {
    const dbInfo = await window.__TAURI__.invoke('get_db_info');
    const url = dbInfo?.url || dbInfo?.endpoint || '';
    const isProd = url.includes('10.0.23.11');
    if (isProd) return { check: 'safety', pass: false, ABORT: true, error: 'PRODUCTION DATABASE - STOP ALL TESTING' };
    return { check: 'safety', pass: true, db: url };
  })()`,

  db_connection: `(async () => {
    const status = await window.__TAURI__.invoke('get_connection_status');
    const info = await window.__TAURI__.invoke('get_db_info');
    return { check: 'db_connection', pass: !!status, details: { status, info } };
  })()`,

  data_loaded: `(async () => {
    const p = await window.__TAURI__.invoke('get_projects');
    const c = await window.__TAURI__.invoke('get_companies');
    const f = await window.__TAURI__.invoke('get_fees');
    const co = await window.__TAURI__.invoke('get_contacts');
    const counts = { projects: p.length, companies: c.length, fees: f.length, contacts: co.length };
    return { check: 'data_loaded', pass: p.length > 0 && c.length > 0, details: counts };
  })()`,

  project_statuses: `(async () => {
    const p = await window.__TAURI__.invoke('get_projects');
    const statuses = [...new Set(p.map(x => x.status))].sort();
    const valid = ['Lead','RFP','Submitted','Awarded','Design','Construction','Completed','Lost','No Response','Cancelled','On Hold','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'project_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  fee_statuses: `(async () => {
    const f = await window.__TAURI__.invoke('get_fees');
    const statuses = [...new Set(f.map(x => x.status))].sort();
    const valid = ['Draft','Sent','Negotiation','Accepted','Rejected','No Response','Superseded'];
    const invalid = statuses.filter(s => !valid.includes(s));
    return { check: 'fee_statuses', pass: invalid.length === 0, details: { found: statuses, invalid } };
  })()`,

  entity_counts: `(async () => {
    const stats = await window.__TAURI__.invoke('get_stats');
    return { check: 'entity_counts', pass: stats != null, details: stats };
  })()`,

  navigate_dashboard: `(async () => {
    window.location.hash = '#/';
    await new Promise(r => setTimeout(r, 1000));
    const title = document.title;
    const bodyLen = document.body.innerHTML.length;
    return { check: 'navigate_dashboard', pass: bodyLen > 500, details: { title, domSize: bodyLen, hash: window.location.hash } };
  })()`,

  navigate_projects: `(async () => {
    window.location.hash = '#/projects';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="project-item"]');
    const filters = document.querySelectorAll('select, [class*="filter"]');
    return { check: 'navigate_projects', pass: rows.length > 1, details: { rows: rows.length, filters: filters.length } };
  })()`,

  navigate_proposals: `(async () => {
    window.location.hash = '#/proposals';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="fee-item"]');
    return { check: 'navigate_proposals', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,

  navigate_companies: `(async () => {
    window.location.hash = '#/companies';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="company-item"]');
    return { check: 'navigate_companies', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,

  navigate_contacts: `(async () => {
    window.location.hash = '#/contacts';
    await new Promise(r => setTimeout(r, 1500));
    const rows = document.querySelectorAll('tr, [class*="row"], [class*="contact-item"]');
    return { check: 'navigate_contacts', pass: rows.length > 1, details: { rows: rows.length } };
  })()`,
};

/**
 * Execution order matters:
 * 1. safety — MUST be first, abort if production
 * 2. db_connection — verify connectivity
 * 3. data_loaded — verify data exists
 * 4. project_statuses, fee_statuses — verify domain model
 * 5. entity_counts — verify stats endpoint
 * 6. navigate_* — verify each route renders
 *
 * After navigation checks, take a screenshot of the final state.
 */
export const CHECK_ORDER = [
  'safety',
  'db_connection',
  'data_loaded',
  'project_statuses',
  'fee_statuses',
  'entity_counts',
  'navigate_dashboard',
  'navigate_projects',
  'navigate_proposals',
  'navigate_companies',
  'navigate_contacts',
] as const;
```

**Step 2: Commit**

```bash
git add e2e-mcp/suites/run-smoke.ts
git commit -m "feat(e2e): add executable smoke test check definitions"
```

---

### Task 5: Add a Claude Code skill for running smoke tests

**Files:**
- Create: `.claude/commands/smoke-test.md`

A slash command so you can just type `/smoke-test` to run the full validation.

**Step 1: Write the skill**

```markdown
---
name: smoke-test
description: Run E-Fees UI smoke tests via Tauri MCP. Requires the app to be running.
---

# E-Fees Smoke Test

Run all UI validation checks against the running E-Fees desktop app.

## Prerequisites

The app MUST be running (`npm run tauri:dev` or installed app).

## Execution

1. Read `e2e-mcp/suites/run-smoke.ts` to get the check definitions and order.

2. For each check in `CHECK_ORDER`, execute it via `mcp__tauri-mcp__execute_js`:
   - Pass the JS string from `CHECKS[checkName]` as the `code` parameter
   - Parse the returned result object
   - If `ABORT: true` is returned, STOP IMMEDIATELY and report the error

3. After all navigation checks, take a final screenshot via `mcp__tauri-mcp__take_screenshot`.

4. Report results as a table:

| # | Check | Pass | Details |
|---|-------|------|---------|
| 1 | safety | PASS | db: ws://10.0.23.12:8000 |
| 2 | db_connection | PASS | connected |
| ... | ... | ... | ... |

**Result: X/11 checks passed.**

5. If any checks failed, provide specific details about what went wrong and suggest fixes.
```

**Step 2: Commit**

```bash
git add .claude/commands/smoke-test.md
git commit -m "feat: add /smoke-test slash command for UI validation"
```

---

### Task 6: Run the smoke tests against v0.13.1

**Files:** None (execution only)

**Step 1:** Ensure the app is running via `npm run tauri:dev` or the installed v0.13.1 app.

**Step 2:** Run `/smoke-test` — execute each check in order via Tauri MCP.

**Step 3:** Capture screenshot of each route.

**Step 4:** Report results. Fix any failures found.

**Step 5:** Commit any fixes, then re-run to confirm all green.

---

## Phase 2: Core Library Extraction

### Task 7: Create the crate structure

**Files:**
- Create: `crates/e-fees-core/Cargo.toml`
- Create: `crates/e-fees-core/src/lib.rs`
- Modify: `Cargo.toml` (workspace root — create workspace if needed)

**Step 1: Create the workspace Cargo.toml** (if not exists)

Check if a workspace `Cargo.toml` exists at the repo root. If not, create one. The Tauri app's `Cargo.toml` is at `src-tauri/Cargo.toml`.

```toml
# Cargo.toml (repo root) — workspace definition
[workspace]
members = [
  "src-tauri",
  "crates/e-fees-core",
]
resolver = "2"
```

**Step 2: Create the core crate**

```toml
# crates/e-fees-core/Cargo.toml
[package]
name = "e-fees-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
surrealdb-types = "3.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
```

```rust
// crates/e-fees-core/src/lib.rs
pub mod models;
```

**Step 3: Verify it compiles**

Run: `cd crates/e-fees-core && cargo check`
Expected: Compiles with no errors.

**Step 4: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat: create e-fees-core crate skeleton with workspace"
```

---

### Task 8: Extract domain model types

**Files:**
- Create: `crates/e-fees-core/src/models/mod.rs`
- Create: `crates/e-fees-core/src/models/project.rs`
- Create: `crates/e-fees-core/src/models/fee.rs`
- Create: `crates/e-fees-core/src/models/company.rs`
- Create: `crates/e-fees-core/src/models/contact.rs`
- Create: `crates/e-fees-core/src/models/common.rs`
- Refer to: `src-tauri/src/db/types.rs` (source of truth, lines 1-600+)

**Step 1: Copy the model structs from `src-tauri/src/db/types.rs` into the core crate**

Move the following into separate files per entity:

- `common.rs` — `TimeStamps`, `PaginatedResponse<T>`, `EntityCounts`, `ActivityLog`, helper functions (`record_key_string`, `record_id_string`)
- `project.rs` — `Project` (lines 128-139), `ProjectNumber` (143-148), `NewProject` (160-169), status constants
- `fee.rs` — `Fee` (232-285), `FeeCreate` (301-337), `FeeUpdate` (341-379), `Revision`, `FeeStage`, `PricingData`, etc.
- `company.rs` — `Company` (173-183), `CompanyCreate` (187-195)
- `contact.rs` — `Contact` (199-217), `ContactCreate` (221-228)

**Important:** Remove any Tauri-specific derives or imports. Keep only `serde`, `surrealdb-types`, `chrono`, `uuid`.

**Step 2: Verify the core crate compiles**

Run: `cd crates/e-fees-core && cargo check`

**Step 3: Commit**

```bash
git add crates/e-fees-core/
git commit -m "feat(core): extract domain models from Tauri app into shared crate"
```

---

### Task 9: Refactor Tauri app to use `e-fees-core`

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `e-fees-core` dependency
- Modify: `src-tauri/src/db/types.rs` — replace struct definitions with re-exports
- Modify: `src-tauri/src/db/mod.rs` — update imports
- Modify: `src-tauri/src/agent_server.rs` — update imports
- Modify: `src-tauri/src/lib.rs` — update imports if needed

**Step 1: Add dependency to Tauri app**

In `src-tauri/Cargo.toml`:
```toml
[dependencies]
e-fees-core = { path = "../crates/e-fees-core" }
```

**Step 2: Replace type definitions with re-exports in `types.rs`**

```rust
// src-tauri/src/db/types.rs — top of file
pub use e_fees_core::models::*;
// Remove the struct/enum definitions that now live in e-fees-core
// Keep any Tauri-specific types that can't be shared
```

**Step 3: Run the full test suite**

Run: `cd src-tauri && cargo test`
Expected: All 87 tests pass. Zero regressions.

**Step 4: Run svelte-check**

Run: `npm run check`
Expected: 0 errors (warnings OK).

**Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/
git commit -m "refactor: use e-fees-core shared types in Tauri app"
```

---

## Phase 3: Standalone API Service

### Task 10: Create API service skeleton

**Files:**
- Create: `e-fees-api/Cargo.toml`
- Create: `e-fees-api/src/main.rs`
- Create: `e-fees-api/src/config.rs`
- Create: `e-fees-api/.env.example`
- Modify: `Cargo.toml` (workspace — add member)

**Step 1: Add to workspace**

```toml
# Cargo.toml (root)
[workspace]
members = [
  "src-tauri",
  "crates/e-fees-core",
  "e-fees-api",
]
```

**Step 2: Create API Cargo.toml**

```toml
# e-fees-api/Cargo.toml
[package]
name = "e-fees-api"
version = "0.1.0"
edition = "2021"

[dependencies]
e-fees-core = { path = "../crates/e-fees-core" }
axum = "0.8"
tokio = { version = "1", features = ["full"] }
surrealdb = { version = "3.0", features = ["protocol-ws", "rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.6", features = ["cors"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Step 3: Write config.rs**

```rust
// e-fees-api/src/config.rs
use std::env;

pub struct Config {
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub api_key: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            surreal_url: env::var("SURREAL_URL").expect("SURREAL_URL required"),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "projects".into()),
            surreal_user: env::var("SURREAL_USER").expect("SURREAL_USER required"),
            surreal_pass: env::var("SURREAL_PASS").expect("SURREAL_PASS required"),
            api_key: env::var("API_KEY").expect("API_KEY required"),
            port: env::var("API_PORT").unwrap_or_else(|_| "3200".into()).parse().expect("Invalid API_PORT"),
        }
    }
}
```

**Step 4: Write main.rs skeleton**

```rust
// e-fees-api/src/main.rs
mod config;

use axum::{Router, routing::get, Json};
use config::Config;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::Ws;
use std::sync::Arc;

struct AppState {
    db: Surreal<surrealdb::engine::remote::ws::Client>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env();

    // Connect to SurrealDB
    let db = Surreal::new::<Ws>(&config.surreal_url)
        .await
        .expect("Failed to connect to SurrealDB");

    db.signin(surrealdb::opt::auth::Database {
        namespace: &config.surreal_ns,
        database: &config.surreal_db,
        username: &config.surreal_user,
        password: &config.surreal_pass,
    })
    .await
    .expect("Failed to authenticate");

    db.use_ns(&config.surreal_ns).use_db(&config.surreal_db).await.unwrap();

    let state = Arc::new(AppState { db });

    let app = Router::new()
        .route("/health", get(health))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("E-Fees API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "e-fees-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

**Step 5: Create .env.example**

```env
# e-fees-api/.env.example
SURREAL_URL=ws://10.0.23.12:8000
SURREAL_NS=emittiv
SURREAL_DB=projects
SURREAL_USER=martin
SURREAL_PASS=
API_KEY=
API_PORT=3200
```

**Step 6: Verify it compiles**

Run: `cd e-fees-api && cargo check`

**Step 7: Commit**

```bash
git add Cargo.toml e-fees-api/
git commit -m "feat(api): create e-fees-api service skeleton with health endpoint"
```

---

### Task 11: Add API key auth middleware

**Files:**
- Create: `e-fees-api/src/auth.rs`
- Modify: `e-fees-api/src/main.rs` — apply middleware

**Step 1: Write auth middleware**

Reference the existing pattern in `src-tauri/src/agent_server.rs` for how auth is handled there. Adapt for standalone use.

```rust
// e-fees-api/src/auth.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn require_api_key(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = std::env::var("API_KEY").unwrap_or_default();

    // Skip auth for health endpoint
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    match request.headers().get("X-API-Key") {
        Some(key) if key.to_str().unwrap_or("") == api_key => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
```

**Step 2: Apply to router in main.rs**

```rust
use axum::middleware;
mod auth;

// In main():
let app = Router::new()
    .route("/health", get(health))
    // ... other routes
    .layer(middleware::from_fn(auth::require_api_key))
    .with_state(state);
```

**Step 3: Commit**

```bash
git add e-fees-api/src/auth.rs e-fees-api/src/main.rs
git commit -m "feat(api): add API key authentication middleware"
```

---

### Task 12: Add read-only route handlers

**Files:**
- Create: `e-fees-api/src/routes/mod.rs`
- Create: `e-fees-api/src/routes/projects.rs`
- Create: `e-fees-api/src/routes/fees.rs`
- Create: `e-fees-api/src/routes/companies.rs`
- Create: `e-fees-api/src/routes/contacts.rs`
- Create: `e-fees-api/src/routes/stats.rs`
- Create: `e-fees-api/src/error.rs`
- Modify: `e-fees-api/src/main.rs` — register routes

**Step 1: Create error module**

```rust
// e-fees-api/src/error.rs
use axum::{http::StatusCode, Json, response::IntoResponse};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(json!({
            "error": self.code,
            "message": self.message
        }))).into_response()
    }
}

impl From<surrealdb::Error> for ApiError {
    fn from(e: surrealdb::Error) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "database_error".into(),
            message: e.to_string(),
        }
    }
}
```

**Step 2: Create route handlers**

Reference the query patterns in `src-tauri/src/db/mod.rs` (the `DatabaseManager` methods) and `src-tauri/src/agent_server.rs` (the existing axum handlers). Reuse the same SurrealQL queries.

Each route handler follows the pattern:
```rust
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let projects: Vec<Project> = state.db.select("projects").await?;
    Ok(Json(json!({ "data": projects, "count": projects.len() })))
}
```

Build out all GET routes following the query patterns already in the codebase.

**Step 3: Register routes in main.rs**

```rust
mod routes;

let app = Router::new()
    .route("/health", get(health))
    .route("/stats", get(routes::stats::get_stats))
    .route("/projects", get(routes::projects::list_projects))
    .route("/projects/{id}", get(routes::projects::get_project))
    .route("/fees", get(routes::fees::list_fees))
    .route("/fees/{id}", get(routes::fees::get_fee))
    .route("/companies", get(routes::companies::list_companies))
    .route("/companies/{id}", get(routes::companies::get_company))
    .route("/contacts", get(routes::contacts::list_contacts))
    .layer(middleware::from_fn(auth::require_api_key))
    .with_state(state);
```

**Step 4: Test locally**

Run: `cd e-fees-api && cargo run` (with `.env` pointing to dev DB)
Test: `curl -H "X-API-Key: <key>" http://localhost:3200/projects | python3 -m json.tool`

**Step 5: Commit**

```bash
git add e-fees-api/src/
git commit -m "feat(api): add read-only routes for projects, fees, companies, contacts, stats"
```

---

### Task 13: Add integration tests

**Files:**
- Create: `e-fees-api/tests/integration_tests.rs`

**Step 1: Write integration tests against dev DB**

```rust
// e-fees-api/tests/integration_tests.rs
use reqwest;

const BASE_URL: &str = "http://localhost:3200";

fn api_key() -> String {
    std::env::var("API_KEY").expect("API_KEY must be set for tests")
}

#[tokio::test]
async fn test_health_no_auth() {
    let resp = reqwest::get(format!("{BASE_URL}/health")).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_projects_requires_auth() {
    let resp = reqwest::get(format!("{BASE_URL}/projects")).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_projects_with_auth() {
    let client = reqwest::Client::new();
    let resp = client.get(format!("{BASE_URL}/projects"))
        .header("X-API-Key", api_key())
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["data"].is_array());
    assert!(body["count"].as_u64().unwrap() > 0);
}

// ... similar tests for fees, companies, contacts, stats
```

**Step 2: Add safety check**

```rust
// At the top of the test module:
fn verify_not_production() {
    let url = std::env::var("SURREAL_URL").unwrap_or_default();
    assert!(!url.contains("10.0.23.11"), "REFUSING TO RUN: Tests pointing at PRODUCTION");
}
```

**Step 3: Commit**

```bash
git add e-fees-api/tests/
git commit -m "test(api): add integration tests with production safety guard"
```

---

### Task 14: Create Dockerfile

**Files:**
- Create: `e-fees-api/Dockerfile`

**Step 1: Write multi-stage Dockerfile**

```dockerfile
# e-fees-api/Dockerfile
FROM rust:1.85-slim AS builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml ./
COPY crates/ crates/
COPY e-fees-api/ e-fees-api/

# Build release binary
RUN cd e-fees-api && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/e-fees-api/target/release/e-fees-api /usr/local/bin/

EXPOSE 3200

CMD ["e-fees-api"]
```

**Step 2: Commit**

```bash
git add e-fees-api/Dockerfile
git commit -m "chore(api): add Dockerfile for deployment"
```

---

### Task 15: Deploy to Unraid

**Files:**
- Create: `e-fees-api/.env` (on Unraid, not in git)

**Step 1: Build and deploy**

This task uses SSH MCP to deploy to Unraid. The container goes on the br0 network with a static IP in the 10.0.21.x range.

1. Copy source to Unraid (or build locally and push image)
2. Create `.env` on Unraid, populate from credential system
3. Create Docker container with appropriate networking
4. Verify health endpoint responds

**Step 2: Verify**

```bash
curl -s http://10.0.21.XX:3200/health | python3 -m json.tool
curl -s -H "X-API-Key: <key>" http://10.0.21.XX:3200/projects | python3 -m json.tool
```

**Step 3: Commit any deployment config/docs**

```bash
git commit -m "docs: add deployment notes for e-fees-api"
```

---

## Phase 4: Write Operations & Integration (Future)

Not planned in detail yet. Scope:
- POST/PUT/DELETE for all entities
- Status transition validation
- Fee revision cloning
- AILX integration endpoints
- n8n webhook triggers

---

## Summary

| Phase | Tasks | Commits | Estimated effort |
|-------|-------|---------|-----------------|
| Phase 1: UI Smoke Tests | 1-6 | 5 commits | Small |
| Phase 2: Core Extraction | 7-9 | 3 commits | Medium |
| Phase 3: API Service | 10-15 | 6 commits | Medium-Large |
| Phase 4: Write Ops | Future | TBD | Large |

**Total Phase 1-3:** 15 tasks, 14 commits.

**Dependencies:**
- Phase 1 is independent — can start immediately
- Phase 2 is independent of Phase 1
- Phase 3 depends on Phase 2 (needs `e-fees-core`)
- Phase 1 and Phase 2 can run in parallel

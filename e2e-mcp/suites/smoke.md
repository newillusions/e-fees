# E-Fees Smoke Test Runbook

Manual smoke test runbook for Claude to execute using Tauri MCP tools against the running E-Fees desktop application.

This is a structured document with 10 validation checks plus a production safety check. Follow each step sequentially, executing the provided JavaScript code using the `mcp__tauri_mcp__execute_js` tool.

## Prerequisites

- E-Fees desktop application is running (`npm run tauri:dev` or installed app)
- Tauri MCP server is connected and available
- Application window is visible (not minimized)
- Database connection is configured and accessible

## Safety Check (MUST RUN FIRST)

**Purpose**: Prevent smoke tests from running against production database.

**Execute this JavaScript code:**

```javascript
// Get current app URL/location to verify we're not on production
const result = {
  url: window.location.href || 'unknown',
  hostname: window.location.hostname || 'unknown',
  isProduction: false
};

// Check if connecting to production DB (10.0.23.11)
// If this check returns isProduction = true, STOP IMMEDIATELY
if (result.url.includes('10.0.23.11') || result.hostname.includes('10.0.23.11')) {
  result.isProduction = true;
  result.error = 'PRODUCTION DATABASE DETECTED - SMOKE TEST ABORTED';
}

result;
```

**Expected Result:**
- `isProduction` is **false**
- URL does NOT include `10.0.23.11` or production hostname
- No error message

**STOP IF:** `isProduction` is true. Do not proceed with any tests.

---

## Check 1: Database Connection Status

**Purpose**: Verify the database connection is active and responding.

**Execute this JavaScript code:**

```javascript
// Get database connection status
const status = await window.__TAURI_INVOKE__('get_connection_status');

const result = {
  connected: status.is_connected || false,
  lastCheck: status.last_check || null,
  errorMessage: status.error_message || null,
  timestamp: new Date().toISOString()
};

result;
```

**Expected Result:**
- `connected` is **true**
- `errorMessage` is **null** or empty
- `lastCheck` timestamp is recent (within last minute)

**Pass Criteria**: `connected === true`

---

## Check 2: Database Configuration Info

**Purpose**: Verify database configuration is correct and connection details are accessible.

**Execute this JavaScript code:**

```javascript
// Get detailed database configuration information
const dbInfo = await window.__TAURI_INVOKE__('get_db_info');

const result = {
  url: dbInfo.url || 'unknown',
  namespace: dbInfo.namespace || 'unknown',
  database: dbInfo.database || 'unknown',
  username: dbInfo.username || 'unknown',
  connectionStatus: {
    isConnected: dbInfo.connection_status?.is_connected || false,
    lastCheck: dbInfo.connection_status?.last_check || null
  },
  timestamp: dbInfo.timestamp || null,
  valid: false
};

// Validate configuration
result.valid =
  dbInfo.url &&
  dbInfo.namespace &&
  dbInfo.database &&
  result.connectionStatus.isConnected;

result;
```

**Expected Result:**
- `url` includes database server address (e.g., `ws://10.0.21.8:8000`)
- `namespace` is **emittiv**
- `database` is **projects**
- `username` is **martin**
- `connectionStatus.isConnected` is **true**
- `valid` is **true**

**Pass Criteria**: `valid === true && connectionStatus.isConnected === true`

---

## Check 3: Data Load Verification

**Purpose**: Verify that all main entities load from the database and return data.

**Execute this JavaScript code:**

```javascript
// Load all main entities and count records
const projects = await window.__TAURI_INVOKE__('get_projects');
const companies = await window.__TAURI_INVOKE__('get_companies');
const fees = await window.__TAURI_INVOKE__('get_fees');
const contacts = await window.__TAURI_INVOKE__('get_contacts');

const result = {
  projectsCount: Array.isArray(projects) ? projects.length : 0,
  companiesCount: Array.isArray(companies) ? companies.length : 0,
  feesCount: Array.isArray(fees) ? fees.length : 0,
  contactsCount: Array.isArray(contacts) ? contacts.length : 0,
  totalRecords: 0,
  allPositive: false
};

result.totalRecords =
  result.projectsCount +
  result.companiesCount +
  result.feesCount +
  result.contactsCount;

result.allPositive =
  result.projectsCount > 0 &&
  result.companiesCount > 0 &&
  result.feesCount > 0 &&
  result.contactsCount > 0;

result;
```

**Expected Result:**
- `projectsCount` > 0
- `companiesCount` > 0
- `feesCount` > 0
- `contactsCount` > 0
- `totalRecords` > 0
- `allPositive` is **true**

**Pass Criteria**: `allPositive === true && totalRecords > 0`

---

## Check 4: Project Status Values Validation

**Purpose**: Verify all projects have valid status values from the domain model.

**Execute this JavaScript code:**

```javascript
// Load projects and validate status values
const projects = await window.__TAURI_INVOKE__('get_projects');

const validStatuses = [
  'Lead',
  'RFP',
  'Submitted',
  'Awarded',
  'Design',
  'Construction',
  'Completed',
  'Lost',
  'No Response',
  'Cancelled',
  'On Hold',
  'Superseded'
];

const result = {
  totalProjects: projects.length,
  validProjects: 0,
  invalidStatuses: [],
  projectsWithInvalidStatus: []
};

projects.forEach(project => {
  if (validStatuses.includes(project.status)) {
    result.validProjects++;
  } else {
    result.invalidStatuses.push(project.status);
    result.projectsWithInvalidStatus.push({
      id: project.id,
      name: project.name,
      status: project.status
    });
  }
});

result.allValid = result.validProjects === result.totalProjects;
result.invalidCount = result.projectsWithInvalidStatus.length;

result;
```

**Expected Result:**
- `totalProjects` > 0
- `validProjects` equals `totalProjects`
- `allValid` is **true**
- `invalidStatuses` array is **empty**
- `projectsWithInvalidStatus` array is **empty**
- `invalidCount` is **0**

**Pass Criteria**: `allValid === true && invalidCount === 0`

---

## Check 5: Fee Status Values Validation

**Purpose**: Verify all fees have valid status values from the domain model.

**Execute this JavaScript code:**

```javascript
// Load fees and validate status values
const fees = await window.__TAURI_INVOKE__('get_fees');

const validStatuses = [
  'Draft',
  'Sent',
  'Negotiation',
  'Accepted',
  'Rejected',
  'No Response',
  'Superseded'
];

const result = {
  totalFees: fees.length,
  validFees: 0,
  invalidStatuses: [],
  feesWithInvalidStatus: []
};

fees.forEach(fee => {
  if (validStatuses.includes(fee.status)) {
    result.validFees++;
  } else {
    result.invalidStatuses.push(fee.status);
    result.feesWithInvalidStatus.push({
      id: fee.id,
      project_id: fee.project_id,
      status: fee.status
    });
  }
});

result.allValid = result.validFees === result.totalFees;
result.invalidCount = result.feesWithInvalidStatus.length;

result;
```

**Expected Result:**
- `totalFees` > 0
- `validFees` equals `totalFees`
- `allValid` is **true**
- `invalidStatuses` array is **empty**
- `feesWithInvalidStatus` array is **empty**
- `invalidCount` is **0**

**Pass Criteria**: `allValid === true && invalidCount === 0`

---

## Check 6: Dashboard Route Renders

**Purpose**: Verify the dashboard (#/) renders properly with content.

**Execute this JavaScript code:**

```javascript
// Navigate to dashboard and verify DOM content
window.location.hash = '#/';

// Wait for navigation (500ms delay for route change)
await new Promise(resolve => setTimeout(resolve, 500));

// Get current DOM and check for dashboard content
const dom = document.documentElement.outerHTML;
const location = window.location.href;

const result = {
  currentRoute: location,
  isDashboard: location.includes('#/') && !location.includes('/projects') && !location.includes('/companies'),
  hasMainContent: dom.includes('dashboard') || dom.includes('Dashboard') || dom.includes('stats') || dom.includes('Stats'),
  domLength: dom.length,
  timestamp: new Date().toISOString()
};

result.renderSuccess = result.isDashboard && result.hasMainContent && result.domLength > 1000;

result;
```

**Expected Result:**
- `isDashboard` is **true**
- `hasMainContent` is **true**
- `domLength` > 1000 (DOM has content)
- `renderSuccess` is **true**

**Pass Criteria**: `renderSuccess === true`

---

## Check 7: Projects Route Renders

**Purpose**: Verify the projects (#/projects) route renders with data.

**Execute this JavaScript code:**

```javascript
// Navigate to projects and verify DOM content
window.location.hash = '#/projects';

// Wait for navigation and data load
await new Promise(resolve => setTimeout(resolve, 800));

// Get current DOM and check for projects content
const dom = document.documentElement.outerHTML;
const location = window.location.href;

const result = {
  currentRoute: location,
  isProjects: location.includes('#/projects'),
  hasTable: dom.includes('<table') || dom.includes('table'),
  hasProjectData: dom.includes('project') || dom.includes('Project'),
  domLength: dom.length,
  timestamp: new Date().toISOString()
};

result.renderSuccess = result.isProjects && (result.hasTable || result.hasProjectData) && result.domLength > 1500;

result;
```

**Expected Result:**
- `isProjects` is **true**
- `hasProjectData` is **true**
- `domLength` > 1500 (DOM has significant content)
- `renderSuccess` is **true**

**Pass Criteria**: `renderSuccess === true`

---

## Check 8: Companies Route Renders

**Purpose**: Verify the companies (#/companies) route renders with data.

**Execute this JavaScript code:**

```javascript
// Navigate to companies and verify DOM content
window.location.hash = '#/companies';

// Wait for navigation and data load
await new Promise(resolve => setTimeout(resolve, 800));

// Get current DOM and check for companies content
const dom = document.documentElement.outerHTML;
const location = window.location.href;

const result = {
  currentRoute: location,
  isCompanies: location.includes('#/companies'),
  hasCompanyData: dom.includes('company') || dom.includes('Company'),
  domLength: dom.length,
  timestamp: new Date().toISOString()
};

result.renderSuccess = result.isCompanies && result.hasCompanyData && result.domLength > 1500;

result;
```

**Expected Result:**
- `isCompanies` is **true**
- `hasCompanyData` is **true**
- `domLength` > 1500
- `renderSuccess` is **true**

**Pass Criteria**: `renderSuccess === true`

---

## Check 9: Contacts Route Renders

**Purpose**: Verify the contacts (#/contacts) route renders with data.

**Execute this JavaScript code:**

```javascript
// Navigate to contacts and verify DOM content
window.location.hash = '#/contacts';

// Wait for navigation and data load
await new Promise(resolve => setTimeout(resolve, 800));

// Get current DOM and check for contacts content
const dom = document.documentElement.outerHTML;
const location = window.location.href;

const result = {
  currentRoute: location,
  isContacts: location.includes('#/contacts'),
  hasContactData: dom.includes('contact') || dom.includes('Contact'),
  domLength: dom.length,
  timestamp: new Date().toISOString()
};

result.renderSuccess = result.isContacts && result.hasContactData && result.domLength > 1500;

result;
```

**Expected Result:**
- `isContacts` is **true**
- `hasContactData` is **true**
- `domLength` > 1500
- `renderSuccess` is **true**

**Pass Criteria**: `renderSuccess === true`

---

## Check 10: Proposals Route Renders

**Purpose**: Verify the proposals (#/proposals) route renders with data.

**Execute this JavaScript code:**

```javascript
// Navigate to proposals and verify DOM content
window.location.hash = '#/proposals';

// Wait for navigation and data load
await new Promise(resolve => setTimeout(resolve, 800));

// Get current DOM and check for proposals content
const dom = document.documentElement.outerHTML;
const location = window.location.href;

const result = {
  currentRoute: location,
  isProposals: location.includes('#/proposals'),
  hasProposalData: dom.includes('proposal') || dom.includes('Proposal') || dom.includes('fee') || dom.includes('Fee'),
  domLength: dom.length,
  timestamp: new Date().toISOString()
};

result.renderSuccess = result.isProposals && result.hasProposalData && result.domLength > 1500;

result;
```

**Expected Result:**
- `isProposals` is **true**
- `hasProposalData` is **true**
- `domLength` > 1500
- `renderSuccess` is **true**

**Pass Criteria**: `renderSuccess === true`

---

## Check 11: Entity Counts Consistency

**Purpose**: Verify that application statistics match the actual entity counts from data loading.

**Execute this JavaScript code:**

```javascript
// Get statistics from backend
const stats = await window.__TAURI_INVOKE__('get_stats');

// Load actual entities to compare counts
const projects = await window.__TAURI_INVOKE__('get_projects');
const companies = await window.__TAURI_INVOKE__('get_companies');
const contacts = await window.__TAURI_INVOKE__('get_contacts');
const fees = await window.__TAURI_INVOKE__('get_fees');

const result = {
  statsProjects: stats.totalProjects || 0,
  statsCompanies: stats.totalCompanies || 0,
  statsContacts: stats.totalContacts || 0,
  statsFees: stats.totalFees || 0,
  actualProjects: projects.length,
  actualCompanies: companies.length,
  actualContacts: contacts.length,
  actualFees: fees.length,
  matches: {}
};

result.matches.projects = result.statsProjects === result.actualProjects;
result.matches.companies = result.statsCompanies === result.actualCompanies;
result.matches.contacts = result.statsContacts === result.actualContacts;
result.matches.fees = result.statsFees === result.actualFees;

result.allMatch =
  result.matches.projects &&
  result.matches.companies &&
  result.matches.contacts &&
  result.matches.fees;

result;
```

**Expected Result:**
- `matches.projects` is **true**
- `matches.companies` is **true**
- `matches.contacts` is **true**
- `matches.fees` is **true**
- `allMatch` is **true**

**Pass Criteria**: `allMatch === true`

---

## Results Summary

**Instructions**:
1. Execute each check sequentially (Checks 1-11)
2. For each check, copy the JavaScript code into `mcp__tauri_mcp__execute_js` tool
3. Record the result (PASS or FAIL) for each check
4. Note any error messages or unexpected values

**Score Card** (fill in as you go):

| Check | Description | Status | Notes |
|-------|-------------|--------|-------|
| Safety | Production DB check | ⚠️ MUST PASS | |
| 1 | Database Connection | __ PASS / __ FAIL | |
| 2 | Database Configuration | __ PASS / __ FAIL | |
| 3 | Data Load Verification | __ PASS / __ FAIL | |
| 4 | Project Status Values | __ PASS / __ FAIL | |
| 5 | Fee Status Values | __ PASS / __ FAIL | |
| 6 | Dashboard Renders | __ PASS / __ FAIL | |
| 7 | Projects Route Renders | __ PASS / __ FAIL | |
| 8 | Companies Route Renders | __ PASS / __ FAIL | |
| 9 | Contacts Route Renders | __ PASS / __ FAIL | |
| 10 | Proposals Route Renders | __ PASS / __ FAIL | |
| 11 | Entity Counts Match | __ PASS / __ FAIL | |

**Overall Result**:

- **✅ PASS**: All 11 checks passed (+ Safety check passed)
- **⚠️ PARTIAL**: Some checks failed (list which)
- **❌ FAIL**: Multiple checks failed or Safety check failed

**Common Failure Causes**:

- **Database Connection Failed** → Verify SurrealDB is running and accessible at configured URL
- **Status Value Mismatch** → May indicate database migration issue or schema mismatch
- **Route Not Rendering** → Check browser console for JavaScript errors
- **Data Count Mismatch** → May indicate transaction or race condition, try reconnecting
- **Production Database Detected** → ABORT and switch to dev database

---

## How to Use This Runbook

1. **For Local Development**: Run after `npm run tauri:dev` with dev database (10.0.21.8:8000)
2. **For Release Validation**: Run against release build before publishing
3. **For Bug Triage**: Use to quickly verify if issue is database-related or application UI
4. **For CI/CD**: Can be automated with Tauri MCP test runner

## See Also

- [E2E MCP Testing Documentation](../README.md) — Full testing infrastructure guide
- [DATABASE_SCHEMA.md](../../DATABASE_SCHEMA.md) — Complete schema reference
- [Development Workflow Rules](./.claude/rules/development-workflow.md) — Testing requirements

---

**Last Updated**: February 28, 2026
**Version**: 1.0
**Purpose**: Manual smoke test validation using Tauri MCP tools

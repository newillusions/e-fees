# E2E Testing with Tauri MCP

**CORRECT** E2E testing implementation using Tauri MCP server for real application interaction.

## Overview

This directory contains the **proper** E2E testing infrastructure that:
- Tests the **actual** desktop application (not browser simulation)
- Interacts with the **real** Tauri backend and SurrealDB database
- Uses **safe** test data with "DELETE ME" identification
- Provides **cleanup utilities** for test data management

## Directory Structure

```
e2e-mcp/
├── config/
│   ├── mcp-setup.ts          # MCP server configuration
│   ├── test-database.ts      # Safe database operations
│   └── app-launcher.ts       # Application launching utilities
├── fixtures/
│   ├── test-data-safe.ts     # "DELETE ME" test data generators
│   ├── cleanup-utilities.ts  # Test data removal tools
│   └── reference-data.ts     # Countries, currencies, etc.
├── helpers/
│   ├── mcp-client.ts         # MCP communication layer
│   ├── app-interaction.ts    # Real application interaction
│   ├── database-helpers.ts   # Database verification
│   └── screenshot-manager.ts # Test documentation
├── tests/
│   ├── setup.mcp.ts          # Environment verification
│   ├── project-crud.mcp.ts   # Real project operations
│   ├── company-crud.mcp.ts   # Real company operations
│   ├── contact-crud.mcp.ts   # Real contact operations
│   ├── proposal-crud.mcp.ts  # Real proposal operations
│   └── navigation.mcp.ts     # Real keyboard shortcuts
└── scripts/
    ├── run-tests.sh          # Test execution
    ├── cleanup.sh            # Data cleanup
    └── setup-environment.sh  # Environment setup
```

## Quick Start

### 1. Setup Environment
```bash
# Build MCP server
cd tauri-plugin-mcp/mcp-server-ts
npm install
npm run build

# Setup test environment
./e2e-mcp/scripts/setup-environment.sh
```

### 2. Run Tests
```bash
# Run all MCP-based E2E tests
npm run test:e2e:mcp

# Run specific test suite
npm run test:e2e:mcp -- --grep "Project CRUD"

# Run with cleanup
npm run test:e2e:safe
```

### 3. Cleanup Test Data
```bash
# Clean up all "DELETE ME" test records
npm run test:e2e:cleanup

# Verify cleanup
npm run test:e2e:verify-clean
```

## Key Features

### ✅ Real Application Testing
- Tests actual Tauri desktop application
- Uses real Rust backend commands
- Interacts with live SurrealDB database
- Tests native keyboard shortcuts and window management

### ✅ Safe Test Data Management
- All test records include "DELETE ME" prefix
- Automatic cleanup utilities
- Timestamp-based identification
- Rollback capabilities for failed tests

### ✅ Comprehensive Coverage
- Project lifecycle (create, edit, delete, file operations)
- Company/Contact management with relationships
- Proposal workflows end-to-end
- Navigation and keyboard shortcuts
- Error scenarios and recovery

### ✅ Production-Safe Testing
- Uses live database with safety measures
- Non-destructive test data identification
- Easy cleanup and verification
- Detailed test documentation via screenshots

## Test Data Safety

### Mandatory "DELETE ME" Prefix
ALL test data must include "DELETE ME" in name fields:

```typescript
// Project Test Data
const testProject = {
  name: "DELETE ME - Test UAE Office Project 2025-08-21-14:30:15",
  description: "DELETE ME - Testing project creation workflow",
  // ... other fields
}

// Company Test Data
const testCompany = {
  name: "DELETE ME - Test Construction LLC",
  description: "DELETE ME - E2E testing company",
  // ... other fields
}
```

### Cleanup Commands
```bash
# List all test data
npm run test:e2e:list-test-data

# Clean specific type
npm run test:e2e:cleanup:projects
npm run test:e2e:cleanup:companies
npm run test:e2e:cleanup:contacts
npm run test:e2e:cleanup:proposals

# Nuclear cleanup (all test data)
npm run test:e2e:cleanup:all
```

## MCP Integration

### Available MCP Tools
- `take_screenshot` - Document test state
- `execute_js` - Run code in app context
- `get_dom` - Access real application DOM
- `text_input` - Simulate user typing
- `mouse_movement` - Click, drag, scroll
- `send_text_to_element` - Direct element interaction
- `manage_window` - Window positioning/sizing

### MCP Communication Example
```typescript
import { MCPClient } from './helpers/mcp-client'

const mcp = new MCPClient()

// Take screenshot for documentation
await mcp.takeScreenshot('before-project-create')

// Navigate using real keyboard shortcut
await mcp.keyboardShortcut('Cmd+2') // Projects page

// Click actual button in running app
await mcp.clickElement('[data-testid="create-project-btn"]')

// Fill form with safe test data
await mcp.typeText('DELETE ME - Test Project UAE Office')

// Verify in database using SurrealDB MCP
const result = await surrealMCP.select({ 
  table: 'projects',
  query: 'name CONTAINS "DELETE ME - Test Project UAE Office"'
})
```

## Error Handling

### Robust Test Recovery
- Automatic screenshot capture on failure
- Test data cleanup even after crashes
- Detailed error logging and context
- Rollback capabilities for partial operations

### Debugging Support
```bash
# Run with debug logging
npm run test:e2e:mcp:debug

# Interactive test mode
npm run test:e2e:mcp:interactive

# Screenshot-only mode (no interaction)
npm run test:e2e:screenshot-only
```

## Performance Monitoring

### Test Execution Metrics
- Real application startup time
- Database operation performance
- UI interaction response times
- Memory usage tracking

### Performance Tests
```typescript
// Measure project creation performance
const startTime = performance.now()
await mcp.createProject(testData)
const endTime = performance.now()
expect(endTime - startTime).toBeLessThan(2000) // < 2 seconds
```

## CI/CD Integration

### GitHub Actions
```yaml
# .github/workflows/e2e-mcp.yml
name: E2E Tests (MCP)
on: [push, pull_request]
jobs:
  e2e-mcp:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
      - name: Install dependencies
        run: npm ci
      - name: Build Tauri app
        run: npm run tauri:build
      - name: Setup MCP server
        run: ./e2e-mcp/scripts/setup-environment.sh
      - name: Run MCP E2E tests
        run: npm run test:e2e:mcp
      - name: Cleanup test data
        if: always()
        run: npm run test:e2e:cleanup
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-mcp-results
          path: e2e-mcp/results/
```

## Migration from Browser-Based Tests

### What Changed
- **FROM**: Browser Playwright automation
- **TO**: Tauri MCP real application interaction

### Test Conversion Example
```typescript
// OLD (Browser-based - WRONG)
test('should create project', async ({ page }) => {
  await page.goto('http://localhost:1420')
  await page.locator('[data-testid="create-project-btn"]').click()
  await page.locator('input[name="project_name"]').fill('Test Project')
  await page.locator('button[type="submit"]').click()
  await expect(page.locator('.success-message')).toBeVisible()
})

// NEW (MCP-based - CORRECT)
test('should create project', async () => {
  const mcp = new MCPClient()
  const projectData = generateSafeProjectData()
  
  await mcp.takeScreenshot('before-project-create')
  const result = await mcp.createProject(projectData)
  
  expect(result.success).toBe(true)
  expect(result.projectNumber).toMatch(/25-971\d{2}/)
  
  // Verify in database
  const dbProject = await surrealMCP.select({
    table: 'projects',
    query: `name = "${projectData.name}"`
  })
  expect(dbProject).toBeDefined()
})
```

## Best Practices

### 1. Always Use Safe Test Data
```typescript
// ✅ CORRECT - Always include "DELETE ME"
const projectName = generateSafeTestName("UAE Office Project")
// Returns: "DELETE ME - UAE Office Project 2025-08-21-14:30:15"

// ❌ WRONG - Never use production-like names
const projectName = "UAE Office Project" // Could be confused with real data
```

### 2. Screenshot Everything
```typescript
test('project workflow', async () => {
  await mcp.takeScreenshot('start-state')
  await mcp.navigateToProjects()
  await mcp.takeScreenshot('projects-page-loaded')
  await mcp.createProject(testData)
  await mcp.takeScreenshot('project-created')
  // Screenshots saved to e2e-mcp/results/screenshots/
})
```

### 3. Always Cleanup
```typescript
afterEach(async () => {
  // Cleanup any data created during test
  await cleanupTestData(testRunId)
})

afterAll(async () => {
  // Verify all test data was removed
  const remainingTestData = await findTestData()
  expect(remainingTestData.length).toBe(0)
})
```

### 4. Verify Database State
```typescript
test('project creation', async () => {
  const beforeCount = await getProjectCount()
  await mcp.createProject(testData)
  const afterCount = await getProjectCount()
  
  expect(afterCount).toBe(beforeCount + 1)
  
  // Verify specific record exists
  const project = await findProjectByName(testData.name)
  expect(project).toBeDefined()
  expect(project.status).toBe('active')
})
```

## Troubleshooting

### Common Issues

**MCP Connection Failed**
```bash
# Check if MCP server is running
ls -la /tmp/tauri-mcp-e2e.sock

# Rebuild MCP server
cd tauri-plugin-mcp/mcp-server-ts
npm run build
```

**App Not Responding**
```bash
# Kill any hanging processes
pkill -f "e-fees"
pkill -f "tauri"

# Restart test environment
./e2e-mcp/scripts/setup-environment.sh
```

**Test Data Not Cleaning Up**
```bash
# Manual cleanup
npm run test:e2e:cleanup:all

# Verify database state
npm run test:e2e:verify-clean
```

---

## Summary

This MCP-based E2E testing approach provides **real** application testing with:
- ✅ Actual desktop application interaction
- ✅ True backend and database testing  
- ✅ Safe test data management
- ✅ Production-safe testing practices
- ✅ Comprehensive cleanup utilities

**Use this instead of the broken browser-based tests for reliable E2E testing.**
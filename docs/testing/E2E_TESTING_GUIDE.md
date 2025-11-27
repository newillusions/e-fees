# End-to-End Testing Guide

## Overview

This document provides comprehensive guidance for the E2E testing infrastructure implemented for the E-Fees Tauri application. The solution uses Playwright with custom Tauri-specific helpers to test complete user workflows from the desktop application interface to database persistence.

## Table of Contents

1. [Architecture](#architecture)
2. [Quick Start](#quick-start)
3. [Test Environment Setup](#test-environment-setup)
4. [Running Tests](#running-tests)
5. [Writing Tests](#writing-tests)
6. [Test Patterns and Best Practices](#test-patterns-and-best-practices)
7. [CI/CD Integration](#cicd-integration)
8. [Troubleshooting](#troubleshooting)
9. [Maintenance](#maintenance)

## Architecture

### Technology Stack

- **Test Framework**: Playwright v1.54.1 (already installed)
- **Application**: Tauri v2 desktop application
- **Database**: Isolated SurrealDB test instance (port 8001)
- **Test Organization**: Page Object Model with fixtures and helpers
- **Reporting**: HTML, JSON, and JUnit reports with video/screenshot capture

### Directory Structure

```
e2e/
├── config/
│   ├── global-setup.ts           # Test environment initialization
│   └── global-teardown.ts        # Cleanup and reporting
├── fixtures/
│   ├── test-data.ts              # Test data definitions
│   └── database-fixtures.ts      # Database state management
├── helpers/
│   ├── page-objects.ts           # Page Object Models
│   └── tauri-helpers.ts          # Tauri-specific utilities
├── tests/
│   ├── setup.spec.ts             # Environment verification
│   ├── app-startup.spec.ts       # Application launch tests
│   ├── project-management.spec.ts # Project CRUD workflows
│   ├── company-contact.spec.ts   # Company/Contact management
│   ├── proposal-workflows.spec.ts # Proposal creation (to be added)
│   ├── navigation.spec.ts        # Navigation and shortcuts (to be added)
│   └── error-scenarios.spec.ts   # Error handling (to be added)
├── reports/                      # Test output and artifacts
└── test-results/                 # Screenshots, videos, traces
```

### Key Components

1. **Global Setup/Teardown**: Manages SurrealDB test instance and environment
2. **Database Fixtures**: Handles test data seeding and cleanup
3. **Page Objects**: Encapsulates UI interactions and assertions
4. **Tauri Helpers**: Manages app lifecycle and native functionality

## Quick Start

### Prerequisites

- **SurrealDB**: Must be installed and available in PATH
- **Node.js**: Version 20+ recommended
- **Rust**: Required for Tauri build process

### Installation

SurrealDB installation varies by platform:

```bash
# macOS (Homebrew)
brew install surrealdb/tap/surreal

# Ubuntu/Linux
curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh

# Windows
# Download from https://surrealdb.com/install
```

### Running Your First E2E Test

```bash
# Run all E2E tests
npm run test:e2e

# Run tests with UI (great for debugging)
npm run test:e2e:ui

# Run tests in development mode (uses running dev server)
npm run test:e2e:dev

# Run specific test file
npm run test:e2e -- app-startup.spec.ts
```

## Test Environment Setup

### Automatic Setup

The E2E infrastructure automatically handles:

1. **SurrealDB Instance**: Starts isolated test database on port 8001
2. **Test Database**: Initializes with reference data (countries, currencies)
3. **Environment Variables**: Sets test-specific configuration
4. **Project Folders**: Creates temporary folder structure
5. **App Building**: Builds Tauri application for testing (if needed)

### Manual Setup (for debugging)

```bash
# 1. Start SurrealDB test instance manually
surreal start --log warn --user root --pass test --bind 0.0.0.0:8001 memory

# 2. Set environment variables
export SURREALDB_URL=ws://localhost:8001
export SURREALDB_NS=test
export SURREALDB_DB=e2e
export SURREALDB_USER=root
export SURREALDB_PASS=test
export PROJECT_FOLDER_PATH=/tmp/test-projects
export NODE_ENV=test

# 3. Run tests in development mode
npm run test:e2e:dev
```

## Running Tests

### Available Scripts

| Command | Description | Use Case |
|---------|-------------|----------|
| `npm run test:e2e` | Run all E2E tests | CI/CD, full validation |
| `npm run test:e2e:ui` | Interactive test runner | Test development, debugging |
| `npm run test:e2e:dev` | Use running dev server | Development workflow |
| `npm run test:e2e:headed` | Run with browser visible | Debugging, demonstration |
| `npm run test:e2e:debug` | Debug mode with step-through | Troubleshooting |
| `npm run test:all` | Unit tests + E2E tests | Complete validation |

### Test Execution Modes

#### Development Mode (`test:e2e:dev`)
- Uses running `npm run tauri:dev` server
- Faster iteration cycle
- Hot reload capabilities
- Best for test development

#### Production Mode (`test:e2e`)
- Builds fresh Tauri application
- Tests actual production build
- Slower but more reliable
- Best for CI/CD and final validation

### Filtering Tests

```bash
# Run specific test file
npm run test:e2e -- app-startup.spec.ts

# Run tests matching pattern
npm run test:e2e -- --grep "project"

# Run tests in specific project
npm run test:e2e -- --project app-startup

# Skip setup tests (for faster iteration)
npm run test:e2e -- --ignore-pattern="setup.spec.ts"
```

## Writing Tests

### Basic Test Structure

```typescript
import { test, expect } from '@playwright/test'
import { TauriApp } from '../helpers/tauri-helpers'
import { NavigationPage, ProjectsPage } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

test.describe('Feature Name', () => {
  let tauriApp: TauriApp
  let dbFixtures: DatabaseFixtures

  test.beforeAll(async () => {
    tauriApp = new TauriApp(process.env.E2E_DEV_MODE === 'true')
  })

  test.beforeEach(async ({ page }) => {
    dbFixtures = new DatabaseFixtures(page)
    await dbFixtures.clearAllData()
    await dbFixtures.initializeReferenceData()
  })

  test.afterAll(async () => {
    if (tauriApp) {
      await tauriApp.close()
    }
  })

  test('should perform specific action', async ({ browser }) => {
    // Launch app if needed
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    // Connect to running app
    const page = await tauriApp.connect(browser)
    
    // Use page objects for interactions
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    
    // Test workflow
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    // ... test logic
    
    // Cleanup
    await page.close()
  })
})
```

### Using Database Fixtures

```typescript
// Clear all data and start fresh
await dbFixtures.clearAllData()
await dbFixtures.initializeReferenceData()

// Seed with minimal data (1 of each entity type)
await dbFixtures.seedMinimalData()

// Seed with complete test dataset
await dbFixtures.seedCompleteData()

// Seed with relationships established
await dbFixtures.seedLinkedData()

// Verify database state
const counts = await dbFixtures.getDatabaseCounts()
expect(counts.companies).toBe(3)
```

### Using Page Objects

```typescript
// Navigation
const navPage = new NavigationPage(page)
await navPage.navigateToProjects()
await navPage.useKeyboardShortcut('2') // Cmd/Ctrl+2

// Projects page interactions
const projectsPage = new ProjectsPage(page)
await projectsPage.clickNewProject()
await projectsPage.searchProjects('Dubai')
const count = await projectsPage.getProjectsCount()

// Modal interactions
const projectModal = new ProjectModal(page)
await projectModal.fillProjectForm({
  name: 'Test Project',
  shortName: 'TP',
  area: 'Business Bay',
  city: 'Dubai',
  country: 'United Arab Emirates'
})
await projectModal.saveProject()
```

### Testing Tauri-Specific Features

```typescript
const helpers = new TauriTestHelpers(page)

// Test keyboard shortcuts
await helpers.testKeyboardShortcut('1') // Dashboard

// Test window positioning
await helpers.testWindowPositioning()

// Test file system integration
const folderExists = await helpers.testFileOperations('Project Name')

// Test responsive behavior
await helpers.testResponsiveBehavior()

// Capture performance metrics
const metrics = await helpers.capturePerformanceMetrics()
```

## Test Patterns and Best Practices

### 1. Test Independence

```typescript
test.beforeEach(async ({ page }) => {
  // Always start with clean state
  await dbFixtures.clearAllData()
  await dbFixtures.initializeReferenceData()
})
```

### 2. Proper Wait Strategies

```typescript
// Wait for app to be fully ready
await tauriApp.waitForReady(page)

// Wait for specific app states
await helpers.waitForAppState('ready')

// Wait for database operations
await page.waitForTimeout(1000) // Allow data to persist
```

### 3. Error Handling

```typescript
test('should handle connection failure', async ({ browser }) => {
  // Test error scenarios
  await helpers.simulateConnectionLoss()
  await helpers.waitForAppState('error')
  
  // Verify graceful error handling
  await expect(page.locator('[data-testid="error-message"]')).toBeVisible()
})
```

### 4. Data Validation

```typescript
test('should persist data correctly', async ({ browser }) => {
  // Create data
  await projectModal.saveProject()
  
  // Verify in database
  const counts = await dbFixtures.getDatabaseCounts()
  expect(counts.projects).toBe(1)
  
  // Verify in UI
  await projectsPage.verifyProjectExists('Test Project')
})
```

### 5. Progressive Enhancement Testing

```typescript
test.describe('Feature with fallbacks', () => {
  test('should work with full functionality', async ({ browser }) => {
    // Test with all features enabled
  })
  
  test('should degrade gracefully', async ({ browser }) => {
    // Test with limited functionality
  })
})
```

## CI/CD Integration

### GitHub Actions Workflow

The E2E tests are integrated into CI/CD via `.github/workflows/e2e-tests.yml`:

**Features:**
- Multi-platform testing (Ubuntu, macOS, Windows)
- Automatic dependency management
- Test result artifacts
- Performance monitoring
- PR comment integration
- Security scanning

**Triggers:**
- Push to main/develop branches
- Pull requests to main
- Manual workflow dispatch

### Test Artifacts

Generated artifacts include:
- Test result reports (HTML, JSON, JUnit)
- Screenshots and videos of failures
- Performance metrics
- Coverage reports
- Test summaries

### Environment Variables for CI

```yaml
env:
  SURREALDB_URL: ws://localhost:8001
  SURREALDB_NS: test
  SURREALDB_DB: e2e
  SURREALDB_USER: root
  SURREALDB_PASS: test
  CI: true
  PLAYWRIGHT_BROWSERS_PATH: ${{ github.workspace }}/pw-browsers
```

## Troubleshooting

### Common Issues

#### 1. SurrealDB Connection Failures

```bash
# Check if SurrealDB is running
curl http://localhost:8001/version

# Manual start if needed
surreal start --user root --pass test --bind 0.0.0.0:8001 memory
```

#### 2. App Launch Timeout

```typescript
// Increase timeout in tauri-helpers.ts
await this.waitForReady(page, 60000) // 60 seconds instead of 30
```

#### 3. Port Conflicts

```bash
# Check for processes using port 8001
lsof -i :8001

# Kill conflicting processes
pkill -f "surreal.*8001"
```

#### 4. Permission Issues (Linux)

```bash
# Install system dependencies
sudo apt-get install -y libwebkit2gtk-4.1-dev xvfb

# Run with virtual display
xvfb-run -a npm run test:e2e
```

### Debug Mode

```bash
# Run with step-through debugging
npm run test:e2e:debug

# Run with browser visible
npm run test:e2e:headed

# Verbose logging
DEBUG=pw:api npm run test:e2e
```

### Test Data Issues

```typescript
// Verify database state
const state = await dbFixtures.verifyDatabaseState()
console.log('Database state:', state)

// Check specific counts
const counts = await dbFixtures.getDatabaseCounts()
console.log('Entity counts:', counts)
```

## Maintenance

### Regular Tasks

#### 1. Update Test Data
- Review and update test data in `e2e/fixtures/test-data.ts`
- Ensure data reflects current business requirements
- Add new test scenarios for new features

#### 2. Update Page Objects
- Add new page objects for new features
- Update selectors when UI changes
- Maintain consistency with application updates

#### 3. Performance Monitoring
- Review test execution times
- Optimize slow tests
- Monitor resource usage

### Extending Tests

#### Adding New Test Files

1. Create new spec file in `e2e/tests/`
2. Follow naming convention: `feature-name.spec.ts`
3. Add to `playwright.config.ts` projects if needed
4. Update documentation

#### Adding New Page Objects

1. Add to `e2e/helpers/page-objects.ts`
2. Follow existing patterns
3. Use data-testid selectors
4. Include proper error handling

#### Adding New Test Data

1. Update `e2e/fixtures/test-data.ts`
2. Add corresponding fixtures methods
3. Document data relationships
4. Consider edge cases

### Version Updates

#### Playwright Updates

```bash
# Update Playwright
npm update playwright

# Update browser binaries
npx playwright install
```

#### Dependency Updates

```bash
# Check for updates
npm outdated

# Update all dependencies
npm update

# Test after updates
npm run test:all
```

### Performance Optimization

#### Test Execution Speed

1. **Parallel Execution**: Currently disabled for desktop app testing
2. **Database Optimization**: Use memory-based SurrealDB for speed
3. **App Reuse**: Consider app reuse patterns for faster test execution
4. **Selective Testing**: Use test filtering for faster iteration

#### Resource Management

1. **Memory Usage**: Monitor app and browser memory usage
2. **Cleanup**: Ensure proper cleanup of resources
3. **Artifacts**: Limit artifact retention to necessary items

## Best Practices Summary

1. **Test Structure**: Use describe blocks to group related tests
2. **Data Management**: Always start with clean database state
3. **Error Handling**: Test both success and failure paths
4. **Assertions**: Use specific assertions with clear messages
5. **Documentation**: Keep tests self-documenting with good naming
6. **Maintenance**: Regular review and updates of test suite

## Getting Help

- **Documentation**: This guide and inline code comments
- **Examples**: Look at existing test files for patterns
- **Debugging**: Use `npm run test:e2e:ui` for interactive debugging
- **Issues**: Check common troubleshooting section above
- **Updates**: Monitor Playwright and Tauri documentation for updates

---

**Last Updated**: August 21, 2025  
**Version**: 1.0.0  
**E2E Framework**: Playwright + Tauri + SurrealDB
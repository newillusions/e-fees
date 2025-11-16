# E-Fees Testing Strategy
**Comprehensive Testing Approach for Desktop Application**

## 🎯 Purpose

This document outlines the complete testing strategy for e-fees, covering unit tests, integration tests, and AI-driven end-to-end tests. The strategy emphasizes test data safety, comprehensive coverage, and efficient debugging workflows.

---

## 📊 Testing Pyramid

```
        ┌─────────────────┐
        │   E2E Tests     │  ← Few, critical paths (MCP-driven)
        │    (Slow)       │
        └─────────────────┘
               ▲
               │
        ┌─────────────────┐
        │ Integration     │  ← More, component interactions
        │   Tests         │
        └─────────────────┘
               ▲
               │
        ┌─────────────────┐
        │  Unit Tests     │  ← Many, individual functions
        │   (Fast)        │
        └─────────────────┘
```

### Test Distribution Target

| Test Type | Quantity | Coverage | Execution Time |
|-----------|----------|----------|----------------|
| Unit Tests | ~500-1000 | 80%+ of functions | < 30 seconds |
| Integration Tests | ~100-200 | Critical paths | < 2 minutes |
| E2E Tests | ~20-50 | User workflows | < 5 minutes |

---

## 🧪 Unit Testing

### Scope

**Unit tests validate individual functions in isolation.**

### Rust Unit Tests

**Location**: `src-tauri/src/**/*.rs` (inline with code)

```rust
// Example: src-tauri/src/db/contacts/queries.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_contact_valid_data() {
        // Arrange
        let db = setup_test_db().await;
        let contact = Contact {
            name: "DELETE ME - Test Contact".to_string(),
            email: "test@example.com".to_string(),
            ..Default::default()
        };
        
        // Act
        let result = create_contact(&db, contact).await;
        
        // Assert
        assert!(result.is_ok());
        let created = result.unwrap();
        assert!(created.id.is_some());
        assert_eq!(created.name, "DELETE ME - Test Contact");
        
        // Cleanup
        cleanup_test_db(&db).await;
    }
    
    #[tokio::test]
    async fn test_create_contact_invalid_email() {
        let db = setup_test_db().await;
        let contact = Contact {
            name: "Test".to_string(),
            email: "invalid-email".to_string(),
            ..Default::default()
        };
        
        let result = create_contact(&db, contact).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email"));
    }
    
    #[tokio::test]
    async fn test_get_contact_not_found() {
        let db = setup_test_db().await;
        
        let result = get_contact(&db, "nonexistent:id").await;
        
        assert!(result.is_err());
    }
}
```

**Run Rust unit tests**:
```bash
cargo test                    # All tests
cargo test contacts          # Specific module
cargo test test_create      # Specific test pattern
cargo test -- --nocapture   # Show println! output
```

### TypeScript/Svelte Unit Tests

**Location**: `src/lib/**/*.test.ts`

```typescript
// Example: src/lib/utils/validation.test.ts

import { describe, it, expect } from 'vitest'
import { validateEmail, validatePhone } from './validation'

describe('Email Validation', () => {
  it('accepts valid email addresses', () => {
    expect(validateEmail('user@example.com')).toBe(true)
    expect(validateEmail('test.user+tag@domain.co.uk')).toBe(true)
  })
  
  it('rejects invalid email addresses', () => {
    expect(validateEmail('invalid')).toBe(false)
    expect(validateEmail('no@domain')).toBe(false)
    expect(validateEmail('@nodomain.com')).toBe(false)
  })
  
  it('handles edge cases', () => {
    expect(validateEmail('')).toBe(false)
    expect(validateEmail(null)).toBe(false)
    expect(validateEmail(undefined)).toBe(false)
  })
})
```

**Run TypeScript unit tests**:
```bash
npm run test:unit              # All unit tests
npm run test:unit:watch       # Watch mode
npm run test:unit:coverage    # With coverage report
```

### Unit Test Best Practices

**DO**:
- ✅ Test one function per test file group
- ✅ Use descriptive test names (`test_create_contact_with_invalid_email`)
- ✅ Follow Arrange-Act-Assert pattern
- ✅ Test edge cases and error conditions
- ✅ Keep tests independent (no shared state)
- ✅ Use test fixtures and helpers
- ✅ Clean up test data after each test

**DON'T**:
- ❌ Test multiple functions in one test
- ❌ Depend on test execution order
- ❌ Use real external services (mock them)
- ❌ Leave test data in database
- ❌ Skip error path testing
- ❌ Write tests that depend on timing

---

## 🔗 Integration Testing

### Scope

**Integration tests validate interactions between components.**

### Tauri Command Integration Tests

**Location**: `src-tauri/tests/integration_*.rs`

```rust
// Example: src-tauri/tests/integration_contacts.rs

use tauri::test::{mock_builder, MockRuntime};
use e_fees::commands::contacts::*;

#[tokio::test]
async fn test_create_and_retrieve_contact_flow() {
    // Setup: Initialize app and database
    let app = mock_builder().build(tauri::generate_context!()).unwrap();
    let db = setup_integration_test_db().await;
    
    // Create contact via command
    let contact_data = Contact {
        name: format!("DELETE ME - Integration Test {}", chrono::Utc::now().timestamp()),
        email: format!("delete-me-{}@test.com", chrono::Utc::now().timestamp()),
        phone: Some("+1234567890".to_string()),
        ..Default::default()
    };
    
    let created = create_contact(contact_data.clone(), db.clone())
        .await
        .expect("Failed to create contact");
    
    assert!(created.id.is_some());
    
    // Retrieve via command
    let retrieved = get_contact(created.id.unwrap(), db.clone())
        .await
        .expect("Failed to retrieve contact");
    
    assert_eq!(retrieved.name, contact_data.name);
    assert_eq!(retrieved.email, contact_data.email);
    
    // Update via command
    let mut updated_data = retrieved.clone();
    updated_data.phone = Some("+0987654321".to_string());
    
    let updated = update_contact(updated_data, db.clone())
        .await
        .expect("Failed to update contact");
    
    assert_eq!(updated.phone.unwrap(), "+0987654321");
    
    // Delete via command
    delete_contact(created.id.unwrap(), db.clone())
        .await
        .expect("Failed to delete contact");
    
    // Verify deletion
    let deleted_result = get_contact(created.id.unwrap(), db.clone()).await;
    assert!(deleted_result.is_err());
    
    // Cleanup
    cleanup_integration_test_db(&db).await;
}

#[tokio::test]
async fn test_contact_company_relationship() {
    let app = mock_builder().build(tauri::generate_context!()).unwrap();
    let db = setup_integration_test_db().await;
    
    // Create company
    let company = create_company(Company {
        name: format!("DELETE ME - Test Company {}", chrono::Utc::now().timestamp()),
        ..Default::default()
    }, db.clone()).await.unwrap();
    
    // Create contact linked to company
    let contact = create_contact(Contact {
        name: format!("DELETE ME - Test Contact {}", chrono::Utc::now().timestamp()),
        email: format!("delete-me-{}@test.com", chrono::Utc::now().timestamp()),
        company_id: company.id.clone(),
        ..Default::default()
    }, db.clone()).await.unwrap();
    
    // Verify relationship
    let contacts_for_company = get_contacts_by_company(company.id.unwrap(), db.clone())
        .await
        .unwrap();
    
    assert_eq!(contacts_for_company.len(), 1);
    assert_eq!(contacts_for_company[0].id, contact.id);
    
    cleanup_integration_test_db(&db).await;
}
```

**Run integration tests**:
```bash
cargo test --test integration_*              # All integration tests
cargo test --test integration_contacts      # Specific integration test
```

### Database Integration Tests

**Focus**: Query correctness, transaction handling, error cases

```rust
#[tokio::test]
async fn test_transaction_rollback_on_error() {
    let db = setup_integration_test_db().await;
    
    let result = db.transaction(|tx| async move {
        // Create contact
        let contact = create_contact_in_tx(&tx, valid_contact()).await?;
        
        // Create invoice (will fail due to invalid data)
        create_invoice_in_tx(&tx, invalid_invoice()).await?;
        
        Ok(contact)
    }).await;
    
    // Transaction should have rolled back
    assert!(result.is_err());
    
    // Verify no data was persisted
    let contacts = get_all_contacts(&db).await.unwrap();
    assert_eq!(contacts.len(), 0);
}
```

### Integration Test Best Practices

**DO**:
- ✅ Test realistic workflows (create → read → update → delete)
- ✅ Test component interactions (contact ↔ company relationships)
- ✅ Test transaction boundaries
- ✅ Test error propagation between layers
- ✅ Use isolated test database
- ✅ Clean up all test data

**DON'T**:
- ❌ Test UI rendering (that's E2E)
- ❌ Share state between tests
- ❌ Skip cleanup steps
- ❌ Test external API integrations (mock them)

---

## 🎭 End-to-End (E2E) Testing

### Scope

**E2E tests validate complete user workflows through the UI.**

### MCP-Driven E2E Tests

**Location**: `e2e-mcp/src/tests/*.test.ts`

E-fees uses a unique MCP-based approach where Claude can directly invoke Tauri commands to test complete workflows.

```typescript
// Example: e2e-mcp/src/tests/contact-workflow.test.ts

import { describe, test, expect, beforeAll, afterAll } from 'vitest'
import { MCPClient } from '../mcp-client'

describe('Contact Management Workflow', () => {
  let mcp: MCPClient
  const testTimestamp = Date.now()
  const createdIds: string[] = []
  
  beforeAll(async () => {
    // Connect to MCP server
    mcp = new MCPClient('/tmp/tauri-mcp-e2e.sock')
    await mcp.connect()
    
    // Verify app is ready
    const health = await mcp.invoke('health_check')
    expect(health.status).toBe('ok')
  })
  
  test('complete contact lifecycle', async () => {
    // 1. CREATE: User creates a new contact
    const contactData = {
      name: `DELETE ME - E2E Test Contact ${testTimestamp}`,
      email: `delete-me-${testTimestamp}@example.com`,
      phone: '+1234567890',
      notes: 'Created via E2E test'
    }
    
    const created = await mcp.invoke('create_contact', contactData)
    expect(created.id).toBeDefined()
    expect(created.name).toBe(contactData.name)
    createdIds.push(created.id)
    
    // 2. READ: User views the contact
    const retrieved = await mcp.invoke('get_contact', { id: created.id })
    expect(retrieved.id).toBe(created.id)
    expect(retrieved.email).toBe(contactData.email)
    
    // 3. UPDATE: User edits contact details
    const updateData = {
      id: created.id,
      name: `DELETE ME - Updated Contact ${testTimestamp}`,
      phone: '+0987654321'
    }
    
    const updated = await mcp.invoke('update_contact', updateData)
    expect(updated.name).toBe(updateData.name)
    expect(updated.phone).toBe(updateData.phone)
    expect(updated.email).toBe(contactData.email) // Unchanged
    
    // 4. SEARCH: User searches for the contact
    const searchResults = await mcp.invoke('search_contacts', {
      query: `DELETE ME - Updated Contact ${testTimestamp}`
    })
    expect(searchResults.length).toBeGreaterThan(0)
    expect(searchResults[0].id).toBe(created.id)
    
    // 5. DELETE: User removes the contact
    await mcp.invoke('delete_contact', { id: created.id })
    
    // 6. VERIFY: Contact is gone
    try {
      await mcp.invoke('get_contact', { id: created.id })
      fail('Contact should not exist')
    } catch (error) {
      expect(error.message).toContain('not found')
    }
  })
  
  test('contact with company relationship', async () => {
    // Create company first
    const company = await mcp.invoke('create_company', {
      name: `DELETE ME - E2E Test Company ${testTimestamp}`,
      industry: 'Technology'
    })
    createdIds.push(company.id)
    
    // Create contact linked to company
    const contact = await mcp.invoke('create_contact', {
      name: `DELETE ME - E2E Contact ${testTimestamp}`,
      email: `delete-me-${testTimestamp}-company@example.com`,
      company_id: company.id
    })
    createdIds.push(contact.id)
    
    // Verify relationship
    const companyContacts = await mcp.invoke('get_company_contacts', {
      company_id: company.id
    })
    
    expect(companyContacts).toHaveLength(1)
    expect(companyContacts[0].id).toBe(contact.id)
    
    // Verify contact shows company
    const contactWithCompany = await mcp.invoke('get_contact', {
      id: contact.id
    })
    expect(contactWithCompany.company_id).toBe(company.id)
  })
  
  test('validation errors are handled', async () => {
    // Invalid email
    try {
      await mcp.invoke('create_contact', {
        name: `DELETE ME - Invalid ${testTimestamp}`,
        email: 'not-an-email'
      })
      fail('Should have thrown validation error')
    } catch (error) {
      expect(error.message).toContain('email')
    }
    
    // Duplicate email
    const firstContact = await mcp.invoke('create_contact', {
      name: `DELETE ME - First ${testTimestamp}`,
      email: `delete-me-unique-${testTimestamp}@example.com`
    })
    createdIds.push(firstContact.id)
    
    try {
      await mcp.invoke('create_contact', {
        name: `DELETE ME - Duplicate ${testTimestamp}`,
        email: `delete-me-unique-${testTimestamp}@example.com`
      })
      fail('Should have thrown duplicate error')
    } catch (error) {
      expect(error.message).toContain('already exists')
    }
  })
  
  afterAll(async () => {
    // Cleanup: Delete all created test data
    for (const id of createdIds) {
      try {
        if (id.startsWith('contact:')) {
          await mcp.invoke('delete_contact', { id })
        } else if (id.startsWith('company:')) {
          await mcp.invoke('delete_company', { id })
        }
      } catch (error) {
        console.warn(`Failed to cleanup ${id}:`, error)
      }
    }
    
    // Verify cleanup
    const remaining = await mcp.invoke('verify_database_clean')
    expect(remaining).toHaveLength(0)
    
    await mcp.disconnect()
  })
})
```

**Run E2E tests**:
```bash
# Start Tauri app first (MCP server must be running)
npm run tauri:dev

# In another terminal:
npm run test:e2e                    # All E2E tests
npm run test:e2e contacts          # Specific suite
npm run test:e2e:verify-clean      # Verify no test data remains
```

### E2E Test Organization

```
e2e-mcp/
├── src/
│   ├── tests/
│   │   ├── contacts.test.ts       # Contact workflows
│   │   ├── companies.test.ts      # Company workflows
│   │   ├── invoices.test.ts       # Invoice workflows
│   │   ├── projects.test.ts       # Project workflows
│   │   └── integration/           # Cross-module workflows
│   │       ├── contact-invoice.test.ts
│   │       └── project-team.test.ts
│   ├── mcp-client.ts              # MCP connection wrapper
│   ├── test-helpers.ts            # Shared test utilities
│   └── fixtures/                  # Test data fixtures
│       ├── contacts.ts
│       └── companies.ts
├── vitest.config.ts
└── package.json
```

### E2E Test Best Practices

**DO**:
- ✅ Test complete user workflows (not isolated actions)
- ✅ Use realistic data (but always with "DELETE ME")
- ✅ Test happy path AND error scenarios
- ✅ Verify cleanup after each test
- ✅ Use descriptive test names
- ✅ Test cross-module interactions
- ✅ Keep tests independent

**DON'T**:
- ❌ Test internal implementation details
- ❌ Create test data without "DELETE ME" prefix
- ❌ Skip cleanup verification
- ❌ Make tests depend on each other
- ❌ Test every possible combination (focus on critical paths)
- ❌ Leave orphaned test data

---

## 🏷️ "DELETE ME" Pattern (CRITICAL)

### Purpose

The "DELETE ME" pattern ensures test data is:
1. **Identifiable** - Easy to spot in database
2. **Cleanable** - Automated cleanup via pattern matching
3. **Auditable** - Clear when test data leaks
4. **Safe** - Prevents accidental deletion of real data

### Implementation

```typescript
// ✅ CORRECT - All test data includes "DELETE ME"
const testContact = {
  name: `DELETE ME - Test Contact ${Date.now()}`,
  email: `delete-me-${Date.now()}@example.com`,
  company: `DELETE ME - Test Company ${Date.now()}`
}

// ❌ WRONG - Will be rejected in code review
const testContact = {
  name: "Test Contact",
  email: "test@example.com"
}
```

### Timestamp Usage

**Always include timestamp in test data** to ensure uniqueness:

```typescript
const timestamp = Date.now()

const testData = {
  name: `DELETE ME - Contact ${timestamp}`,
  email: `delete-me-${timestamp}@example.com`
}

// Multiple entities in same test:
const contact1 = { name: `DELETE ME - Contact ${timestamp}-1` }
const contact2 = { name: `DELETE ME - Contact ${timestamp}-2` }
```

### Automated Cleanup Tool

```typescript
// MCP Tool: cleanup_test_data
// Removes all entities containing "DELETE ME"

async function cleanupTestData(dryRun = false) {
  const results = {
    contacts: [],
    companies: [],
    invoices: [],
    projects: []
  }
  
  // Find all test entities
  const testContacts = await db.query(`
    SELECT * FROM contact WHERE name CONTAINS 'DELETE ME'
  `)
  
  if (!dryRun) {
    for (const contact of testContacts) {
      await db.delete(`contact:${contact.id}`)
      results.contacts.push(contact.id)
    }
  }
  
  // Repeat for other entity types...
  
  return results
}
```

**Usage**:
```bash
# Preview what would be deleted
npm run test:cleanup -- --dry-run

# Actually delete test data
npm run test:cleanup

# Verify database is clean
npm run test:e2e:verify-clean
```

### Pre-Commit Hook

**Enforce "DELETE ME" pattern** before commits:

```bash
# .git/hooks/pre-commit

#!/bin/bash

# Check test files for "DELETE ME" pattern
for file in $(git diff --cached --name-only | grep -E '\.(test|spec)\.(ts|js)$'); do
  if grep -q "name:" "$file" && ! grep -q "DELETE ME" "$file"; then
    echo "ERROR: Test file $file creates data without 'DELETE ME' prefix"
    exit 1
  fi
done
```

---

## 🔍 Test Data Management

### Test Fixtures

**Location**: `e2e-mcp/src/fixtures/*.ts`

```typescript
// fixtures/contacts.ts

export function createTestContact(overrides = {}) {
  const timestamp = Date.now()
  
  return {
    name: `DELETE ME - Test Contact ${timestamp}`,
    email: `delete-me-${timestamp}@example.com`,
    phone: '+1234567890',
    notes: 'Generated by test fixture',
    ...overrides
  }
}

export function createTestCompany(overrides = {}) {
  const timestamp = Date.now()
  
  return {
    name: `DELETE ME - Test Company ${timestamp}`,
    industry: 'Technology',
    ...overrides
  }
}

// Usage in tests:
const contact = await mcp.invoke('create_contact', 
  createTestContact({ phone: '+9999999999' })
)
```

### Test Database Isolation

```rust
// src-tauri/src/db/test_helpers.rs

pub async fn setup_test_db() -> Database {
    let test_db_path = format!("memory://test_{}", uuid::Uuid::new_v4());
    
    let db = Database::new(&test_db_path).await.unwrap();
    
    // Run migrations
    db.run_migrations().await.unwrap();
    
    db
}

pub async fn cleanup_test_db(db: &Database) {
    // Remove all data
    db.execute("DELETE contact;").await.unwrap();
    db.execute("DELETE company;").await.unwrap();
    db.execute("DELETE invoice;").await.unwrap();
    db.execute("DELETE project;").await.unwrap();
}
```

### Cleanup Verification

**Every test suite must verify cleanup**:

```typescript
afterAll(async () => {
  // Manual cleanup of tracked IDs
  for (const id of createdIds) {
    await deleteEntity(id)
  }
  
  // Automated verification - checks for "DELETE ME" pattern
  const remaining = await mcp.invoke('verify_database_clean')
  
  if (remaining.length > 0) {
    console.error('Test data not cleaned up:', remaining)
    throw new Error(`${remaining.length} test entities remain in database`)
  }
})
```

---

## 📈 Test Coverage

### Coverage Targets

| Component | Target | Current | Status |
|-----------|--------|---------|--------|
| Rust Backend | 80% | TBD | 🔄 |
| TypeScript Frontend | 70% | TBD | 🔄 |
| Tauri Commands | 100% | TBD | 🔄 |
| Critical Paths | 100% | TBD | 🔄 |

### Generating Coverage Reports

```bash
# Rust coverage (using tarpaulin)
cargo tarpaulin --out Html --output-dir coverage/rust

# TypeScript coverage (using vitest)
npm run test:unit:coverage

# Open coverage reports
open coverage/rust/index.html
open coverage/typescript/index.html
```

### Coverage Requirements

**Must have tests for**:
- ✅ All Tauri commands (integration tests)
- ✅ All database queries (unit + integration tests)
- ✅ All validation logic (unit tests)
- ✅ All error handling paths (unit tests)
- ✅ All critical user workflows (E2E tests)

**Can skip tests for**:
- 🟡 Simple getters/setters
- 🟡 Auto-generated code
- 🟡 Third-party library wrappers (if thin)

---

## 🐛 Testing for Bug Fixes

### Bug Fix Testing Workflow

```bash
# 1. Create failing test that reproduces bug
git checkout -b fix/issue-123-contact-email-validation

# 2. Write test
cat > src-tauri/tests/regression_issue_123.rs << 'EOF'
#[tokio::test]
async fn test_issue_123_email_validation() {
    // Test case that demonstrates the bug
    let result = validate_email("user+tag@example.com");
    assert!(result.is_ok()); // Currently fails
}
EOF

# 3. Verify test fails
cargo test test_issue_123_email_validation
# Should fail, proving bug exists

# 4. Fix the bug
# ... make code changes ...

# 5. Verify test passes
cargo test test_issue_123_email_validation
# Should pass now

# 6. Run full test suite
npm test
# Ensure no regressions

# 7. Commit with link to issue
git commit -m "fix(contacts): allow plus signs in email addresses

Fixes #123. Email validation regex was rejecting valid addresses
containing plus signs. Added regression test.

Test: test_issue_123_email_validation"
```

### Regression Test Requirements

**Every bug fix MUST include**:
- ✅ Test that reproduces the original bug
- ✅ Test that verifies the fix
- ✅ Test name references issue number
- ✅ Test comment explains the bug
- ✅ Commit message links to issue

```rust
/// Regression test for issue #123
/// Bug: Email validation rejected valid addresses with plus signs
/// Example: user+tag@example.com was incorrectly marked invalid
#[tokio::test]
async fn test_issue_123_email_validation() {
    let valid_emails = vec![
        "user+tag@example.com",
        "test+filter@domain.co.uk",
        "admin+2024@company.org"
    ];
    
    for email in valid_emails {
        assert!(validate_email(email).is_ok(), 
                "Email {} should be valid", email);
    }
}
```

---

## 🚀 CI/CD Integration

### Test Execution in CI

```yaml
# .github/workflows/test.yml

name: Test Suite

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run Rust tests
        run: cargo test --all-features
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v3
  
  typescript-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Install dependencies
        run: npm ci
      - name: Run unit tests
        run: npm run test:unit:coverage
      - name: Upload coverage
        uses: codecov/codecov-action@v3
  
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - uses: actions/setup-node@v3
      - name: Install dependencies
        run: |
          npm ci
          cargo build
      - name: Start Tauri app
        run: npm run tauri:dev &
        background: true
      - name: Wait for MCP socket
        run: |
          timeout 30 bash -c 'until [ -S /tmp/tauri-mcp-e2e.sock ]; do sleep 1; done'
      - name: Run E2E tests
        run: npm run test:e2e
      - name: Verify cleanup
        run: npm run test:e2e:verify-clean
```

### Test Failure Notifications

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: 'Test suite failed on ${{ github.ref }}'
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

---

## 📊 Test Metrics & Monitoring

### Key Metrics to Track

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Test execution time | < 10 min total | CI logs |
| Test flakiness | < 1% | Track failures over time |
| Code coverage | 80%+ | Tarpaulin + Vitest |
| Test count growth | +10 tests/week | Git history |
| Bug regression rate | 0% | Track reopened issues |

### Tracking Test Health

```bash
# Test execution time trend
git log --all --grep="CI: Tests passed" --format="%ai %s" | grep "in [0-9]"

# Test count over time
git log --all --oneline -- '**/*.test.ts' '**/*_test.rs' | wc -l

# Coverage trend
git log --all --grep="Coverage:" --format="%ai %s"
```

---

## 🔗 Related Documentation

- **MCP Architecture**: `.claude/context/mcp-architecture.md`
- **Development Workflow**: `.claude/rules/development-workflow.md`
- **Testing Specialist Sub-Agent**: `.claude/subagents/subagent-testing-specialist.md`
- **Database Patterns**: `.claude/prompts/database-patterns.md`

---

## ✅ Testing Checklist

### Before Every Commit

- [ ] All unit tests pass (`cargo test`, `npm run test:unit`)
- [ ] All integration tests pass (`cargo test --test integration_*`)
- [ ] Test data uses "DELETE ME" prefix
- [ ] New features have tests
- [ ] Bug fixes have regression tests
- [ ] Coverage meets minimums (80% Rust, 70% TS)

### Before Every PR

- [ ] All tests pass in CI
- [ ] E2E tests pass locally (`npm run test:e2e`)
- [ ] No test data remains (`npm run test:e2e:verify-clean`)
- [ ] Code coverage report reviewed
- [ ] Test names are descriptive
- [ ] Test documentation updated

### Before Every Release

- [ ] Full test suite passes
- [ ] Manual smoke testing complete
- [ ] E2E critical paths verified
- [ ] Performance tests pass
- [ ] Database migrations tested
- [ ] Rollback procedure tested

---

**Last Updated**: October 26, 2025  
**Version**: 1.0  
**Maintained By**: Martin & Claude Code

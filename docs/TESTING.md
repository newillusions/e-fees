# End-to-End Testing Procedures

## Overview

This document provides comprehensive testing procedures for the Fee Proposal Management System. These tests ensure all components work together correctly and verify the complete user workflows from frontend to database.

## Test Environment Setup

### Prerequisites

Before running tests, ensure the following environment is configured:

```bash
# 1. SurrealDB Test Instance
surreal start --log debug --user root --pass root --bind 0.0.0.0:8001 memory

# 2. Test Environment Variables
export SURREALDB_URL=ws://localhost:8001
export SURREALDB_NS=test
export SURREALDB_DB=projects
export SURREALDB_USER=root
export SURREALDB_PASS=root
export PROJECT_FOLDER_PATH=/tmp/test-projects

# 3. Application Development Server
npm run tauri:dev
```

### Test Data Setup

Create test data structure:

```sql
-- Countries reference data
INSERT INTO country [
  { name: "United Arab Emirates", dial_code: 971, code: "AE" },
  { name: "Saudi Arabia", dial_code: 966, code: "SA" },
  { name: "Qatar", dial_code: 974, code: "QA" }
];

-- Test companies
INSERT INTO company [
  {
    name: "Test Company One",
    name_short: "Test Co 1",
    abbreviation: "TC1",
    city: "Dubai",
    country: "United Arab Emirates"
  },
  {
    name: "Test Company Two", 
    name_short: "Test Co 2",
    abbreviation: "TC2",
    city: "Riyadh",
    country: "Saudi Arabia"
  }
];
```

## Core Functionality Tests

### 1. Database Connection Tests

#### Test 1.1: Initial Connection
**Objective**: Verify application can connect to SurrealDB on startup

**Steps**:
1. Start application with test environment variables
2. Observe ConnectionStatus component in sidebar
3. Check application console for connection logs

**Expected Results**:
- ConnectionStatus shows green indicator
- Console shows "Database connection established successfully"
- No error messages in UI

**Failure Scenarios**:
- Red indicator → Check database server status
- Authentication errors → Verify credentials
- Network errors → Check firewall/network configuration

#### Test 1.2: Connection Resilience
**Objective**: Test application behavior during database disconnection

**Steps**:
1. Establish successful connection
2. Stop SurrealDB server
3. Attempt database operations (e.g., load projects)
4. Restart SurrealDB server
5. Wait for automatic reconnection

**Expected Results**:
- ConnectionStatus changes to red during disconnection
- Error messages are user-friendly
- Application reconnects automatically when server returns
- Data operations resume normally after reconnection

### 2. Project Management Tests

#### Test 2.1: Project Number Generation
**Objective**: Verify automatic project number generation system

**Steps**:
1. Navigate to Projects page
2. Click "New Project" button
3. Enter project details:
   - Name: "Test Museum Project"
   - Short Name: "TMP"
   - Area: "Business Bay"
   - City: "Dubai"
   - Country: Start typing "United" and select "United Arab Emirates"
4. Observe project number field updates automatically
5. Save project

**Expected Results**:
- Project number follows YY-CCCNN format (e.g., "25-97101")
- Country selection triggers number generation
- Number is unique and sequential
- Project saves successfully with generated number

**Validation Points**:
- Year component matches current year (last 2 digits)
- Country code is 971 for UAE
- Sequence starts at 01 for first project of year/country
- Subsequent projects increment sequence (02, 03, etc.)

#### Test 2.2: Project Creation with Template
**Objective**: Test complete project creation workflow with file system integration

**Steps**:
1. Ensure template folder exists at PROJECT_FOLDER_PATH
2. Create new project through modal
3. Fill all required fields
4. Submit project creation
5. Verify database record creation
6. Check file system for project folder
7. Verify template files were copied and renamed

**Expected Results**:
- Project appears in projects list immediately
- File system folder created with project number and name
- Template files copied with updated naming
- Database record contains correct folder path
- No file permission errors

#### Test 2.3: Project Search and Filtering
**Objective**: Verify search functionality across project fields

**Test Data**: Create projects with various names, locations, and numbers

**Search Tests**:
1. **Name Search**: Search "museum" → should return projects with "museum" in name
2. **Location Search**: Search "dubai" → should return Dubai-based projects
3. **Number Search**: Search "25-971" → should return UAE projects from 2025
4. **Partial Search**: Search "test" → should return partial matches
5. **Case Insensitive**: Search "DUBAI" → should return same results as "dubai"

**Expected Results**:
- Search is real-time (updates as user types)
- Results include all relevant matches
- Search works across all searchable fields
- No database errors during search operations

### 3. Company Management Tests

#### Test 3.1: Company CRUD Operations
**Objective**: Test complete company lifecycle

**Create Company**:
1. Navigate to Companies page
2. Click "New Company" button
3. Fill company form:
   - Name: "Test Engineering LLC"
   - Short Name: "Test Eng"
   - Abbreviation: "TEL"
   - City: "Dubai"
   - Country: "United Arab Emirates"
   - Registration No: "REG-123456"
   - Tax No: "TRN-987654321"
4. Submit form

**Update Company**:
1. Find created company in list
2. Click edit button
3. Modify fields (e.g., change city to "Abu Dhabi")
4. Save changes

**Delete Company**:
1. Select company without associated contacts/projects
2. Click delete button
3. Confirm deletion

**Expected Results**:
- Company appears in list after creation
- Edit modal pre-populates with existing data
- Changes persist after update
- Company removed from list after deletion
- Referential integrity prevents deletion of companies with dependencies

#### Test 3.2: Company Validation
**Objective**: Test form validation and error handling

**Invalid Data Tests**:
1. **Required Fields**: Try to save company with empty name
2. **Duplicate Abbreviation**: Create company with existing abbreviation
3. **Invalid Characters**: Use special characters in names
4. **Long Text**: Exceed maximum field lengths

**Expected Results**:
- Clear error messages for invalid data
- Form prevents submission until errors resolved
- Existing data preserved during validation errors

### 4. Contact Management Tests

#### Test 4.1: Contact-Company Relationship
**Objective**: Verify contact-company associations

**Steps**:
1. Ensure at least one company exists
2. Navigate to Contacts page
3. Create new contact:
   - First Name: "Ahmed"
   - Last Name: "Al-Mansoori"
   - Email: "ahmed@testcompany.ae"
   - Phone: "+971-50-123-4567"
   - Position: "Project Manager"
   - Company: Select existing company
4. Verify contact appears with company name

**Expected Results**:
- Contact list shows company name alongside contact details
- Company dropdown populated with existing companies
- Full name auto-computed from first and last names
- Phone number validation accepts international format

#### Test 4.2: Contact Search and Filtering
**Objective**: Test contact search across multiple fields

**Search Scenarios**:
1. Search by first name
2. Search by last name
3. Search by email address
4. Search by company name
5. Search by position

**Expected Results**:
- Search works across all contact fields
- Company information included in search results
- Real-time filtering as user types

### 5. RFP Management Tests

#### Test 5.1: RFP Creation Workflow
**Objective**: Test complete RFP creation with relationships

**Prerequisites**: 
- At least one project exists
- At least one company exists
- At least one contact exists

**Steps**:
1. Navigate to Proposals page
2. Click "New RFP" button
3. Fill RFP form:
   - Name: "Dubai Museum Design Proposal"
   - Number: "RFP-25-001"
   - Project: Select existing project
   - Company: Select existing company
   - Contact: Select existing contact (should filter by selected company)
   - Issue Date: Enter date in YYMMDD format
   - Activity: "Design and Consultancy"
   - Staff details: Auto-populate from environment

**Expected Results**:
- Contact dropdown filters based on selected company
- Staff information pre-populated from configuration
- RFP appears in proposals list
- All relationships properly linked

#### Test 5.2: RFP Status Workflow
**Objective**: Test RFP status transitions

**Status Progression**:
1. Create RFP with "Draft" status
2. Update to "Prepared" status
3. Change to "Sent" status
4. Update to "Under Review"
5. Final status: "Awarded" or "Lost"

**Expected Results**:
- Status changes persist in database
- Status history maintained
- UI updates reflect current status
- Business rules enforced for status transitions

### 6. Integration Tests

#### Test 6.1: Cross-Entity Relationships
**Objective**: Verify data integrity across related entities

**Test Scenario**:
1. Create Company A
2. Create Contact 1 associated with Company A
3. Create Project 1 in UAE
4. Create RFP 1 linking Project 1, Company A, and Contact 1
5. Attempt to delete Company A (should fail)
6. Delete RFP 1
7. Delete Contact 1
8. Delete Company A (should succeed)

**Expected Results**:
- Referential integrity prevents orphaned records
- Cascade deletions work correctly
- Error messages explain relationship constraints
- Data consistency maintained throughout operations

#### Test 6.2: File System Integration
**Objective**: Test project folder management

**Steps**:
1. Create project through NewProjectModal
2. Verify folder created in file system
3. Click "Open Folder" button in projects list
4. Verify file explorer opens correct location
5. Create additional projects with similar names
6. Verify unique folder names generated

**Expected Results**:
- Folders created with correct naming convention
- File explorer integration works on current platform
- No conflicts with existing folders
- Folder paths stored correctly in database

### 7. UI/UX Tests

#### Test 7.1: Keyboard Navigation
**Objective**: Verify accessibility and keyboard shortcuts

**Keyboard Shortcuts**:
- `Cmd/Ctrl + 1`: Dashboard
- `Cmd/Ctrl + 2`: Projects
- `Cmd/Ctrl + 3`: Proposals
- `Cmd/Ctrl + 4`: Companies
- `Cmd/Ctrl + 5`: Contacts
- `Cmd/Ctrl + W`: Position window for 4K

**Expected Results**:
- All shortcuts work correctly
- Navigation updates URL and page content
- Window positioning optimizes for display
- Focus management follows accessibility guidelines

#### Test 7.2: Responsive Design
**Objective**: Test UI adaptation to different screen sizes

**Test Scenarios**:
1. **Standard Display** (1920x1080): Verify default layout
2. **4K Display** (3840x2160): Test high-DPI scaling
3. **Minimum Window Size** (1024x600): Verify layout constraints
4. **Window Resizing**: Test dynamic layout adjustments

**Expected Results**:
- UI scales appropriately for each resolution
- Text remains readable at all sizes
- Interactive elements maintain proper spacing
- No content overflow or clipping

#### Test 7.3: Real-time Updates
**Objective**: Verify reactive UI updates

**Test Steps**:
1. Open application in two windows (if possible)
2. Create/modify data in one window
3. Observe updates in second window
4. Test with different entity types (projects, companies, etc.)

**Expected Results**:
- UI updates immediately after data changes
- Changes reflected across all components
- No stale data displayed
- Smooth animations during updates

### 8. Performance Tests

#### Test 8.1: Large Dataset Handling
**Objective**: Test application performance with substantial data

**Setup**:
```sql
-- Create large dataset
FOR $i IN 1..1000 {
    CREATE projects SET 
        name = "Project " + <string>$i,
        name_short = "P" + <string>$i,
        status = "Draft",
        area = "Test Area",
        city = "Dubai", 
        country = "United Arab Emirates",
        folder = "/test/project" + <string>$i,
        number = { year: 25, country: 971, seq: $i, id: "25-971" + <string>$i };
};
```

**Performance Metrics**:
1. **Initial Load**: Time to display projects list
2. **Search Response**: Time for search results to appear
3. **Scroll Performance**: Smooth scrolling through large lists
4. **Memory Usage**: Monitor application memory consumption

**Expected Results**:
- Initial load < 3 seconds
- Search results < 1 second
- Smooth scrolling performance
- Memory usage remains stable
- No UI freezing or lag

#### Test 8.2: Database Query Optimization
**Objective**: Verify efficient database operations

**Query Analysis**:
1. Monitor SurrealDB query logs during operations
2. Measure query execution times
3. Verify appropriate use of indexes
4. Check for N+1 query problems

**Expected Results**:
- Queries use appropriate filtering
- No unnecessary data fetching
- Query times remain under 500ms
- Efficient use of database resources

### 9. Error Handling Tests

#### Test 9.1: Network Interruption
**Objective**: Test application behavior during network issues

**Simulation**:
1. Block network access to database server
2. Attempt various operations
3. Restore network access
4. Verify recovery behavior

**Expected Results**:
- User-friendly error messages
- Application doesn't crash
- Automatic retry mechanisms
- Data integrity maintained

#### Test 9.2: Invalid Data Handling
**Objective**: Test robustness against malformed data

**Test Cases**:
1. **SQL Injection**: Attempt malicious input in search fields
2. **XSS Attacks**: Try script injection in text fields
3. **Invalid Characters**: Unicode and special characters
4. **Oversized Input**: Extremely long text in form fields

**Expected Results**:
- All input properly sanitized
- No security vulnerabilities
- Graceful handling of edge cases
- Appropriate error messages

### 10. Security Tests

#### Test 10.1: Authentication and Authorization
**Objective**: Verify access control mechanisms

**Test Scenarios**:
1. **Invalid Credentials**: Wrong username/password
2. **Expired Sessions**: Long-running application sessions
3. **Permission Levels**: Different user access levels
4. **Database Security**: Direct database access attempts

**Expected Results**:
- Failed authentication handled gracefully
- Session management works correctly
- Users can only access permitted resources
- Database access properly controlled

#### Test 10.2: Data Validation
**Objective**: Ensure all data inputs are validated

**Validation Tests**:
1. **Email Formats**: Valid vs invalid email addresses
2. **Phone Numbers**: International format validation
3. **Date Formats**: YYMMDD format enforcement
4. **Project Numbers**: YY-CCCNN format validation

**Expected Results**:
- Invalid data rejected with clear messages
- Valid data accepted and processed correctly
- No data corruption in database
- Consistent validation rules across application

## Automated Testing Framework

### Test Automation Setup

For automated testing, consider implementing:

```typescript
// Example test structure using Playwright
import { test, expect } from '@playwright/test';

test.describe('Project Management', () => {
  test('should create project with auto-generated number', async ({ page }) => {
    await page.goto('/projects');
    await page.click('[data-testid="new-project-button"]');
    
    await page.fill('[data-testid="project-name"]', 'Test Project');
    await page.fill('[data-testid="project-area"]', 'Business Bay');
    await page.selectOption('[data-testid="country-select"]', 'United Arab Emirates');
    
    // Verify project number is generated
    const projectNumber = await page.inputValue('[data-testid="project-number"]');
    expect(projectNumber).toMatch(/^\d{2}-\d{3}\d{2}$/);
    
    await page.click('[data-testid="save-project"]');
    
    // Verify project appears in list
    await expect(page.locator('[data-testid="projects-list"]')).toContainText('Test Project');
  });
});
```

### Continuous Integration Tests

Add to CI/CD pipeline:

```yaml
# .github/workflows/test.yml
name: E2E Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      surrealdb:
        image: surrealdb/surrealdb:latest
        ports:
          - 8000:8000
        options: --rm
        
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          
      - name: Install dependencies
        run: npm ci
        
      - name: Run E2E tests
        run: npm run test:e2e
        env:
          SURREALDB_URL: ws://localhost:8000
          SURREALDB_PASS: test
```

## Test Reporting

### Test Results Documentation

For each test run, document:

1. **Test Environment**: OS, browser, database version
2. **Test Duration**: Total time for complete test suite
3. **Pass/Fail Rate**: Number of tests passed vs failed
4. **Performance Metrics**: Load times, query performance
5. **Issues Found**: Bugs discovered during testing
6. **Recommendations**: Improvements for next iteration

### Test Coverage Metrics

Track coverage across:

- **Frontend Components**: UI component testing
- **API Endpoints**: Backend command testing  
- **Database Operations**: CRUD operation testing
- **Business Logic**: Workflow and validation testing
- **Integration Points**: Cross-system functionality

## Troubleshooting Test Issues

### Common Test Failures

#### Database Connection Issues
```
Error: "No database connection"
Solution: 
1. Verify SurrealDB is running on correct port
2. Check environment variables
3. Ensure test database is accessible
```

#### UI Element Not Found
```
Error: "Element with selector not found"
Solution:
1. Check if page has fully loaded
2. Verify element exists in current application state
3. Add explicit waits for dynamic content
```

#### Data Inconsistency
```
Error: "Expected data not found"
Solution:
1. Ensure test data setup completed successfully
2. Check for previous test data interference
3. Verify database is properly reset between tests
```

### Best Practices

1. **Test Isolation**: Each test should be independent
2. **Clean State**: Reset database state between tests
3. **Realistic Data**: Use representative test data
4. **Error Scenarios**: Test both success and failure paths
5. **Performance Monitoring**: Track test execution times
6. **Documentation**: Keep test procedures updated

## Conclusion

These end-to-end testing procedures provide comprehensive coverage of the Fee Proposal Management System. Regular execution of these tests ensures:

- **Functionality**: All features work as designed
- **Reliability**: System handles errors gracefully
- **Performance**: Application meets speed requirements
- **Security**: Data and access properly protected
- **Usability**: User experience meets expectations

For questions about testing procedures or to report issues, refer to the main project documentation or contact the development team.

---

**Last Updated**: June 16, 2025  
**Version**: 2.0.0  
**Test Suite Status**: Ready for Implementation
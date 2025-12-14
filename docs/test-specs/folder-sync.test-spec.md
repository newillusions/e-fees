# Folder Sync Feature - Test Specification

## Overview
This document defines test cases for the Folder Sync feature, which detects and resolves inconsistencies between the database project statuses and actual file system folder locations.

## Test Environment

### Status-to-Folder Mapping
| Status | Folder |
|--------|--------|
| rfp | 01 RFPs |
| active | 11 Current |
| completed | 99 Completed |
| cancelled | 00 Inactive |

### Test Data Setup
For each test, create a temporary directory structure:
```
/tmp/test-projects/
├── 00 Inactive/
├── 01 RFPs/
├── 11 Current/
└── 99 Completed/
```

---

## Unit Tests - Rust Backend

### TC-FS-001: Scan Returns Empty When All Folders Match
**Description:** When all project folders are in their expected locations based on DB status, scan should return no inconsistencies.

**Setup:**
- DB has project `25-97101 Test Project A` with status `active`
- Folder exists at `/base/11 Current/25-97101 Test Project A`

**Input:**
```rust
scan_folder_sync("/base".to_string(), state)
```

**Expected Output:**
```json
{
  "inconsistencies": [],
  "total_projects": 1,
  "total_folders": 1,
  "errors": []
}
```

---

### TC-FS-002: Detect Wrong Location
**Description:** When a project folder exists but in a different status folder than the DB expects.

**Setup:**
- DB has project `25-97102 Test Project B` with status `active` (expects `11 Current`)
- Folder actually exists at `/base/01 RFPs/25-97102 Test Project B`

**Input:**
```rust
scan_folder_sync("/base".to_string(), state)
```

**Expected Output:**
```json
{
  "inconsistencies": [{
    "project_id": "projects:xxx",
    "project_number": "25-97102",
    "project_name": "Test Project B",
    "folder_name": "25-97102 Test Project B",
    "type": "wrong_location",
    "db_status": "active",
    "expected_path": "/base/11 Current/25-97102 Test Project B",
    "actual_path": "/base/01 RFPs/25-97102 Test Project B",
    "actual_status": "rfp"
  }]
}
```

---

### TC-FS-003: Detect Missing Folder
**Description:** When a project exists in DB but no folder is found anywhere on disk.

**Setup:**
- DB has project `25-97103 Test Project C` with status `rfp`
- No folder exists on disk matching this project

**Input:**
```rust
scan_folder_sync("/base".to_string(), state)
```

**Expected Output:**
```json
{
  "inconsistencies": [{
    "project_id": "projects:xxx",
    "project_number": "25-97103",
    "project_name": "Test Project C",
    "folder_name": "25-97103 Test Project C",
    "type": "missing",
    "db_status": "rfp",
    "expected_path": "/base/01 RFPs/25-97103 Test Project C",
    "actual_path": null,
    "actual_status": null
  }]
}
```

---

### TC-FS-004: Detect Orphan Folder
**Description:** When a folder exists on disk but has no matching project in the database.

**Setup:**
- DB has no project with number `25-97104`
- Folder exists at `/base/11 Current/25-97104 Unknown Project`

**Input:**
```rust
scan_folder_sync("/base".to_string(), state)
```

**Expected Output:**
```json
{
  "inconsistencies": [{
    "project_id": "",
    "project_number": "25-97104",
    "project_name": "Unknown Project",
    "folder_name": "25-97104 Unknown Project",
    "type": "orphan",
    "db_status": null,
    "expected_path": null,
    "actual_path": "/base/11 Current/25-97104 Unknown Project",
    "actual_status": "active"
  }]
}
```

---

### TC-FS-005: Detect Duplicate Folders
**Description:** When the same project folder exists in multiple status folders.

**Setup:**
- DB has project `25-97105 Test Project D` with status `active`
- Folder exists at `/base/01 RFPs/25-97105 Test Project D`
- Folder ALSO exists at `/base/11 Current/25-97105 Test Project D`

**Input:**
```rust
scan_folder_sync("/base".to_string(), state)
```

**Expected Output:**
```json
{
  "inconsistencies": [{
    "project_id": "projects:xxx",
    "project_number": "25-97105",
    "project_name": "Test Project D",
    "folder_name": "25-97105 Test Project D",
    "type": "duplicate",
    "db_status": "active",
    "expected_path": "/base/11 Current/25-97105 Test Project D",
    "actual_path": "/base/11 Current/25-97105 Test Project D",
    "actual_status": "active",
    "duplicate_paths": [
      "/base/01 RFPs/25-97105 Test Project D",
      "/base/11 Current/25-97105 Test Project D"
    ]
  }]
}
```

---

### TC-FS-006: Resolution - Update DB Status
**Description:** User resolves a wrong_location by updating the DB status to match actual folder location.

**Setup:**
- Project `25-97102` has status `active` in DB
- Folder is in `01 RFPs` (status should be `rfp`)

**Input:**
```rust
resolve_folder_inconsistency(ResolutionRequest {
    action: "update_db",
    project_id: "projects:xxx",
    new_status: "rfp"
}, state)
```

**Expected Behavior:**
1. DB project status updated to `rfp`
2. Returns success message
3. Subsequent scan shows no inconsistency

**Expected Output:**
```json
{
  "success": true,
  "message": "Project status updated to rfp"
}
```

---

### TC-FS-007: Resolution - Move Folder
**Description:** User resolves a wrong_location by moving the folder to the expected location.

**Setup:**
- Project `25-97102` has status `active` in DB
- Folder currently at `/base/01 RFPs/25-97102 Test Project B`
- Expected at `/base/11 Current/25-97102 Test Project B`

**Input:**
```rust
resolve_folder_inconsistency(ResolutionRequest {
    action: "move_folder",
    from_path: "/base/01 RFPs/25-97102 Test Project B",
    to_path: "/base/11 Current/25-97102 Test Project B"
}, state)
```

**Expected Behavior:**
1. Folder moved from source to destination
2. Source folder no longer exists
3. Destination folder exists with all contents
4. Returns success message

**Expected Output:**
```json
{
  "success": true,
  "message": "Folder moved successfully"
}
```

---

### TC-FS-008: Resolution - Create Missing Folder
**Description:** User creates a missing project folder using the template.

**Setup:**
- Project `25-97103 Test Project C` exists in DB with status `rfp`
- No folder exists on disk

**Input:**
```rust
resolve_folder_inconsistency(ResolutionRequest {
    action: "create_folder",
    project_id: "projects:xxx"
}, state)
```

**Expected Behavior:**
1. Template folder copied to expected location
2. New folder exists at `/base/01 RFPs/25-97103 Test Project C`
3. Returns success message

---

## Edge Case Tests

### TC-FS-E01: Handle Spaces in Folder Names
**Setup:** Folder name `25-97110 Beach Resort Development`

**Expected:** Folder is found and processed correctly without escaping issues.

---

### TC-FS-E02: Handle Unicode Characters
**Setup:** Folder name `25-97111 Dubai مشروع`

**Expected:** Folder is found and processed correctly with UTF-8 handling.

---

### TC-FS-E03: Handle Permission Denied
**Setup:** One status folder has no read permissions.

**Expected:**
- Scan continues for other folders
- Error is logged in `errors` array
- Partial results returned

---

### TC-FS-E04: Handle Non-Existent Base Path
**Input:** `scan_folder_sync("/nonexistent/path".to_string(), state)`

**Expected:** Returns error: "Base path does not exist: /nonexistent/path"

---

### TC-FS-E05: Handle Case-Insensitive Matching (macOS)
**Setup:**
- DB has project `25-97112 Test Project`
- Folder exists as `25-97112 test project` (different case)

**Expected:**
- macOS: Matched as same folder
- Linux: Treated as missing (case-sensitive)

---

### TC-FS-E06: Folder Without Project Number Prefix
**Setup:** Folder `Random Folder Name` exists in `11 Current`

**Expected:** Ignored - not reported as orphan (doesn't match project number pattern).

---

### TC-FS-E07: Multiple Projects Match Pattern Partially
**Setup:**
- Projects: `25-97113 Project A`, `25-97113-01 Project A Ext`
- Folders match expected locations

**Expected:** Both matched correctly by exact project number.

---

## Integration Tests

### TC-FS-I01: Full Scan and Resolution Workflow
**Steps:**
1. Create test directory structure with known inconsistencies
2. Run scan_folder_sync
3. Verify all inconsistencies detected
4. Resolve each inconsistency
5. Run scan again
6. Verify no inconsistencies remain

---

### TC-FS-I02: Frontend Modal Integration
**Steps:**
1. Open Settings modal
2. Click "Folder Sync" button
3. Click "Scan Now"
4. Verify results table populated
5. Click resolution action on each row
6. Verify success feedback
7. Re-scan to confirm resolution

---

## Performance Tests

### TC-FS-P01: Large Number of Projects
**Setup:** 500 projects in DB, 500 folders on disk

**Expected:** Scan completes in < 5 seconds

---

### TC-FS-P02: Deep Folder Structure
**Setup:** Project folders contain nested subdirectories (10+ levels)

**Expected:** Only top-level project folders scanned, not recursively.

---

## Acceptance Criteria

1. All unit tests pass
2. All edge case tests pass
3. Integration workflow completes successfully
4. No data loss occurs during folder moves
5. UI provides clear feedback for all operations
6. Errors are gracefully handled and reported

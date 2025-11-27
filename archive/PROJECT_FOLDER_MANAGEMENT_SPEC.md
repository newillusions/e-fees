# Project Folder Management System Specification

## Overview
This document outlines the implementation of a project folder management system that maintains synchronization between the file system project folders and the database records for projects and fee proposals.

## Background Research Summary

### Folder Structure Analysis
Based on examination of `/Volumes/base/emittiv/emittiv/01 Projects/`, the organization follows this pattern:

```
/01 Projects/
├── 00 Inactive/          # Completed/inactive/cancelled projects
├── 01 RFPs/              # Active RFP submissions (proposal phase)
├── 11 Current/           # Active ongoing projects (awarded/in-progress)
└── 99 Completed/         # Successfully completed projects
```

### Individual Project Structure
Each project folder follows standardized format: `YY-CCCNN Project Name/`
- **YY**: Year (25 = 2025)
- **CCC**: Country code (971 = UAE, 966 = Saudi)
- **NN**: Sequential number per country/year

Standard subfolders:
- `01 Client Info/` - Client communications and pre-award docs
- `02 Proposal/` - Fee proposals, pricing, InDesign files
- `03 Contract/` - Purchase orders, contracts, invoices (when awarded)
- `04 Deliverables/` - Project deliverables, drawings, specs
- `05 Submittals/` - Client submissions

### Status Relationship Rules
**Project → Fee Proposals (Downstream Impact)**
- Project cancelled → All related fee proposals should be cancelled
- Project delayed → Related fee proposals may need status updates
- Project completed → Fee proposals get final status updates

**Fee Proposals ≠ Project Status (Independent)**
- Multiple proposals per project can have different statuses
- Losing one proposal doesn't affect project or other proposals
- Proposals have independent lifecycles from projects

## Implementation Tasks

### Phase 1: Core Folder Management Functions

#### Task 1: Project Folder Movement Functions
**Files to create/modify:**
- `src-tauri/src/commands/folder_management.rs` (new)
- `src-tauri/src/lib.rs` (add command exports)
- `src/lib/api/folderManagement.ts` (new)

**Functions needed:**
```rust
// Move project from RFP to Current/Archive
move_project_from_rfp(project_number: String, destination: String) -> Result<String, String>

// Move project from Current to Archive
move_project_to_archive(project_number: String) -> Result<String, String>

// Validate project folder exists and get current location
get_project_folder_location(project_number: String) -> Result<String, String>

// List all projects in a specific folder
list_projects_in_folder(folder_path: String) -> Result<Vec<String>, String>
```

#### Task 2: Status Synchronization System
**Files to create/modify:**
- `src/lib/components/StatusChangeModal.svelte` (new)
- `src/lib/components/ProjectStatusManager.svelte` (new)
- Enhanced project/proposal modals with status change triggers

**Features needed:**
- Confirmation dialogs showing downstream impact
- Option to update related proposal statuses
- Warning when status changes affect multiple records
- Audit trail of status changes

#### Task 3: Background Reconciliation System
**Files to create/modify:**
- `src-tauri/src/services/reconciliation.rs` (new)
- `src-tauri/src/scheduler.rs` (new)
- `src/lib/components/ReconciliationReport.svelte` (new)

**Reconciliation features:**
```rust
pub struct ReconciliationReport {
    pub scan_date: DateTime<Utc>,
    pub total_projects: i32,
    pub total_matches: i32,
    pub discrepancies: ReconciliationDiscrepancies,
}

pub struct ReconciliationDiscrepancies {
    pub missing_in_db: Vec<String>,           // Folders without DB records
    pub missing_on_filesystem: Vec<String>,   // DB records without folders
    pub status_mismatches: Vec<StatusMismatch>, // Location vs status conflicts
    pub orphaned_proposals: Vec<String>,      // Proposals without project folders
}

pub struct StatusMismatch {
    pub project_number: String,
    pub folder_location: String,
    pub db_status: String,
    pub suggested_action: String,
}
```

## Workflow Design

### Project Status → Folder Location Mapping
- **RFP/Proposal** → `01 RFPs/`
- **Active/Current/Awarded** → `11 Current/`
- **Completed** → `99 Completed/`
- **Cancelled/Inactive/Lost** → `00 Inactive/`

### User Interaction Flow
1. **Manual Status Change**: User changes project status in UI
2. **Impact Analysis**: System identifies related proposals that may be affected
3. **Confirmation Dialog**: Show user what will change, get approval
4. **Folder Move**: Move project folder to appropriate directory
5. **Database Update**: Update project and optionally proposal statuses
6. **Audit Trail**: Log all changes for tracking

### Background Reconciliation Flow
1. **Scheduled Scan**: Run every 24 hours or on manual trigger
2. **Filesystem Scan**: Find all project folders and their locations
3. **Database Query**: Get all projects and their statuses
4. **Comparison**: Identify discrepancies between filesystem and database
5. **Report Generation**: Create detailed reconciliation report
6. **Notification**: Alert users to discrepancies requiring attention
7. **Auto-fix Options**: Provide safe correction options with approval

## Technical Considerations

### Error Handling
- **File Permission Issues**: Handle cases where folders cannot be moved
- **Network Drive Issues**: Robust handling of network storage
- **Concurrent Access**: Prevent conflicts when multiple users access same project
- **Partial Failures**: Handle cases where folder moves succeed but DB updates fail

### Security & Safety
- **Backup Verification**: Ensure folders are properly moved, not copied/deleted
- **Atomic Operations**: Database and filesystem changes should be coordinated
- **Permission Checks**: Verify user has rights to move folders
- **Audit Logging**: Track all folder movements and status changes

### Performance
- **Batch Operations**: Handle multiple folder moves efficiently
- **Background Processing**: Don't block UI during folder operations
- **Caching**: Cache folder structure to avoid repeated filesystem scans
- **Progress Indicators**: Show progress for long-running operations

### Configuration
Environment variables needed:
```env
PROJECT_BASE_PATH=/Volumes/base/emittiv/emittiv/01 Projects
RECONCILIATION_SCHEDULE_HOURS=24
AUTO_FIX_ENABLED=false
AUDIT_LOG_RETENTION_DAYS=365
```

## Implementation Priority

### Phase 1 (High Priority)
1. Basic folder movement functions (RFP → Current/Archive)
2. Simple status change confirmations
3. Manual reconciliation trigger

### Phase 2 (Medium Priority)
1. Advanced status synchronization with impact analysis
2. Background reconciliation scheduling
3. Comprehensive reconciliation reporting

### Phase 3 (Future Enhancements)
1. Auto-fix capabilities for common discrepancies
2. Advanced audit trail and reporting
3. Integration with external project management tools

## Testing Strategy

### Test Cases Needed
1. **Folder Movement Tests**: Verify folders move correctly between directories
2. **Status Sync Tests**: Ensure database updates match folder movements
3. **Reconciliation Tests**: Test discrepancy detection and reporting
4. **Error Handling Tests**: Test failure scenarios and recovery
5. **Concurrent Access Tests**: Test multiple users accessing same project

### Test Data
- Create test project folders in development environment
- Use isolated database instance for testing
- Mock filesystem operations for unit tests

## Success Criteria
- ✅ Users can move projects between lifecycle stages via UI
- ✅ Database status automatically stays synchronized with folder location
- ✅ Background reconciliation detects and reports discrepancies
- ✅ System handles manual folder moves gracefully
- ✅ All operations are logged for audit purposes
- ✅ No data loss occurs during folder operations

---

**Status**: Specification Complete - Ready for Implementation  
**Next Step**: Begin Phase 1 implementation with basic folder movement functions
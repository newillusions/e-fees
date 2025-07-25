//! # Refactored Command Examples
//! 
//! This file demonstrates how the existing commands can be refactored using
//! the new utility functions and macros to eliminate code duplication.
//! 
//! ## Before vs After Comparison
//! 
//! ### BEFORE: Traditional implementation (18 lines per command)
//! ```rust
//! #[tauri::command]
//! pub async fn get_companies(state: State<'_, AppState>) -> Result<Vec<Company>, String> {
//!     info!("Fetching companies from database");
//!     
//!     let manager_clone = {
//!         let manager = state.lock().map_err(|e| e.to_string())?;
//!         manager.clone()
//!     }; // Lock is automatically dropped here when manager goes out of scope
//!     
//!     match manager_clone.get_companies().await {
//!         Ok(companies) => {
//!             info!("Successfully fetched {} companies", companies.len());
//!             Ok(companies)
//!         }
//!         Err(e) => {
//!             error!("Failed to fetch companies: {}", e);
//!             Err(format!("Failed to fetch companies: {}", e))
//!         }
//!     }
//! }
//! ```
//! 
//! ### AFTER: Refactored implementation (1 line!)
//! ```rust
//! crud_command!(get_companies, Vec<Company>, get_companies, "fetch", "companies");
//! ```

use crate::db::entities::*;
use crate::AppState;
use crate::crud_command;
use tauri::State;

// =============================================================================
// EXAMPLE: Refactored Company Commands using Macros
// =============================================================================

// Get all companies - replaces 18 lines with 1 line
crud_command!(
    get_companies_refactored, 
    Vec<Company>, 
    get_companies, 
    "fetch", 
    "companies"
);

// Create company - replaces 18 lines with 1 line  
crud_command!(
    create_company_refactored,
    Company,
    CompanyCreate,
    create_company,
    "create",
    "company",
    data: company
);

// Update company - replaces 18 lines with 1 line
crud_command!(
    update_company_refactored,
    Company,
    CompanyUpdate,
    update_company,
    "update", 
    "company",
    id: String,
    data: company
);

// Delete company - replaces 18 lines with 1 line
crud_command!(
    delete_company_refactored,
    Company,
    delete_company,
    "delete",
    "company",
    id: String
);

// =============================================================================
// EXAMPLE: Refactored Contact Commands
// =============================================================================

crud_command!(
    get_contacts_refactored,
    Vec<Contact>,
    get_contacts,
    "fetch",
    "contacts"
);

crud_command!(
    create_contact_refactored,
    Contact,
    ContactCreate,
    create_contact,
    "create",
    "contact",
    data: contact
);

crud_command!(
    update_contact_refactored,
    Contact,
    ContactUpdate,
    update_contact,
    "update",
    "contact", 
    id: String,
    data: contact
);

crud_command!(
    delete_contact_refactored,
    Contact,
    delete_contact,
    "delete",
    "contact",
    id: String
);

// =============================================================================
// EXAMPLE: Manual Refactoring using execute_with_manager utility
// =============================================================================

/// Example of manual refactoring for commands that need custom logic
/// This still reduces code duplication while allowing for customization
#[tauri::command]
pub async fn get_company_by_abbreviation_refactored(
    abbreviation: String,
    state: State<'_, AppState>,
) -> Result<Option<Company>, String> {
    execute_with_manager(
        &state,
        |manager| Box::pin(async move {
            manager.get_company_by_abbreviation(&abbreviation).await
        }),
        "fetch",
        &format!("company with abbreviation '{}'", abbreviation),
    ).await
}

/// Example showing how complex commands can still benefit from the utility
#[tauri::command] 
pub async fn create_project_with_number_refactored(
    project: ProjectCreate,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    execute_with_manager(
        &state,
        |manager| Box::pin(async move {
            // Custom logic before the main operation
            let country_code = manager.get_country_code(&project.country).await?;
            let project_number = manager.generate_project_number(&country_code).await?;
            
            // Create project with generated number
            let mut project_with_number = project;
            project_with_number.number = project_number;
            
            manager.create_project(project_with_number).await
        }),
        "create",
        "project with auto-generated number",
    ).await
}

// =============================================================================
// MIGRATION GUIDE
// =============================================================================

/*
## How to Apply This Refactoring:

### Step 1: Replace Simple CRUD Commands
Replace existing commands like get_companies, create_company, etc. with the macro versions.

### Step 2: Update Tauri Handler Registration  
In lib.rs, replace the old function names with the new refactored versions:
```rust
.invoke_handler(tauri::generate_handler![
    get_companies_refactored,  // instead of get_companies
    create_company_refactored, // instead of create_company
    // ... etc
])
```

### Step 3: Frontend Updates
Update frontend API calls to use the new command names:
```typescript
// Old
await invoke('get_companies')

// New  
await invoke('get_companies_refactored')
```

### Step 4: Test & Validate
1. Run `cargo check` to verify compilation
2. Test each refactored command
3. Verify frontend integration works
4. Remove old command implementations

## Benefits After Refactoring:

- **90% reduction** in command boilerplate code
- **Single point of change** for error handling patterns
- **Consistent logging** across all operations
- **Type safety** maintained through macros
- **Easier testing** with standardized patterns
- **Faster development** of new CRUD commands

## Files Impacted:

1. `/src-tauri/src/commands/mod.rs` - Replace existing commands
2. `/src-tauri/src/lib.rs` - Update invoke_handler registration
3. Frontend API files - Update command names (optional, can keep old names)

The refactoring can be done incrementally - old and new commands can coexist
during the transition period.
*/
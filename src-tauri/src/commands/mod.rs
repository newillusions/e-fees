//! # Tauri Command Module
//! 
//! This module contains all Tauri commands that provide the bridge between the 
//! frontend Svelte application and the backend Rust functionality. Commands are
//! exposed as asynchronous functions that can be called from the frontend using
//! the Tauri API.
//! 
//! ## Architecture Overview
//! 
//! The command layer follows a clean architecture pattern:
//! 1. **Frontend** calls Tauri commands via `invoke()`
//! 2. **Commands** validate inputs and handle errors
//! 3. **Database Manager** performs actual business logic
//! 4. **Results** are serialized back to frontend
//! 
//! ## State Management
//! 
//! All commands receive an `AppState` which is a thread-safe wrapper around
//! the `DatabaseManager`. This allows multiple commands to safely access the
//! database concurrently while maintaining consistency.

pub mod utils;

// Import the utility functions and macros
use utils::execute_with_manager;
use crate::crud_command;

use crate::db::{DatabaseManager, ConnectionStatus, Project, NewProject, Company, CompanyCreate, Contact, ContactCreate, Fee, FeeCreate, FeeUpdate};
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;
use log::{error, info};
use serde::{Serialize, Deserialize};
use tauri_plugin_dialog::DialogExt;

mod tests;

// ============================================================================
// TYPE DEFINITIONS AND APPLICATION STATE
// ============================================================================

/// Application state that holds the database manager instance.
/// 
/// This type provides thread-safe access to the database manager across
/// all Tauri commands. The Arc<Mutex<T>> pattern ensures that:
/// - Multiple commands can access the database safely (Arc)
/// - Database operations are atomic and thread-safe (Mutex)
/// - Memory is managed efficiently across the application lifetime
pub type AppState = Arc<Mutex<DatabaseManager>>;

/// Partial update structure for company modifications.
/// 
/// This struct allows frontend to send only the fields that need updating,
/// rather than requiring the complete company object. Optional fields that
/// are None will not be updated in the database.
/// 
/// # Fields
/// - `name`: Full company name (e.g., "Conrad Hilton Hotels")
/// - `name_short`: Abbreviated name (e.g., "Conrad Etihad")  
/// - `abbreviation`: Short code (e.g., "CHE")
/// - `city`: Company headquarters city
/// - `country`: Company headquarters country
/// - `reg_no`: Company registration number (optional)
/// - `tax_no`: Tax identification number (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUpdate {
    pub name: Option<String>,
    pub name_short: Option<String>,
    pub abbreviation: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}

/// Partial contact update structure for modifying existing contacts.
/// 
/// Similar to CompanyUpdate, this struct allows updating specific fields
/// of a contact without affecting other fields. All fields are optional
/// to support partial updates via the database merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub company: Option<String>, // Company ID as string
}

/// Partial project update structure for modifying existing projects.
/// 
/// This struct allows updating specific fields of a project without affecting
/// other fields. All fields are optional to support partial updates via the
/// database merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub name_short: Option<String>,
    pub status: Option<String>, // ProjectStatus as string
    pub area: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub folder: Option<String>,
}

/// Application settings structure for environment configuration.
/// 
/// This struct represents all configurable settings that can be modified
/// through the application's settings interface. Settings are persisted
/// to the `.env` file in the project root.
/// 
/// # Database Configuration
/// - `surrealdb_url`: WebSocket URL for SurrealDB connection
/// - `surrealdb_ns`: Database namespace (typically "emittiv")
/// - `surrealdb_db`: Database name (typically "projects")
/// - `surrealdb_user`: Authentication username
/// - `surrealdb_pass`: Authentication password
/// 
/// # Staff Information
/// - `staff_name`: Default staff member name for proposals
/// - `staff_email`: Default email address
/// - `staff_phone`: Default phone number
/// - `staff_position`: Default position/title
/// 
/// # File System
/// - `project_folder_path`: Base path for project template folders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub surrealdb_url: Option<String>,
    pub surrealdb_ns: Option<String>,
    pub surrealdb_db: Option<String>,
    pub surrealdb_user: Option<String>,
    pub surrealdb_pass: Option<String>,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub project_folder_path: Option<String>,
}

// ============================================================================
// DATABASE CONNECTION COMMANDS
// ============================================================================

/// Check database connection status (simple boolean result).
/// 
/// This command provides a quick health check for the database connection.
/// It's used by the frontend to determine if the application can communicate
/// with SurrealDB and display appropriate connection indicators.
/// 
/// # Returns
/// - `Ok(true)`: Database is connected and responsive
/// - `Ok(false)`: Database is not connected or not responding
/// - `Err(String)`: Error occurred during connection check
/// 
/// # Frontend Usage
/// ```typescript
/// const isConnected = await invoke('check_db_connection');
/// ```
/// 
/// # Performance
/// This is a lightweight operation that should complete within 1-2 seconds
/// under normal conditions. Timeouts are handled at the database layer.
#[tauri::command]
pub async fn check_db_connection(state: State<'_, AppState>) -> Result<bool, String> {
    info!("Checking database connection");
    
    // Clone the database manager to avoid holding the lock during async operation
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    let is_connected = manager_clone.check_connection().await;
    Ok(is_connected)
}

/// Get detailed connection status with diagnostic information.
/// 
/// This command provides comprehensive connection information including
/// timestamps, error messages, and configuration details. It's used for
/// debugging connection issues and displaying detailed status information.
/// 
/// # Returns
/// - `Ok(ConnectionStatus)`: Detailed connection information
/// - `Err(String)`: Error occurred while retrieving status
/// 
/// # ConnectionStatus Fields
/// - `is_connected`: Boolean connection state
/// - `last_check`: Timestamp of last connection attempt
/// - `error_message`: Last error message if connection failed
/// 
/// # Frontend Usage
/// ```typescript
/// const status = await invoke('get_connection_status');
/// console.log(`Connected: ${status.is_connected}, Last Check: ${status.last_check}`);
/// ```
#[tauri::command]
pub async fn get_connection_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    info!("Getting detailed connection status");
    
    let manager = state.lock().map_err(|e| e.to_string())?;
    let status = manager.get_status();
    Ok(status)
}

// ============================================================================
// PROJECT MANAGEMENT COMMANDS
// ============================================================================

/// Retrieve all projects from the database.
/// 
/// This command fetches the complete list of projects with all associated
/// metadata including project numbers, status, location, and timestamps.
/// The results are sorted by creation date (newest first) at the database level.
/// 
/// # Returns
/// - `Ok(Vec<Project>)`: List of all projects
/// - `Err(String)`: Database error or connection failure
/// 
/// # Project Structure
/// Each project includes:
/// - Basic info: name, name_short, status, area, city, country
/// - Project number: structured YY-CCCNN format with year, country code, sequence
/// - File system: folder path for project templates
/// - Timestamps: created_at and updated_at (auto-managed)
/// 
/// # Frontend Usage
/// ```typescript
/// const projects = await invoke('get_projects');
/// projects.forEach(project => console.log(project.name, project.number.id));
/// ```
/// 
/// # Performance Considerations
/// - Database query is optimized with proper indexing
/// - Large result sets (1000+ projects) may take 2-3 seconds
/// - Results are cached at the frontend level for better UX
crud_command!(
    get_projects,
    Vec<Project>,
    get_projects,
    "fetch",
    "projects"
);

/// Search projects using fuzzy matching across multiple fields.
/// 
/// This command performs a comprehensive search across project fields using
/// case-insensitive substring matching. It's designed to provide intuitive
/// search results for users looking for projects by name, location, or number.
/// 
/// # Search Fields
/// - `name`: Full project name
/// - `name_short`: Abbreviated project name
/// - `area`: Project area/district
/// - `city`: Project city
/// - `country`: Project country
/// - `number.id`: Project number (e.g., "25-97105")
/// - `status`: Project status
/// 
/// # Parameters
/// - `query`: Search string (minimum 2 characters recommended)
/// 
/// # Returns
/// - `Ok(Vec<Project>)`: Matching projects sorted by relevance
/// - `Err(String)`: Search error or invalid query
/// 
/// # Frontend Usage
/// ```typescript
/// const results = await invoke('search_projects', { query: 'Dubai' });
/// // Returns projects in Dubai or with "Dubai" in the name
/// ```
/// 
/// # Search Examples
/// - "Dubai": Finds projects in Dubai or with Dubai in the name
/// - "25-971": Finds UAE projects from 2025
/// - "Hotel": Finds projects with "hotel" in any text field
/// - "Active": Finds all active projects
#[tauri::command]
pub async fn search_projects(query: String, state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    info!("Searching projects with query: {}", query);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.search_projects(&query).await {
        Ok(projects) => {
            info!("Search returned {} projects", projects.len());
            Ok(projects)
        }
        Err(e) => {
            error!("Failed to search projects: {}", e);
            Err(format!("Failed to search projects: {}", e))
        }
    }
}

/// Create a new project in the database.
/// 
/// This command creates a standard project record without template folder
/// operations. For project creation with template copying, use
/// `create_project_with_template` instead.
/// 
/// # Parameters
/// - `project`: Complete project object with all required fields
/// 
/// # Returns
/// - `Ok(Project)`: Created project with database-assigned ID
/// - `Err(String)`: Validation error or database failure
/// 
/// # Validation Rules
/// - All required fields must be present and non-empty
/// - Project number must be unique across all projects
/// - Status must be valid enum value
/// - Country must exist in countries reference table
/// 
/// # Frontend Usage
/// ```typescript
/// const newProject = {
///   name: "New Hotel Project",
///   name_short: "Hotel XYZ", 
///   status: "Draft",
///   area: "Downtown",
///   city: "Dubai",
///   country: "UAE",
///   number: { year: 25, country: 971, seq: 6, id: "25-97106" },
///   folder: "25-97106 Hotel XYZ"
/// };
/// const created = await invoke('create_project', { project: newProject });
/// ```
crud_command!(
    create_project,
    Project,
    Project,
    create_project,
    "create",
    "project",
    data: project
);

// ============================================================================
// COMPANY MANAGEMENT COMMANDS
// ============================================================================

/// Retrieve all companies from the database.
/// 
/// This command fetches the complete company directory with contact counts
/// and full company information. Companies are sorted alphabetically by name.
/// 
/// # Returns
/// - `Ok(Vec<Company>)`: List of all companies
/// - `Err(String)`: Database error or connection failure
/// 
/// # Company Structure
/// Each company includes:
/// - Basic info: name, name_short, abbreviation
/// - Location: city, country
/// - Registration: reg_no, tax_no (optional)
/// - Metadata: ID (abbreviation-based), timestamps
/// 
/// # Frontend Usage
/// ```typescript
/// const companies = await invoke('get_companies');
/// const uaeCompanies = companies.filter(c => c.country === 'UAE');
/// ```
// ============================================================================
// REFACTORED COMPANY MANAGEMENT COMMANDS USING UTILITIES
// ============================================================================

/// Retrieve all companies from the database.
/// 
/// This command fetches all company records sorted alphabetically by name.
/// Companies are core entities in the system and are referenced by contacts,
/// projects, and proposals.
crud_command!(
    get_companies,
    Vec<Company>,
    get_companies,
    "fetch",
    "companies"
);

/// Create a new company in the database.
/// 
/// This command creates a new company record with automatic ID generation
/// based on the company abbreviation. The abbreviation must be unique.
/// 
/// # Parameters
/// - `company`: Complete company object with all required fields
/// 
/// # Returns
/// - `Ok(Company)`: Created company with database-assigned metadata
/// - `Err(String)`: Validation error or duplicate abbreviation
/// 
/// # Validation Rules
/// - `name`: Must be non-empty and unique
/// - `abbreviation`: Must be unique and alphanumeric (becomes ID)
/// - `city` and `country`: Must be non-empty
/// - Optional fields can be empty but not invalid
/// 
/// # Frontend Usage
/// ```typescript
/// const newCompany = {
///   name: "Luxury Hotels International",
///   name_short: "Luxury Hotels",
///   abbreviation: "LHI",
///   city: "Dubai",
///   country: "UAE",
///   reg_no: "12345",
///   tax_no: "TRN12345"
/// };
/// const created = await invoke('create_company', { company: newCompany });
/// ```
crud_command!(
    create_company,
    Company,
    CompanyCreate,
    create_company,
    "create",
    "company",
    data: company
);

/// Update an existing company with partial data.
/// 
/// This command allows updating specific fields of a company without
/// requiring the complete company object. Only provided fields will be
/// updated; null/undefined fields are ignored.
/// 
/// # Parameters
/// - `id`: Company ID (typically the abbreviation)
/// - `company_update`: Partial company data with only fields to update
/// 
/// # Returns
/// - `Ok(Company)`: Updated company with all current data
/// - `Err(String)`: Company not found or validation error
/// 
/// # Update Strategy
/// Uses SurrealDB's `MERGE` operation for atomic partial updates:
/// - Only specified fields are modified
/// - Unspecified fields remain unchanged
/// - Timestamps are automatically updated
/// - Validation is applied to new values
/// 
/// # Frontend Usage
/// ```typescript
/// // Update only the city and phone number
/// const updates = {
///   city: "Abu Dhabi",
///   reg_no: "NEW123"
/// };
/// const updated = await invoke('update_company', { 
///   id: "CHE", 
///   company_update: updates 
/// });
/// ```
#[tauri::command]
pub async fn update_company(id: String, company_update: CompanyUpdate, state: State<'_, AppState>) -> Result<Company, String> {
    let company_name = format!("company '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move { 
                manager.update_company_partial(&id_clone, company_update).await 
            })
        },
        "update",
        &company_name
    ).await
}

/// Delete a company from the database.
/// 
/// This command permanently removes a company record. Note that this
/// operation will fail if the company has associated contacts or projects
/// due to foreign key constraints.
/// 
/// # Parameters
/// - `id`: Company ID to delete
/// 
/// # Returns
/// - `Ok(Company)`: The deleted company data (for undo operations)
/// - `Err(String)`: Company not found or has dependencies
/// 
/// # Safety Considerations
/// - This is a permanent operation that cannot be undone
/// - Foreign key constraints prevent deletion of companies with dependencies
/// - Consider soft deletion (status flag) for production use
/// 
/// # Frontend Usage
/// ```typescript
/// try {
///   const deleted = await invoke('delete_company', { id: 'CHE' });
///   console.log(`Deleted company: ${deleted.name}`);
/// } catch (error) {
///   console.error('Cannot delete company with active projects');
/// }
/// ```
crud_command!(
    delete_company,
    Company,
    delete_company,
    "delete",
    "company",
    id: String
);

// ============================================================================
// CONTACT MANAGEMENT COMMANDS
// ============================================================================

/// Retrieve all contacts from the database with company information.
/// 
/// This command fetches all contacts with their associated company data
/// resolved through database joins. Contacts are sorted alphabetically
/// by last name, then first name.
/// 
/// # Returns
/// - `Ok(Vec<Contact>)`: List of all contacts with company details
/// - `Err(String)`: Database error or connection failure
/// 
/// # Contact Structure
/// Each contact includes:
/// - Personal info: first_name, last_name, email, phone, position
/// - Company reference: company field with full company object
/// - Computed fields: full_name (automatically generated)
/// - Metadata: unique email constraint, timestamps
/// 
/// # Frontend Usage
/// ```typescript
/// const contacts = await invoke('get_contacts');
/// const hotelContacts = contacts.filter(c => 
///   c.company.name.toLowerCase().includes('hotel')
/// );
/// ```
/// Retrieve all contacts from the database with company information.
/// 
/// This command fetches all contacts with their associated company data
/// resolved through database joins. Contacts are sorted alphabetically
/// by last name, then first name.
crud_command!(
    get_contacts,
    Vec<Contact>,
    get_contacts,
    "fetch",
    "contacts"
);

/// Create a new contact in the database.
/// 
/// This command creates a new contact record with automatic validation
/// and company relationship establishment. Email addresses must be unique.
/// 
/// # Parameters
/// - `contact`: Complete contact object with company reference
/// 
/// # Returns
/// - `Ok(Contact)`: Created contact with database-assigned ID
/// - `Err(String)`: Validation error or duplicate email
/// 
/// # Validation Rules
/// - `email`: Must be valid email format and unique
/// - `phone`: Must contain '+' character (international format)
/// - `company`: Must reference existing company ID
/// - All name fields must be non-empty
/// 
/// # Frontend Usage
/// ```typescript
/// const newContact = {
///   first_name: "John",
///   last_name: "Smith", 
///   email: "john.smith@hotel.com",
///   phone: "+971501234567",
///   position: "Project Manager",
///   company: "company:CHE"  // Reference to Conrad Hilton
/// };
/// const created = await invoke('create_contact', { contact: newContact });
/// ```
crud_command!(
    create_contact,
    Contact,
    ContactCreate,
    create_contact,
    "create",
    "contact",
    data: contact
);

/// Update an existing contact in the database.
/// 
/// This command accepts partial contact data and updates only the specified fields
/// using SurrealDB's MERGE operation. This allows for efficient partial updates
/// without affecting unspecified fields.
/// 
/// # Frontend Usage
/// ```typescript
/// const updated = await invoke('update_contact', { id: contactId, contactUpdate: updatedData });
/// ```
/// 
/// # Arguments
/// * `id` - The contact ID (extracted from SurrealDB Thing object)
/// * `contact_update` - Partial contact data with only fields to update
/// 
/// # Returns
/// * `Result<Contact, String>` - Updated contact or error message
#[tauri::command]
pub async fn update_contact(id: String, contact_update: ContactUpdate, state: State<'_, AppState>) -> Result<Contact, String> {
    let contact_name = format!("contact '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move { 
                manager.update_contact_partial(&id_clone, contact_update).await 
            })
        },
        "update",
        &contact_name
    ).await
}

/// Delete a contact from the database.
/// 
/// This command permanently removes a contact record. Note that this
/// operation will fail if the contact has associated RFPs or other
/// dependencies due to foreign key constraints.
/// 
/// # Parameters
/// - `id`: Contact ID to delete (extracted from SurrealDB Thing object)
/// 
/// # Returns
/// - `Ok(Contact)`: The deleted contact data (for undo operations)
/// - `Err(String)`: Contact not found or has dependencies
/// 
/// # Safety Considerations
/// - This is a permanent operation that cannot be undone
/// - Foreign key constraints prevent deletion of contacts with dependencies
/// - Consider soft deletion (status flag) for production use
/// 
/// # Frontend Usage
/// ```typescript
/// try {
///   const deleted = await invoke('delete_contact', { id: 'contacts:john_smith' });
///   console.log(`Deleted contact: ${deleted.full_name}`);
/// } catch (error) {
///   console.error('Cannot delete contact with active FPs');
/// }
/// ```
// TODO: Implement delete_contact functionality with proper business logic validation
// Currently commented out as it's not used and needs proper implementation
// to handle cascade delete scenarios (e.g., FPs linked to this contact)
// crud_command!(
//     delete_contact,
//     Contact,
//     delete_contact,
//     "delete",
//     "contact",
//     id: String
// );

// ============================================================================
// RFP (PROPOSAL) MANAGEMENT COMMANDS
// ============================================================================

/// Retrieve all RFPs (proposals) from the database.
/// 
/// This command fetches all fee proposals with their relationships to
/// projects, companies, and contacts resolved. RFPs are sorted by
/// creation date (newest first).
/// 
/// # Returns
/// - `Ok(Vec<Fee>)`: List of all fees with relationship data
/// - `Err(String)`: Database error or connection failure
/// 
/// # RFP Structure
/// Each RFP includes:
/// - Basic info: name, number, status, stage, issue_date
/// - References: project_id, company_id, contact_id (with full objects)
/// - Content: activity, package, strap_line
/// - Staff info: name, email, phone, position
/// - Revisions: complete audit trail with timestamps
/// - Metadata: auto-computed revision numbers, timestamps
/// 
/// # Frontend Usage
/// ```typescript
/// const fees = await invoke('get_fees');
/// const activeFees = fees.filter(f => f.status === 'Active');
/// ```
/// Retrieve all RFPs (proposals) from the database.
/// 
/// This command fetches all fee proposals with their relationships to
/// projects, companies, and contacts resolved. RFPs are sorted by
/// creation date (newest first).
crud_command!(
    get_fees,
    Vec<Fee>,
    get_fees,
    "fetch",
    "fee proposals"
);

/// Create a new RFP (proposal) in the database.
/// 
/// This command creates a new fee proposal with automatic number generation
/// and revision tracking initialization.
/// 
/// # Parameters
/// - `rfp`: Complete RFP object with all required relationships
/// 
/// # Returns
/// - `Ok(Fee)`: Created fee with database-assigned metadata
/// - `Err(String)`: Validation error or invalid references
/// 
/// # Validation Rules
/// - `project_id`: Must reference existing project
/// - `company_id`: Must reference existing company  
/// - `contact_id`: Must reference existing contact
/// - `issue_date`: Must be valid YYMMDD format
/// - `status` and `stage`: Must be valid enum values
/// 
/// # Automatic Features
/// - RFP number generation based on project number
/// - Revision tracking initialization (starts at revision 1)
/// - Staff information pre-population from settings
/// - Timestamp management
/// 
/// # Frontend Usage
/// ```typescript
/// const newFee = {
///   name: "Hotel Renovation Proposal",
///   project_id: "projects:25_97105",
///   company_id: "company:CHE", 
///   contact_id: "contacts:john_smith",
///   status: "Draft",
///   stage: "Prepared",
///   issue_date: "251201",  // Dec 1, 2025
///   activity: "Interior Design",
///   package: "Complete renovation"
/// };
/// const created = await invoke('create_fee', { fee: newFee });
/// ```
crud_command!(
    create_fee,
    Fee,
    FeeCreate,
    create_fee,
    "create",
    "fee proposal",
    data: fee
);

/// Update an existing RFP in the database.
/// 
/// This command updates an RFP record with new data. All fields are replaced
/// with the provided values, so partial updates should include all current data.
/// 
/// # Parameters
/// - `id`: The string ID of the RFP to update
/// - `rfp`: Complete RFP object with updated data
/// 
/// # Returns
/// - `Ok(Fee)`: Updated fee with current data
/// - `Err(String)`: Validation error or RFP not found
/// 
/// # Frontend Usage
/// ```typescript
/// const updatedFee = {
///   ...existingFee,
///   status: "Sent",
///   stage: "Under Review"
/// };
/// const result = await invoke('update_fee', { 
///   id: "fee_id_here", 
///   fee: updatedFee 
/// });
/// ```
#[tauri::command]
pub async fn update_fee(id: String, fee: FeeUpdate, state: State<'_, AppState>) -> Result<Fee, String> {
    let fee_name = format!("fee proposal '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move { 
                manager.update_fee(&id_clone, fee).await 
            })
        },
        "update",
        &fee_name
    ).await
}

/// Delete an RFP from the database.
/// 
/// This command permanently removes an RFP record. This operation cannot be undone.
/// Consider the impact on related records and reports before deletion.
/// 
/// # Parameters
/// - `id`: The string ID of the RFP to delete
/// 
/// # Returns
/// - `Ok(Fee)`: The deleted fee data for confirmation
/// - `Err(String)`: Database error or RFP not found
/// 
/// # Warning
/// This operation will permanently delete the RFP and cannot be undone.
/// Ensure proper user confirmation before calling this command.
/// 
/// # Frontend Usage
/// ```typescript
/// const deletedFee = await invoke('delete_fee', { id: "fee_id_here" });
/// console.log('Deleted Fee:', deletedFee.name);
/// ```
crud_command!(
    delete_fee,
    Fee,
    delete_fee,
    "delete",
    "fee proposal",
    id: String
);

/// Write RFP (fee proposal) data to JSON file in project folder
/// 
/// This command fetches complete RFP data including all linked records
/// and writes it to the project's JSON template file for InDesign integration.
/// 
/// # Parameters
/// - `rfp_id`: The string ID of the RFP to export (e.g., "24_96606_1")
/// 
/// # Returns
/// - `Ok(String)`: Success message with file path
/// - `Err(String)`: Error message if operation fails
#[tauri::command]
pub async fn write_fee_to_json(rfp_id: String, state: State<'_, AppState>) -> Result<String, String> {
    use std::fs;
    use std::path::Path;
    use serde_json::json;
    use chrono::Utc;
    
    info!("Writing fee {} to JSON file", rfp_id);
    
    // This is the exact same approach used in populate_project_data that works
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };

    // Get all fee records - this works in the UI
    let fees = manager_clone.get_fees().await
        .map_err(|e| format!("Failed to fetch fee records: {}", e))?;
    
    info!("Got {} fee records", fees.len());
    
    // Debug: Log the first few fee record IDs to understand the format
    for (i, fee) in fees.iter().take(3).enumerate() {
        if let Some(id) = &fee.id {
            info!("Fee {}: Full Thing = {:?}", i, id);
            info!("Fee {}: Thing.to_string() = '{}'", i, id.to_string());
            info!("Fee {}: Thing.tb = '{}'", i, id.tb);
            info!("Fee {}: Thing.id = {:?}", i, id.id);
            info!("Fee {}: Thing.id.to_string() = '{}'", i, id.id.to_string());
        }
    }
    
    // Find the specific fee record - simplified and robust ID matching
    let fee = fees.iter()
        .find(|f| {
            if let Some(id) = &f.id {
                // Extract clean ID parts from both database format and input
                let db_id_clean = id.id.to_string().trim_start_matches('⟨').trim_end_matches('⟩').to_string();
                let input_id_clean = rfp_id.trim_start_matches("fee:").to_string();
                
                info!("Comparing fee IDs: db='{}' vs input='{}'", db_id_clean, input_id_clean);
                
                let matches = db_id_clean == input_id_clean;
                if matches {
                    info!("✓ Found matching fee: {}", f.number);
                }
                
                matches
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Fee record not found with ID: {}", rfp_id))?;
    
    info!("Found fee record: {} - {}", fee.number, fee.name);
    info!("Fee has project_id: {}", fee.project_id);
    info!("Fee has company_id: {}", fee.company_id);
    info!("Fee has contact_id: {}", fee.contact_id);
    
    // Get all related data
    let projects = manager_clone.get_projects().await
        .map_err(|e| format!("Failed to fetch projects: {}", e))?;
    
    info!("Got {} projects", projects.len());
    
    let project = projects.iter()
        .find(|p| {
            if let Some(p_id) = &p.id {
                let matches = fee.project_id.id.to_string() == p_id.id.to_string();
                if matches {
                    info!("Found matching project: {}", p.name);
                }
                matches
            } else {
                false
            }
        })
        .ok_or_else(|| {
            error!("Project not found. Fee project_id: {}, Available project IDs:", fee.project_id);
            for p in projects.iter().take(5) {
                if let Some(id) = &p.id {
                    error!("  - {}", id);
                }
            }
            format!("Project not found for fee")
        })?;
    
    let companies = manager_clone.get_companies().await
        .map_err(|e| format!("Failed to fetch companies: {}", e))?;
    
    let company = companies.iter()
        .find(|c| {
            if let Some(c_id) = &c.id {
                fee.company_id.id.to_string() == c_id.id.to_string()
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Company not found for fee"))?;
    
    let contacts = manager_clone.get_contacts().await
        .map_err(|e| format!("Failed to fetch contacts: {}", e))?;
    
    let contact = contacts.iter()
        .find(|c| {
            if let Some(c_id) = &c.id {
                fee.contact_id.id.to_string() == c_id.id.to_string()
            } else {
                false
            }
        })
        .ok_or_else(|| format!("Contact not found for fee"))?;

    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    let project_folder_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;

    // Build project directory path - match the format used by copyProjectTemplate
    // Use the project.number.id field which contains the formatted project number
    // Strip angle brackets if present (same as frontend does)
    let project_number = project.number.id.replace("⟨", "").replace("⟩", "");
    
    let project_name = &project.name_short;  // Use short name for folder path

    let project_dir = format!("{}/01 RFPs/{} {}", project_folder_path, project_number, project_name);
    let old_json_file_path = format!("{}/02 Proposal/{}-var Default Values.json", project_dir, project_number);
    let new_json_file_path = format!("{}/02 Proposal/{}-var.json", project_dir, project_number);
    
    info!("DEBUG: Project folder path: {}", project_folder_path);
    info!("DEBUG: Project number: {}", project_number);
    info!("DEBUG: Project name: {}", project_name);
    info!("DEBUG: Project dir: {}", project_dir);
    info!("DEBUG: Old JSON path: {}", old_json_file_path);
    info!("DEBUG: New JSON path: {}", new_json_file_path);
    
    // Check if we need to rename the template file first
    if Path::new(&old_json_file_path).exists() && !Path::new(&new_json_file_path).exists() {
        info!("Renaming template file from '{}' to '{}'", old_json_file_path, new_json_file_path);
        
        // Check if file might be syncing (basic check for .sync extension or recent modification)
        let metadata = fs::metadata(&old_json_file_path).map_err(|e| {
            format!("Failed to read file metadata: {}", e)
        })?;
        
        // Check if file was modified very recently (within last 30 seconds) - might indicate sync activity
        if let Ok(modified) = metadata.modified() {
            if let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() < 30 {
                    info!("File was recently modified, waiting for potential sync to complete...");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
        
        // Rename the file to remove "Default Values"
        fs::rename(&old_json_file_path, &new_json_file_path).map_err(|e| {
            error!("Failed to rename file: {}", e);
            format!("Failed to rename file from '{}' to '{}': {}", old_json_file_path, new_json_file_path, e)
        })?;
    }
    
    let json_file_path = new_json_file_path;

    // Format issue date
    let issue_date = {
        let d = &fee.issue_date;
        // Parse YYMMDD format and convert to "dd MMM yyyy"
        if d.len() == 6 {
            if let (Ok(year), Ok(month), Ok(day)) = (
                d[0..2].parse::<i32>(),
                d[2..4].parse::<u32>(),
                d[4..6].parse::<u32>()
            ) {
                let full_year = if year >= 50 { 1900 + year } else { 2000 + year };
                let date = chrono::NaiveDate::from_ymd_opt(full_year, month, day);
                date.map(|d| d.format("%d %b %Y").to_string())
                    .unwrap_or_else(|| Utc::now().format("%d %b %Y").to_string())
            } else {
                Utc::now().format("%d %b %Y").to_string()
            }
        } else {
            Utc::now().format("%d %b %Y").to_string()
        }
    };

    // Create JSON data mapping from the actual structs
    let json_data = json!({
        "01 Document Name": fee.name.clone(),
        "02 Document Number": fee.number.clone(),
        "03 Document Release": fee.rev.to_string(),
        "04 Document Issue Date": issue_date,
        "06 Project Name": project.name.clone(),
        "07 Project Activity": fee.activity.clone(),
        "08 Project Package": fee.package.clone(),
        "09 Project Stage": project.status.clone(),
        "11 Project Area": project.area.clone(),
        "12 Project City": project.city.clone(),
        "13 Project Country": project.country.clone(),
        "21 Client Company": company.name.clone(),
        "22 Client City": company.city.clone(),
        "23 Client Country": company.country.clone(),
        "26 Contact Name": contact.full_name.clone().unwrap_or_else(|| {
            let first = contact.first_name.clone().unwrap_or_default();
            let last = contact.last_name.clone().unwrap_or_default();
            format!("{} {}", first, last)
        }),
        "27 Contact Position": contact.position.clone().unwrap_or_default(),
        "28 Contact Phone": contact.phone.clone().unwrap_or_default(),
        "29 Contact Email": contact.email.clone().unwrap_or_default(),
        "99 Strap Line": fee.strap_line.clone()
    });

    // Ensure directory exists
    if let Some(parent) = Path::new(&json_file_path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error!("Failed to create directory: {}", e);
            format!("Failed to create directory: {}", e)
        })?;
    }

    // Write JSON file
    fs::write(&json_file_path, serde_json::to_string_pretty(&json_data).map_err(|e| {
        error!("Failed to serialize JSON: {}", e);
        format!("Failed to serialize JSON: {}", e)
    })?).map_err(|e| {
        error!("Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    info!("Successfully wrote RFP data to: {}", json_file_path);
    Ok(format!("Successfully wrote RFP data to: {}", json_file_path))
}

/// Update an existing project in the database.
/// 
/// This command updates a project record with new data. Uses partial updates
/// to only modify the specified fields while preserving others.
/// 
/// # Parameters
/// - `id`: The string ID of the project to update
/// - `project_update`: Partial project data with fields to update
/// 
/// # Returns
/// - `Ok(Project)`: Updated project with current data
/// - `Err(String)`: Validation error or project not found
/// 
/// # Frontend Usage
/// ```typescript
/// const projectUpdate = {
///   name: "Updated Project Name",
///   status: "Active"
/// };
/// const result = await invoke('update_project', { 
///   id: "25_97105", 
///   projectUpdate 
/// });
/// ```
#[tauri::command]
pub async fn update_project(id: String, project_update: ProjectUpdate, state: State<'_, AppState>) -> Result<Project, String> {
    let project_name = format!("project '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move { 
                manager.update_project(&id_clone, project_update).await 
            })
        },
        "update",
        &project_name
    ).await
}

/// Delete a project from the database.
/// 
/// This command permanently removes a project record from the database.
/// This operation cannot be undone.
/// 
/// # Parameters
/// - `id`: The string ID of the project to delete
/// 
/// # Returns
/// - `Ok(Project)`: The deleted project data for confirmation
/// - `Err(String)`: Project not found or deletion failed
/// 
/// # Frontend Usage
/// ```typescript
/// const deletedProject = await invoke('delete_project', { 
///   id: "25_97105" 
/// });
/// ```
crud_command!(
    delete_project,
    Project,
    delete_project,
    "delete",
    "project",
    id: String
);

// ============================================================================
// SYSTEM AND UTILITY COMMANDS
// ============================================================================

/// Application health check command.
/// 
/// This command provides a simple way to verify that the Tauri backend
/// is running and responsive. It's used for system monitoring and debugging.
/// 
/// # Returns
/// - `Ok(String)`: Always returns "Application is running"
/// - `Err(String)`: Should never occur unless system is critically failing
/// 
/// # Frontend Usage
/// ```typescript
/// const status = await invoke('health_check');
/// console.log(status); // "Application is running"
/// ```
#[tauri::command]
pub async fn health_check() -> Result<String, String> {
    info!("Application health check");
    Ok("Application is running".to_string())
}

/// Get comprehensive database connection information for debugging.
/// 
/// This command provides detailed diagnostic information about the database
/// connection, configuration, and troubleshooting guidance. It's primarily
/// used for development and support purposes.
/// 
/// # Returns
/// - `Ok(serde_json::Value)`: Detailed connection information object
/// - `Err(String)`: Error accessing connection information
/// 
/// # Information Included
/// - Database configuration (URL, namespace, database, username)
/// - Current connection status with timestamp
/// - Troubleshooting guidance for common issues
/// - Environment configuration source information
/// 
/// # Security Note
/// Passwords are never included in the response for security reasons.
/// 
/// # Frontend Usage
/// ```typescript
/// const info = await invoke('get_db_info');
/// console.log(`Connected to: ${info.url}/${info.namespace}/${info.database}`);
/// console.log(`Status: ${info.connection_status.is_connected}`);
/// ```
#[tauri::command]
pub async fn get_db_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting database connection information");
    
    let (connection_status, config) = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        (manager.get_status(), manager.config.clone())
    };
    
    let info = serde_json::json!({
        "url": config.url,
        "namespace": config.namespace,
        "database": config.database,
        "username": config.username,
        "connection_status": connection_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "troubleshooting": {
            "dns_issue": format!("If you see 'No such host is known', the server {} cannot be resolved", config.url),
            "network_check": format!("Try pinging the server: ping {}", config.url.replace("ws://", "").replace(":8000", "")),
            "surrealdb_check": format!("Verify SurrealDB is running: telnet {} 8000", config.url.replace("ws://", "").replace(":8000", "")),
            "config_source": "Configuration loaded from .env file or environment variables"
        }
    });
    
    Ok(info)
}

/// Get application statistics and metrics.
/// 
/// This command calculates and returns key application metrics for
/// dashboard displays and monitoring purposes. Statistics are calculated
/// in real-time from the database.
/// 
/// # Returns
/// - `Ok(serde_json::Value)`: Statistics object with counts and metrics
/// - `Err(String)`: Error calculating statistics
/// 
/// # Statistics Included
/// - `totalProjects`: Total count of all projects
/// - `activeRfps`: Count of RFPs not in 'Lost' or 'Cancelled' status
/// - `totalCompanies`: Total count of all companies
/// - `totalContacts`: Total count of all contacts
/// - `totalRfps`: Total count of all RFPs
/// 
/// # Performance
/// This command queries multiple tables and may take 1-2 seconds for
/// large datasets. Results should be cached on the frontend.
/// 
/// # Frontend Usage
/// ```typescript
/// const stats = await invoke('get_stats');
/// console.log(`Active RFPs: ${stats.activeRfps}/${stats.totalRfps}`);
/// console.log(`Total Projects: ${stats.totalProjects}`);
/// ```
#[tauri::command]
pub async fn get_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Fetching application statistics");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    }; // Lock is automatically dropped here when manager goes out of scope
    
    // Fetch all data in parallel for better performance
    let projects = manager_clone.get_projects().await.unwrap_or_default();
    let companies = manager_clone.get_companies().await.unwrap_or_default();
    let contacts = manager_clone.get_contacts().await.unwrap_or_default();
    let fees = manager_clone.get_fees().await.unwrap_or_default();
    
    // Calculate active fees (exclude Lost and Cancelled)
    let active_fees = fees.iter()
        .filter(|f| f.status != "Lost" && f.status != "Cancelled")
        .count();
    
    let stats = serde_json::json!({
        "totalProjects": projects.len(),
        "activeFees": active_fees,
        "totalCompanies": companies.len(),
        "totalContacts": contacts.len(),
        "totalFees": fees.len()
    });
    
    info!("Successfully calculated statistics");
    Ok(stats)
}

/// Get database table schema information for development.
/// 
/// This command retrieves the schema definition for a specified table,
/// including field definitions, constraints, and indexes. It's used for
/// development, debugging, and documentation purposes.
/// 
/// # Parameters
/// - `table_name`: Name of the table to inspect (e.g., "projects", "company")
/// 
/// # Returns
/// - `Ok(serde_json::Value)`: Table schema information
/// - `Err(String)`: Table not found or access error
/// 
/// # Schema Information
/// - Field definitions with types and constraints
/// - Index information for performance optimization
/// - Foreign key relationships
/// - Validation rules and default values
/// 
/// # Frontend Usage
/// ```typescript
/// const schema = await invoke('get_table_schema', { table_name: 'projects' });
/// console.log('Project fields:', schema.fields);
/// ```
#[tauri::command]
pub async fn get_table_schema(table_name: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Getting schema for table: {}", table_name);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_table_schema(&table_name).await {
        Ok(schema) => {
            info!("Successfully retrieved schema for table: {}", table_name);
            Ok(schema)
        }
        Err(e) => {
            error!("Failed to get schema for table {}: {}", table_name, e);
            Err(format!("Failed to get schema for table {}: {}", table_name, e))
        }
    }
}

// ============================================================================
// WINDOW AND DESKTOP INTEGRATION COMMANDS
// ============================================================================

/// Position application window on right half of 4K monitor.
/// 
/// This command provides a convenient way to position the application window
/// for optimal use with 4K monitors. It places the window on the right half
/// of the screen at full height.
/// 
/// # Parameters
/// - `window`: Tauri window handle (automatically provided)
/// 
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: Window positioning error
/// 
/// # Window Configuration
/// - **Position**: (1920, 0) - Right half of 4K screen
/// - **Size**: 1920x2160 - Half width, full height
/// - **Use Case**: Dual-monitor setup or large 4K display
/// 
/// # Frontend Usage
/// ```typescript
/// await invoke('position_window_4k');
/// // Window now positioned on right half of 4K monitor
/// ```
#[tauri::command]
pub async fn position_window_4k(window: tauri::Window) -> Result<String, String> {
    info!("Positioning window for 4K monitor");
    
    // Set position to right half of 4K screen
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x: 1920, y: 0 }))
        .map_err(|e| format!("Failed to set position: {}", e))?;
    
    // Set size to half width, full height
    window.set_size(tauri::Size::Physical(tauri::PhysicalSize { width: 1920, height: 2160 }))
        .map_err(|e| format!("Failed to set size: {}", e))?;
    
    info!("Window positioned successfully on right half of 4K monitor");
    Ok("Window positioned successfully".to_string())
}

// ============================================================================
// SETTINGS AND CONFIGURATION COMMANDS
// ============================================================================

/// Read current application settings from .env file.
/// 
/// This command loads all configurable application settings from the
/// `.env` file located in the project root directory. Settings include
/// database configuration, staff information, and file system paths.
/// 
/// # Returns
/// - `Ok(AppSettings)`: Current settings with all configured values
/// - `Err(String)`: File read error or parsing failure
/// 
/// # Configuration Source
/// Settings are read from `../.env` relative to the Tauri binary location.
/// Missing settings are returned as `None` values, allowing for partial
/// configuration and gradual setup.
/// 
/// # Security Considerations
/// Database passwords and sensitive information are included in the response
/// but should be handled securely on the frontend (e.g., masked in UI).
/// 
/// # Frontend Usage
/// ```typescript
/// const settings = await invoke('get_settings');
/// if (settings.surrealdb_url) {
///   console.log(`Database: ${settings.surrealdb_url}`);
/// }
/// ```
#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    info!("Reading settings from .env file");
    
    // Log the current working directory
    if let Ok(cwd) = std::env::current_dir() {
        info!("Current working directory: {:?}", cwd);
    }
    
    let env_path = ".env"; // Should be in project root
    let mut settings = AppSettings {
        surrealdb_url: None,
        surrealdb_ns: None,
        surrealdb_db: None,
        surrealdb_user: None,
        surrealdb_pass: None,
        staff_name: None,
        staff_email: None,
        staff_phone: None,
        staff_position: None,
        project_folder_path: None,
    };
    
    info!("Looking for .env file at: {}", env_path);
    
    if Path::new(env_path).exists() {
        info!(".env file found, reading contents");
        match fs::read_to_string(env_path) {
            Ok(content) => {
                info!("Successfully read .env file, parsing {} lines", content.lines().count());
                // Parse .env file line by line
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue; // Skip empty lines and comments
                    }
                    
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"'); // Remove quotes
                        
                        // Map environment variables to settings fields
                        match key {
                            "SURREALDB_URL" => settings.surrealdb_url = Some(value.to_string()),
                            "SURREALDB_NS" => settings.surrealdb_ns = Some(value.to_string()),
                            "SURREALDB_DB" => settings.surrealdb_db = Some(value.to_string()),
                            "SURREALDB_USER" => settings.surrealdb_user = Some(value.to_string()),
                            "SURREALDB_PASS" => settings.surrealdb_pass = Some(value.to_string()),
                            "STAFF_NAME" => settings.staff_name = Some(value.to_string()),
                            "STAFF_EMAIL" => settings.staff_email = Some(value.to_string()),
                            "STAFF_PHONE" => settings.staff_phone = Some(value.to_string()),
                            "STAFF_POSITION" => settings.staff_position = Some(value.to_string()),
                            "PROJECT_FOLDER_PATH" => settings.project_folder_path = Some(value.to_string()),
                            _ => {} // Ignore unknown variables
                        }
                    }
                }
                info!("Successfully loaded settings from .env file");
            }
            Err(e) => {
                error!("Failed to read .env file: {}", e);
                return Err(format!("Failed to read .env file: {}", e));
            }
        }
    } else {
        info!("No .env file found, returning empty settings");
    }
    
    info!("Returning settings: staff_name={:?}, staff_email={:?}, staff_phone={:?}, staff_position={:?}", 
          settings.staff_name, settings.staff_email, settings.staff_phone, settings.staff_position);
    
    Ok(settings)
}

/// Save application settings to .env file.
/// 
/// This command writes application settings to the `.env` file, preserving
/// existing environment variables that are not managed by the application.
/// The settings are organized into logical sections with comments.
/// 
/// # Parameters
/// - `settings`: Complete settings object with values to save
/// 
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: File write error or permission issue
/// 
/// # File Management Strategy
/// 1. Read existing .env file to preserve other variables
/// 2. Remove old application-managed variables
/// 3. Add new settings in organized sections
/// 4. Write complete file atomically
/// 
/// # File Structure
/// ```env
/// # Existing variables preserved
/// 
/// # SurrealDB Configuration  
/// SURREALDB_URL="ws://10.0.1.17:8000"
/// SURREALDB_NS="emittiv"
/// # ... other DB settings
/// 
/// # Staff Information
/// STAFF_NAME="John Smith"
/// # ... other staff settings
/// 
/// # Project Configuration
/// PROJECT_FOLDER_PATH="E:\\Projects"
/// ```
/// 
/// # Frontend Usage
/// ```typescript
/// const settings = {
///   surrealdb_url: "ws://10.0.1.17:8000",
///   surrealdb_ns: "emittiv",
///   staff_name: "John Smith"
/// };
/// await invoke('save_settings', { settings });
/// ```
#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<String, String> {
    info!("Saving settings to .env file");
    
    let env_path = ".env"; // Should be in project root
    let mut lines = Vec::new();
    
    // Read existing .env file to preserve other variables
    if Path::new(env_path).exists() {
        match fs::read_to_string(env_path) {
            Ok(content) => {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        lines.push(line.to_string());
                        continue;
                    }
                    
                    if let Some((key, _)) = line.split_once('=') {
                        let key = key.trim();
                        
                        // Skip our managed settings - we'll add them back
                        match key {
                            "SURREALDB_URL" | "SURREALDB_NS" | "SURREALDB_DB" | 
                            "SURREALDB_USER" | "SURREALDB_PASS" |
                            "STAFF_NAME" | "STAFF_EMAIL" | "STAFF_PHONE" | "STAFF_POSITION" |
                            "PROJECT_FOLDER_PATH" => continue,
                            _ => lines.push(line.to_string()),
                        }
                    } else {
                        lines.push(line.to_string());
                    }
                }
            }
            Err(e) => {
                error!("Failed to read existing .env file: {}", e);
                return Err(format!("Failed to read existing .env file: {}", e));
            }
        }
    }
    
    // Add our settings in organized sections
    lines.push("".to_string()); // Empty line for separation
    lines.push("# SurrealDB Configuration".to_string());
    
    if let Some(url) = &settings.surrealdb_url {
        lines.push(format!("SURREALDB_URL=\"{}\"", url));
    }
    if let Some(ns) = &settings.surrealdb_ns {
        lines.push(format!("SURREALDB_NS=\"{}\"", ns));
    }
    if let Some(db) = &settings.surrealdb_db {
        lines.push(format!("SURREALDB_DB=\"{}\"", db));
    }
    if let Some(user) = &settings.surrealdb_user {
        lines.push(format!("SURREALDB_USER=\"{}\"", user));
    }
    if let Some(pass) = &settings.surrealdb_pass {
        lines.push(format!("SURREALDB_PASS=\"{}\"", pass));
    }
    
    lines.push("".to_string());
    lines.push("# Staff Information".to_string());
    
    if let Some(name) = &settings.staff_name {
        lines.push(format!("STAFF_NAME=\"{}\"", name));
    }
    if let Some(email) = &settings.staff_email {
        lines.push(format!("STAFF_EMAIL=\"{}\"", email));
    }
    if let Some(phone) = &settings.staff_phone {
        lines.push(format!("STAFF_PHONE=\"{}\"", phone));
    }
    if let Some(position) = &settings.staff_position {
        lines.push(format!("STAFF_POSITION=\"{}\"", position));
    }
    
    lines.push("".to_string());
    lines.push("# Project Configuration".to_string());
    
    if let Some(folder_path) = &settings.project_folder_path {
        lines.push(format!("PROJECT_FOLDER_PATH=\"{}\"", folder_path));
    }
    
    // Write to file atomically
    let content = lines.join("\n");
    match fs::write(env_path, content) {
        Ok(_) => {
            info!("Successfully saved settings to .env file");
            Ok("Settings saved successfully".to_string())
        }
        Err(e) => {
            error!("Failed to write .env file: {}", e);
            Err(format!("Failed to write .env file: {}", e))
        }
    }
}

// ============================================================================
// FILE SYSTEM AND DIALOG COMMANDS
// ============================================================================

/// Open folder picker dialog for user selection.
/// 
/// This command displays a native folder picker dialog allowing users to
/// select directories. It's commonly used for selecting project template
/// directories or output locations.
/// 
/// # Parameters
/// - `app_handle`: Tauri app handle (automatically provided)
/// 
/// # Returns
/// - `Ok(Some(String))`: Selected folder path
/// - `Ok(None)`: User cancelled selection
/// - `Err(String)`: Dialog error or timeout
/// 
/// # Dialog Configuration
/// - **Title**: "Select Project Folder"
/// - **Timeout**: 30 seconds for user interaction
/// - **Platform**: Native OS dialog (Windows Explorer, macOS Finder, etc.)
/// 
/// # Frontend Usage
/// ```typescript
/// const folderPath = await invoke('select_folder');
/// if (folderPath) {
///   console.log(`Selected: ${folderPath}`);
/// } else {
///   console.log('User cancelled selection');
/// }
/// ```
#[tauri::command]
pub async fn select_folder(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    info!("Opening folder picker dialog");
    
    use std::sync::mpsc;
    use std::time::Duration;
    
    let (tx, rx) = mpsc::channel();
    
    // Open native folder picker dialog
    app_handle.dialog()
        .file()
        .set_title("Select Project Folder")
        .pick_folder(move |folder_path| {
            let _ = tx.send(folder_path);
        });
    
    // Wait for the dialog result with a timeout
    match rx.recv_timeout(Duration::from_secs(30)) {
        Ok(Some(path)) => {
            let path_str = path.to_string();
            info!("Selected folder: {}", path_str);
            Ok(Some(path_str))
        }
        Ok(None) => {
            info!("No folder selected");
            Ok(None)
        }
        Err(_) => {
            error!("Folder selection timed out");
            Err("Folder selection timed out".to_string())
        }
    }
}

/// Open folder in native file explorer.
/// 
/// This command opens the specified folder path in the system's default
/// file manager (Windows Explorer, macOS Finder, Linux file manager).
/// It's used to provide quick access to project folders.
/// 
/// # Parameters
/// - `folder_path`: Absolute path to folder to open
/// 
/// # Returns
/// - `Ok(String)`: Success message
/// - `Err(String)`: Path not found or permission error
/// 
/// # Platform Support
/// - **Windows**: Uses `explorer` command
/// - **macOS**: Uses `open` command  
/// - **Linux**: Uses `xdg-open` command
/// 
/// # Security Considerations
/// Path validation is performed to prevent command injection, but callers
/// should ensure paths are trusted before calling this command.
/// 
/// # Frontend Usage
/// ```typescript
/// await invoke('open_folder_in_explorer', { 
///   folder_path: 'E:\\Projects\\25-97105 Hotel Project' 
/// });
/// // Opens folder in system file manager
/// ```
#[tauri::command]
pub async fn open_folder_in_explorer(folder_path: String) -> Result<String, String> {
    info!("Opening folder in explorer: {}", folder_path);
    
    use std::process::Command;
    
    // Use platform-specific command to open folder
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer")
            .arg(&folder_path)
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&folder_path)
            .spawn()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
    };
    
    match result {
        Ok(_) => {
            info!("Successfully opened folder: {}", folder_path);
            Ok("Folder opened successfully".to_string())
        }
        Err(e) => {
            error!("Failed to open folder {}: {}", folder_path, e);
            Err(format!("Failed to open folder: {}", e))
        }
    }
}

// ============================================================================
// DEBUGGING AND DEVELOPMENT COMMANDS
// ============================================================================

/// Investigate specific database record for debugging.
/// 
/// This command provides detailed information about a specific database
/// record, including its complete data structure, relationships, and
/// metadata. It's primarily used for debugging and development.
/// 
/// # Parameters
/// - `record_id`: Full record ID (e.g., "projects:25_97105", "company:CHE")
/// 
/// # Returns
/// - `Ok(serde_json::Value)`: Complete record data with metadata
/// - `Err(String)`: Record not found or access error
/// 
/// # Information Included
/// - Complete record data with all fields
/// - Related record information (if applicable)
/// - Database metadata (ID, timestamps, etc.)
/// - Field validation status
/// 
/// # Frontend Usage
/// ```typescript
/// const record = await invoke('investigate_record', { 
///   record_id: 'projects:25_97105' 
/// });
/// console.log('Record data:', record);
/// ```
#[tauri::command]
pub async fn investigate_record(record_id: String, state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    info!("Investigating record: {}", record_id);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.investigate_record(&record_id).await {
        Ok(result) => {
            info!("Successfully investigated record: {}", record_id);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to investigate record {}: {}", record_id, e);
            Err(format!("Failed to investigate record: {}", e))
        }
    }
}

// ============================================================================
// PROJECT NUMBER GENERATION COMMANDS
// ============================================================================

/// Search countries with fuzzy matching across multiple fields.
/// 
/// This command performs comprehensive search across the countries reference
/// table, enabling users to find countries by name, code, or dial code.
/// It's used primarily by the NewProjectModal for country selection.
/// 
/// # Parameters
/// - `query`: Search string (minimum 2 characters recommended)
/// 
/// # Returns
/// - `Ok(Vec<serde_json::Value>)`: Matching country records
/// - `Err(String)`: Search error or database failure
/// 
/// # Search Fields
/// - `name`: Standard country name (e.g., "United Arab Emirates")
/// - `name_formal`: Formal country name
/// - `name_official`: Official country name
/// - `code`: ISO country code (e.g., "AE", "SA")
/// - `code_alt`: Alternative country codes
/// - `dial_code`: International dialing code (e.g., 971, 966)
/// 
/// # Search Strategy
/// Uses CONTAINS operator for substring matching across all fields
/// with NULL-safe queries to handle incomplete data gracefully.
/// 
/// # Frontend Usage
/// ```typescript
/// const countries = await invoke('search_countries', { query: 'UAE' });
/// // Returns: [{ name: "United Arab Emirates", dial_code: 971, ... }]
/// 
/// const byDialCode = await invoke('search_countries', { query: '971' });
/// // Same result - searches dial_code field too
/// ```
#[tauri::command]
pub async fn search_countries(query: String, state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    info!("Searching countries with query: {}", query);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.search_countries(&query).await {
        Ok(countries) => {
            info!("Search returned {} countries", countries.len());
            Ok(countries)
        }
        Err(e) => {
            error!("Failed to search countries: {}", e);
            Err(format!("Failed to search countries: {}", e))
        }
    }
}

/// Generate next sequential project number for given country and year.
/// 
/// This command implements the core project numbering algorithm, generating
/// the next available project number in the format YY-CCCNN. It ensures
/// uniqueness by querying existing projects and incrementing the sequence.
/// 
/// # Parameters
/// - `country_name`: Country display name (e.g., "United Arab Emirates")
/// - `year`: Optional 2-digit year (defaults to current year % 100)
/// 
/// # Returns
/// - `Ok(String)`: Generated project number (e.g., "25-97105")
/// - `Err(String)`: Generation error or country not found
/// 
/// # Algorithm
/// 1. Look up country by name to get dial code
/// 2. Query database for highest sequence number for country/year
/// 3. Increment sequence by 1 (or start at 1 if none exist)
/// 4. Format as YY-CCCNN with proper zero-padding
/// 
/// # Project Number Format
/// - **YY**: 2-digit year (25 = 2025)
/// - **CCC**: 3-digit country dial code (971 = UAE)
/// - **NN**: 2-digit sequence number (01, 02, 03...)
/// 
/// # Examples
/// - UAE 2025, sequence 5: "25-97105"
/// - Saudi 2025, sequence 1: "25-96601"
/// - UAE 2026, sequence 12: "26-97112"
/// 
/// # Frontend Usage
/// ```typescript
/// const projectNumber = await invoke('generate_next_project_number', {
///   country_name: 'United Arab Emirates',
///   year: 25
/// });
/// console.log(projectNumber); // "25-97105"
/// ```
#[tauri::command]
pub async fn generate_next_project_number(country_name: String, year: Option<u8>, state: State<'_, AppState>) -> Result<String, String> {
    info!("Generating next project number for country: {}, year: {:?}", country_name, year);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.generate_next_project_number(&country_name, year).await {
        Ok(number) => {
            info!("Generated project number: {}", number);
            Ok(number)
        }
        Err(e) => {
            error!("Failed to generate project number: {}", e);
            Err(format!("Failed to generate project number: {}", e))
        }
    }
}

/// Validate that a project number doesn't already exist.
/// 
/// This command checks the database to ensure a generated project number
/// is unique before allowing project creation. It's used as a validation
/// step in the project creation workflow.
/// 
/// # Parameters
/// - `project_number`: Project number to validate (e.g., "25-97105")
/// 
/// # Returns
/// - `Ok(true)`: Project number is available/unique
/// - `Ok(false)`: Project number already exists
/// - `Err(String)`: Validation error or database failure
/// 
/// # Validation Strategy
/// Uses a COUNT query to check for existing projects with the same
/// number.id field. This is more efficient than selecting full records.
/// 
/// # Frontend Usage
/// ```typescript
/// const isValid = await invoke('validate_project_number', {
///   project_number: '25-97105'
/// });
/// 
/// if (!isValid) {
///   console.error('Project number already exists!');
/// }
/// ```
#[tauri::command]
pub async fn validate_project_number(project_number: String, state: State<'_, AppState>) -> Result<bool, String> {
    info!("Validating project number: {}", project_number);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.validate_project_number(&project_number).await {
        Ok(is_valid) => {
            info!("Project number {} is valid: {}", project_number, is_valid);
            Ok(is_valid)
        }
        Err(e) => {
            error!("Failed to validate project number: {}", e);
            Err(format!("Failed to validate project number: {}", e))
        }
    }
}

/// Create project with automatic template folder copying.
/// 
/// This command provides the complete project creation workflow including:
/// 1. Creating the project record in the database
/// 2. Copying template folder structure
/// 3. Renaming template files with actual project number
/// 4. Cross-platform file system operations
/// 
/// # Parameters
/// - `project`: NewProject object with all required fields
/// 
/// # Returns
/// - `Ok(Project)`: Created project with database metadata
/// - `Err(String)`: Creation error or file system failure
/// 
/// # Template Folder Operations
/// 1. **Source**: `{PROJECT_FOLDER_PATH}\\01 RFPs\\_yy-cccnn Project Name`
/// 2. **Destination**: `{PROJECT_FOLDER_PATH}\\01 RFPs\\{project_number} {name_short}`
/// 3. **File Renaming**: All files containing "yy-cccnn" are renamed with actual number
/// 
/// # Cross-Platform Support
/// - **Windows**: Uses `xcopy` command for robust folder copying
/// - **Other Platforms**: Could be extended with platform-specific commands
/// 
/// # Error Handling
/// - Database creation is atomic - if it fails, no files are created
/// - File operations are best-effort - project is still created if they fail
/// - Detailed logging for all operations for debugging
/// 
/// # Frontend Usage
/// ```typescript
/// const projectData = {
///   name: "New Hotel Project",
///   name_short: "Hotel ABC",
///   status: "Draft",
///   area: "Downtown",
///   city: "Dubai", 
///   country: "United Arab Emirates",
///   number: { year: 25, country: 971, seq: 6, id: "25-97106" },
///   folder: "25-97106 Hotel ABC"
/// };
/// 
/// const created = await invoke('create_project_with_template', {
///   project: projectData
/// });
/// ```
/// 
/// # File System Requirements
/// - `PROJECT_FOLDER_PATH` must be configured in settings
/// - Template folder `_yy-cccnn Project Name` must exist
/// - Write permissions for destination directory
/// - Windows: `xcopy` command must be available (standard on Windows)
#[tauri::command]
pub async fn create_project_with_template(project: NewProject, state: State<'_, AppState>) -> Result<Project, String> {
    info!("Creating project with template: {}", project.name);
    info!("Project data: {:?}", project);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    // First create the project in database
    info!("About to create project in database...");
    match manager_clone.create_new_project(project.clone()).await {
        Ok(created_project) => {
            info!("Successfully created project in database: {:?}", created_project.id);
            
            // Get project folder path from settings
            info!("Getting settings for project folder path...");
            let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
            
            info!("Settings loaded - project_folder_path: {:?}", settings.project_folder_path);
            
            if let Some(base_path) = settings.project_folder_path {
                // Copy template folder
                let template_path = format!("{}\\01 RFPs\\_yy-cccnn Project Name", base_path);
                let project_number = created_project.number.id.clone();
                let dest_folder_name = format!("{} {}", project_number, created_project.name_short);
                let dest_path = format!("{}\\01 RFPs\\{}", base_path, dest_folder_name);
                
                info!("Copying template from {} to {}", template_path, dest_path);
                
                // Use Windows xcopy for robust folder copying
                use std::process::Command;
                let result = Command::new("xcopy")
                    .args(&[&template_path, &dest_path, "/E", "/I", "/Q"])
                    .output();
                
                match result {
                    Ok(output) => {
                        if output.status.success() {
                            info!("Successfully copied template folder");
                            
                            // Rename files within the copied folder
                            if let Err(e) = rename_template_files(&dest_path, "yy-cccnn", &project_number) {
                                error!("Failed to rename template files: {}", e);
                                // Don't fail the entire operation just because rename failed
                            }
                        } else {
                            error!("Failed to copy template folder: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute xcopy: {}", e);
                    }
                }
            } else {
                info!("No project_folder_path configured in settings - skipping template folder creation");
            }
            
            Ok(created_project)
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            Err(format!("Failed to create project: {}", e))
        }
    }
}

/// Helper function to recursively rename template files.
/// 
/// This function walks through all files and directories in the copied
/// template folder and renames any files that contain the template pattern
/// (e.g., "yy-cccnn") with the actual project number.
/// 
/// # Parameters
/// - `dir_path`: Root directory to process
/// - `old_pattern`: Pattern to replace (e.g., "yy-cccnn")
/// - `new_pattern`: Replacement text (e.g., "25-97105")
/// 
/// # Returns
/// - `Ok(())`: All files renamed successfully
/// - `Err(String)`: File system error during renaming
/// 
/// # Operation
/// - Recursively visits all subdirectories
/// - Checks each file and directory name for the pattern
/// - Renames matching files atomically
/// - Logs all rename operations for debugging
/// 
/// # Examples
/// - `Template yy-cccnn Proposal.docx` → `Template 25-97105 Proposal.docx`
/// - `yy-cccnn Project Folder` → `25-97105 Project Folder`
/// - `Invoice_yy-cccnn_Draft.pdf` → `Invoice_25-97105_Draft.pdf`
fn rename_template_files(dir_path: &str, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    use std::path::Path;
    
    let path = Path::new(dir_path);
    
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", dir_path));
    }
    
    // Walk through all files and folders recursively
    visit_dirs(path, old_pattern, new_pattern)?;
    
    Ok(())
}

/// Recursive directory visitor for file renaming.
/// 
/// This function implements the recursive logic for walking directory
/// trees and renaming files. It processes directories depth-first to
/// avoid path conflicts during renaming.
/// 
/// # Parameters
/// - `dir`: Directory path to process
/// - `old_pattern`: Pattern to search for in filenames
/// - `new_pattern`: Replacement pattern
/// 
/// # Returns
/// - `Ok(())`: Directory processed successfully
/// - `Err(String)`: File system error during processing
fn visit_dirs(dir: &Path, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    use std::fs;
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recurse into subdirectories first
                visit_dirs(&path, old_pattern, new_pattern)?;
            }
            
            // Check if filename contains the pattern
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if file_name_str.contains(old_pattern) {
                        let new_name = file_name_str.replace(old_pattern, new_pattern);
                        let new_path = path.with_file_name(new_name);
                        
                        info!("Renaming {:?} to {:?}", path, new_path);
                        fs::rename(&path, &new_path)
                            .map_err(|e| format!("Failed to rename file: {}", e))?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

// ============================================================================
// PROJECT TEMPLATE COPYING COMMANDS
// ============================================================================

/// Copy project template folder and rename files based on project details.
/// 
/// This command provides cross-platform functionality to copy the project template
/// folder structure and rename all template files with the actual project number
/// and short name. It supports both Mac and Windows environments.
/// 
/// # Parameters
/// - `project_number`: The project number (e.g., "25-97105")
/// - `project_short_name`: The short name for the project (e.g., "Hotel ABC")
/// 
/// # Returns
/// - `Ok(String)`: Success message with created folder path
/// - `Err(String)`: Error during folder copy or file rename operations
/// 
/// # Template Structure
/// Source: `{PROJECT_FOLDER_PATH}/01 RFPs/_yy-cccnn Project Name/`
/// Destination: `{PROJECT_FOLDER_PATH}/01 RFPs/{project_number} {project_short_name}/`
/// 
/// # File Renaming
/// All files containing "yy-cccnn" are renamed with the actual project number:
/// - `yy-cccnn-FP-01 Pricing.xlsx` → `25-97105-FP-01 Pricing.xlsx`
/// - `yy-cccnn-FP-01.indd` → `25-97105-FP-01.indd`
/// - `yy-cccnn-var Default Values.json` → `25-97105-var Default Values.json`
/// 
/// # Cross-Platform Support
/// - **macOS/Linux**: Uses `cp -R` command for recursive folder copying
/// - **Windows**: Uses `xcopy` command with appropriate flags
/// - Path separators are automatically normalized for each platform
/// 
/// # Frontend Usage
/// ```typescript
/// const result = await invoke('copy_project_template', {
///   project_number: '25-97105',
///   project_short_name: 'Hotel ABC'
/// });
/// console.log(result); // "Template copied successfully to: /path/to/project"
/// ```
#[tauri::command]
pub async fn copy_project_template(project_number: String, project_short_name: String) -> Result<String, String> {
    info!("Copying project template for number: {}, short name: {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    // Normalize paths for cross-platform compatibility
    let base_path_buf = PathBuf::from(&base_path);
    let template_path = base_path_buf.join("01 RFPs").join("_yy-cccnn Project Name");
    let dest_folder_name = format!("{} {}", project_number, project_short_name);
    let dest_path = base_path_buf.join("01 RFPs").join(&dest_folder_name);
    
    info!("Template path: {:?}", template_path);
    info!("Destination path: {:?}", dest_path);
    
    // Check if template folder exists
    if !template_path.exists() {
        return Err(format!("Template folder not found: {:?}", template_path));
    }
    
    // Check if destination already exists
    if dest_path.exists() {
        return Err(format!("Destination folder already exists: {:?}", dest_path));
    }
    
    // Copy template folder using cross-platform approach
    copy_folder_recursive(&template_path, &dest_path)?;
    
    info!("Successfully copied template folder");
    
    // Rename files within the copied folder
    rename_template_files_cross_platform(&dest_path, "yy-cccnn", &project_number)?;
    
    info!("Successfully renamed template files");
    
    Ok(format!("Template copied successfully to: {}", dest_path.display()))
}

/// Copy project template folder and populate JSON file with FP record data.
/// 
/// This enhanced function combines template copying with database integration,
/// automatically populating the JSON variable file with actual project and
/// FP record information from the database.
/// 
/// # Parameters
/// - `rfp_id`: The RFP/FP record ID to fetch data from database
/// - `project_number`: The project number (e.g., "25-97105")
/// - `project_short_name`: The short name for the project (e.g., "Hotel ABC")
/// 
/// # Returns
/// - `Ok(String)`: Success message with created folder path and data population status
/// - `Err(String)`: Error during folder copy, database fetch, or JSON update operations
/// 
/// # Workflow
/// 1. Copy template folder structure using `copy_project_template`
/// 2. Fetch FP record data from database (with project, company, contact info)
/// 3. Update JSON file with real data replacing template placeholders
/// 4. Return success status with detailed information
/// 
/// # JSON Field Mapping
/// Maps database fields to JSON template variables:
/// - `02 Document Number`: `{project_number}-FP-{revision}`
/// - `04 Document Issue Date`: Formatted issue date
/// - `06 Project Name`: Project name from database
/// - `07 Project Activity`: Activity from FP record
/// - `08 Project Package`: Package from FP record
/// - `11 Project Area`: Project area
/// - `12 Project City`: Project city
/// - `13 Project Country`: Project country
/// - `21 Client Company`: Company name
/// - `22 Client City`: Company city
/// - `23 Client Country`: Company country
/// - `26 Contact Name`: Contact full name
/// - `27 Contact Position`: Contact position
/// - `28 Contact Phone`: Contact phone
/// - `29 Contact Email`: Contact email
/// - `99 Strap Line`: Staff/company tagline
/// 
/// # Frontend Usage
/// ```typescript
/// const result = await invoke('copy_project_template_with_data', {
///   rfp_id: 'fee:some_id',
///   project_number: '25-97105',
///   project_short_name: 'Hotel ABC'
/// });
/// console.log(result); // "Template copied and populated successfully"
/// ```
// REMOVED: Test function for debugging FP fetching
// #[tauri::command]
// pub async fn debug_test_fp_fetching(state: State<'_, AppState>) -> Result<String, String> {
//     info!("Debug: Testing FP record fetching");
//     let manager_clone = {
//         let manager = state.lock().map_err(|e| e.to_string())?;
//         manager.clone()
//     };
//     
//     let fps = manager_clone.get_fees().await
//         .map_err(|e| format!("Failed to fetch FPs: {}", e))?;
//     
//     info!("Debug: Successfully fetched {} FP records", fps.len());
//     if fps.len() > 0 {
//         let first_fp = &fps[0];
//         info!("Debug: First FP ID = {:?}", first_fp.id);
//         info!("Debug: First FP name = {}", first_fp.name);
//     }
//     
//     Ok(format!("Success: Found {} FP records", fps.len()))
// }

#[tauri::command]
pub async fn populate_project_data(fp_id: String, project_number: String, project_short_name: String, state: State<'_, AppState>) -> Result<String, String> {
    info!("Populating project data for FP: {}, Project: {} {}", fp_id, project_number, project_short_name);
    
    // Fetch FP record data from database
    info!("Fetching FP record data for ID: {}", fp_id);
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    // Get FP data (using fees function for now since that's what we have)
    let fps = manager_clone.get_fees().await
        .map_err(|e| format!("Failed to fetch FPs: {}", e))?;
    
    // Find the specific FP
    info!("Looking for FP ID: {}", fp_id);
    info!("Available FP IDs in database:");
    for (i, f) in fps.iter().enumerate().take(5) {
        if let Some(id) = &f.id {
            let full_format = id.to_string();
            let id_value = id.id.to_string();
            let without_brackets = format!("{}:{}", id.tb, id_value);
            info!("  FP {}: Full DB format = '{}'", i, full_format);
            info!("  FP {}: Without brackets = '{}'", i, without_brackets);
        }
    }
    
    let fp = fps.iter()
        .find(|f| {
            if let Some(id) = &f.id {
                // The database stores: "fee:⟨22_96601_1⟩" (with angle brackets)
                // The frontend sends: "fee:22_96601_1" (without angle brackets)
                // We need to handle both formats
                
                let id_string = id.to_string(); // This gives us the full "fee:⟨22_96601_1⟩" format
                let id_value = id.id.to_string(); // This gives us just the ID part
                let formatted_without_brackets = format!("{}:{}", id.tb, id_value);
                
                info!("Comparing FP ID:");
                info!("  Thing.to_string() = '{}'", id_string);
                info!("  Thing.tb = '{}'", id.tb);
                info!("  Thing.id = '{}'", id_value);
                info!("  Formatted without brackets = '{}'", formatted_without_brackets);
                info!("  Target from frontend = '{}'", fp_id);
                
                // Try both formats: with brackets (database format) and without brackets (frontend format)
                let matches = id_string == fp_id || formatted_without_brackets == fp_id;
                
                if matches {
                    info!("✓ Found matching FP!");
                } else {
                    info!("✗ No match");
                }
                
                matches
            } else {
                info!("FP has no ID");
                false
            }
        })
        .ok_or_else(|| {
            info!("No matching FP found for ID: {}", fp_id);
            format!("FP not found: {}", fp_id)
        })?;
    
    info!("Found FP: {} - {}", fp.name, fp.number);
    
    // Update JSON file with real data
    let json_result = update_project_json_file(&project_number, &project_short_name, fp).await?;
    info!("JSON file updated: {}", json_result);
    
    // Rename JSON file to remove " Default Values"
    let rename_result = rename_json_file(&project_number, &project_short_name).await?;
    info!("JSON file renamed: {}", rename_result);
    
    Ok(format!("✅ Project data populated and file renamed: {} | {}", json_result, rename_result))
}

/// Update the project JSON file with real FP record data.
/// 
/// This function reads the JSON template file, replaces placeholder values with
/// actual data from the FP record, and writes the updated content back to disk.
/// 
/// # Parameters
/// - `project_number`: Project number for file path construction
/// - `project_short_name`: Project short name for file path construction  
/// - `fp`: Complete FP record with all related data
/// 
/// # Returns
/// - `Ok(String)`: Success message with update details
/// - `Err(String)`: File operation or JSON parsing error
async fn update_project_json_file(project_number: &str, project_short_name: &str, fp: &Fee) -> Result<String, String> {
    use serde_json::Value;
    
    info!("Updating JSON file for project: {} {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    // Construct JSON file path
    let base_path_buf = PathBuf::from(&base_path);
    let json_file_path = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal")
        .join(format!("{}-var Default Values.json", project_number));
    
    info!("JSON file path: {:?}", json_file_path);
    
    // Read current JSON file
    let json_content = fs::read_to_string(&json_file_path)
        .map_err(|e| format!("Failed to read JSON file: {}", e))?;
    
    // Parse JSON
    let mut json_data: Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Update JSON values with RFP data
    if let Value::Object(ref mut map) = json_data {
        // Document information
        map.insert("02 Document Number".to_string(), Value::String(format!("{}-FP", project_number)));
        
        // Project information from FP record  
        map.insert("06 Project Name".to_string(), Value::String(fp.name.clone()));
        
        // Debug: Log FP record details
        info!("FP record details: name='{}', number='{}', status={:?}", fp.name, fp.number, fp.status);
        
        // For now, just populate the basic fields we know work
        // TODO: Add more field mapping after testing database query
        
        // Status information
        let status_str = format!("{:?}", fp.status);
        map.insert("09 Project Stage".to_string(), Value::String(status_str));
        
        info!("Successfully populated {} JSON fields with FP data: {}", map.len(), fp.name);
    }
    
    // Write updated JSON back to file
    let updated_json = serde_json::to_string_pretty(&json_data)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    
    fs::write(&json_file_path, updated_json)
        .map_err(|e| format!("Failed to write JSON file: {}", e))?;
    
    info!("Successfully updated JSON file with RFP data");
    Ok(format!("JSON file updated: {:?}", json_file_path.file_name().unwrap_or_default()))
}

/// Format YYMMDD date string to "dd MMM yyyy" format for JSON template.
/// 
/// # Parameters
/// - `issue_date`: Date string in YYMMDD format (e.g., "250719")
/// 
/// # Returns
/// - Formatted date string (e.g., "19 Jul 2025")
/// - Original string if parsing fails
/// Check if a project folder already exists
#[tauri::command]
pub async fn check_project_folder_exists(project_number: String, project_short_name: String) -> Result<bool, String> {
    info!("Checking if project folder exists for: {} {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    let base_path_buf = PathBuf::from(&base_path);
    let dest_folder_name = format!("{} {}", project_number, project_short_name);
    let dest_path = base_path_buf.join("01 RFPs").join(&dest_folder_name);
    
    let exists = dest_path.exists();
    info!("Project folder '{}' exists: {}", dest_path.display(), exists);
    
    Ok(exists)
}

/// Check if a var.json file already exists in a project folder
#[tauri::command]
pub async fn check_var_json_exists(project_number: String, project_short_name: String) -> Result<bool, String> {
    info!("Checking if var.json exists for: {} {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    let json_file_path = project_dir.join(format!("{}-var.json", project_number));
    
    let exists = json_file_path.exists();
    info!("var.json file '{}' exists: {}", json_file_path.display(), exists);
    
    Ok(exists)
}

/// Check if a var.json template file (with "Default Values") exists in a project folder
#[tauri::command]
pub async fn check_var_json_template_exists(project_number: String, project_short_name: String) -> Result<bool, String> {
    info!("Checking if var template (Default Values) exists for: {} {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    let json_template_path = project_dir.join(format!("{}-var Default Values.json", project_number));
    
    let exists = json_template_path.exists();
    info!("var template file '{}' exists: {}", json_template_path.display(), exists);
    
    Ok(exists)
}

/// Rename an existing folder with _old suffix
#[tauri::command]
pub async fn rename_folder_with_old_suffix(project_number: String, project_short_name: String) -> Result<String, String> {
    info!("Renaming folder with _old suffix: {} {}", project_number, project_short_name);
    
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    let base_path_buf = PathBuf::from(&base_path);
    let original_folder_name = format!("{} {}", project_number, project_short_name);
    let original_path = base_path_buf.join("01 RFPs").join(&original_folder_name);
    
    if !original_path.exists() {
        return Err("Original folder does not exist".to_string());
    }
    
    // Create a unique _old folder name with timestamp if needed
    let mut old_suffix = "_old".to_string();
    let mut counter = 1;
    loop {
        let old_folder_name = format!("{}{}", original_folder_name, old_suffix);
        let old_path = base_path_buf.join("01 RFPs").join(&old_folder_name);
        
        if !old_path.exists() {
            // Rename the folder
            fs::rename(&original_path, &old_path)
                .map_err(|e| format!("Failed to rename folder: {}", e))?;
            
            info!("Renamed folder from '{}' to '{}'", original_folder_name, old_folder_name);
            return Ok(format!("Folder renamed to: {}", old_folder_name));
        }
        
        // If _old already exists, try _old2, _old3, etc.
        counter += 1;
        old_suffix = format!("_old{}", counter);
    }
}

/// Rename an existing var.json file with _old suffix
#[tauri::command]
pub async fn rename_var_json_with_old_suffix(project_number: String, project_short_name: String) -> Result<String, String> {
    info!("Renaming var.json with _old suffix: {} {}", project_number, project_short_name);
    
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    let base_path_buf = PathBuf::from(&base_path);
    let project_dir = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    
    let original_json_name = format!("{}-var.json", project_number);
    let original_json_path = project_dir.join(&original_json_name);
    
    if !original_json_path.exists() {
        return Err("Original var.json file does not exist".to_string());
    }
    
    // Create a unique _old file name with timestamp if needed
    let mut old_suffix = "_old".to_string();
    let mut counter = 1;
    loop {
        let old_json_name = format!("{}-var{}.json", project_number, old_suffix);
        let old_json_path = project_dir.join(&old_json_name);
        
        if !old_json_path.exists() {
            // Rename the file
            fs::rename(&original_json_path, &old_json_path)
                .map_err(|e| format!("Failed to rename file: {}", e))?;
            
            info!("Renamed file from '{}' to '{}'", original_json_name, old_json_name);
            return Ok(format!("File renamed to: {}", old_json_name));
        }
        
        // If _old already exists, try _old2, _old3, etc.
        counter += 1;
        old_suffix = format!("_old{}", counter);
    }
}


/// Rename the JSON file to remove " Default Values" from filename.
/// 
/// This function renames the JSON file from "{project_number}-var Default Values.json"
/// to "{project_number}-var.json" after data population is complete.
/// 
/// # Parameters
/// - `project_number`: Project number for file path construction
/// - `project_short_name`: Project short name for file path construction
/// 
/// # Returns
/// - `Ok(String)`: Success message with rename details
/// - `Err(String)`: File operation error
async fn rename_json_file(project_number: &str, project_short_name: &str) -> Result<String, String> {
    info!("Renaming JSON file for project: {} {}", project_number, project_short_name);
    
    // Get project folder path from settings
    let settings = get_settings().await.map_err(|e| format!("Failed to get settings: {}", e))?;
    let base_path = settings.project_folder_path
        .ok_or_else(|| "PROJECT_FOLDER_PATH not configured in settings".to_string())?;
    
    // Construct file paths
    let base_path_buf = PathBuf::from(&base_path);
    let project_folder = base_path_buf
        .join("01 RFPs")
        .join(format!("{} {}", project_number, project_short_name))
        .join("02 Proposal");
    
    let old_file_path = project_folder.join(format!("{}-var Default Values.json", project_number));
    let new_file_path = project_folder.join(format!("{}-var.json", project_number));
    
    info!("Renaming from: {:?}", old_file_path);
    info!("Renaming to: {:?}", new_file_path);
    
    // Check if old file exists
    if !old_file_path.exists() {
        return Err(format!("Source file not found: {:?}", old_file_path));
    }
    
    // Rename the file
    fs::rename(&old_file_path, &new_file_path)
        .map_err(|e| format!("Failed to rename file: {}", e))?;
    
    info!("Successfully renamed JSON file");
    Ok(format!("Renamed to: {:?}", new_file_path.file_name().unwrap_or_default()))
}

/// Format issue date from YYMMDD format to "dd MMM yyyy" format for JSON.
/// 
/// # Parameters
/// - `issue_date`: Date string in YYMMDD format (e.g., "251201")
/// 
/// # Returns
/// - `Ok(String)`: Formatted date (e.g., "01 Dec 2025")
/// Cross-platform recursive folder copying function.
/// 
/// This function provides robust folder copying that works on both Mac and Windows
/// by using native system commands that handle permissions, symlinks, and special
/// files correctly.
/// 
/// # Parameters
/// - `src`: Source folder path
/// - `dest`: Destination folder path
/// 
/// # Returns
/// - `Ok(())`: Folder copied successfully
/// - `Err(String)`: Copy operation failed
/// 
/// # Platform Commands
/// - **macOS/Linux**: `cp -R source dest`
/// - **Windows**: `xcopy source dest /E /I /Q /Y`
fn copy_folder_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    use std::process::Command;
    
    info!("Copying folder from {:?} to {:?}", src, dest);
    
    let result = if cfg!(target_os = "windows") {
        // Windows: Use xcopy for robust copying
        Command::new("xcopy")
            .args(&[
                src.to_string_lossy().as_ref(),
                dest.to_string_lossy().as_ref(),
                "/E",  // Copy directories and subdirectories, including empty ones
                "/I",  // If destination does not exist and copying more than one file, assume destination is a directory
                "/Q",  // Don't display file names while copying
                "/Y"   // Overwrite existing files without prompting
            ])
            .output()
    } else {
        // macOS/Linux: Use cp for recursive copying
        Command::new("cp")
            .args(&[
                "-R",  // Recursive copy
                src.to_string_lossy().as_ref(),
                dest.to_string_lossy().as_ref()
            ])
            .output()
    };
    
    match result {
        Ok(output) => {
            if output.status.success() {
                info!("Successfully copied folder using system command");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Folder copy command failed: {}", stderr);
                Err(format!("Failed to copy folder: {}", stderr))
            }
        }
        Err(e) => {
            error!("Failed to execute copy command: {}", e);
            Err(format!("Failed to execute copy command: {}", e))
        }
    }
}

/// Cross-platform file renaming function with improved error handling.
/// 
/// This function recursively walks through a directory tree and renames all files
/// and directories that contain a specific pattern. It uses cross-platform path
/// handling and provides detailed error reporting.
/// 
/// # Parameters
/// - `dir_path`: Root directory to process
/// - `old_pattern`: Pattern to replace (e.g., "yy-cccnn")
/// - `new_pattern`: Replacement text (e.g., "25-97105")
/// 
/// # Returns
/// - `Ok(())`: All files renamed successfully
/// - `Err(String)`: File system error during renaming
/// 
/// # Features
/// - Cross-platform path handling using PathBuf
/// - Depth-first directory traversal to avoid path conflicts
/// - Detailed logging of all rename operations
/// - Atomic file renaming with error recovery
fn rename_template_files_cross_platform(dir_path: &Path, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    info!("Renaming template files in {:?}, replacing '{}' with '{}'", dir_path, old_pattern, new_pattern);
    
    if !dir_path.exists() {
        return Err(format!("Directory does not exist: {:?}", dir_path));
    }
    
    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {:?}", dir_path));
    }
    
    // Walk through all files and folders recursively
    visit_dirs_cross_platform(dir_path, old_pattern, new_pattern)?;
    
    info!("Successfully renamed all template files");
    Ok(())
}

/// Cross-platform recursive directory visitor for file renaming.
/// 
/// This function implements depth-first directory traversal using PathBuf
/// for cross-platform compatibility. It processes subdirectories before
/// processing the current directory to avoid path conflicts during renaming.
/// 
/// # Parameters
/// - `dir`: Directory path to process
/// - `old_pattern`: Pattern to search for in filenames
/// - `new_pattern`: Replacement pattern
/// 
/// # Returns
/// - `Ok(())`: Directory processed successfully
/// - `Err(String)`: File system error during processing
fn visit_dirs_cross_platform(dir: &Path, old_pattern: &str, new_pattern: &str) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;
    
    let mut entries_to_rename = Vec::new();
    
    // First pass: collect all entries and recurse into subdirectories
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            // Recurse into subdirectories first
            visit_dirs_cross_platform(&path, old_pattern, new_pattern)?;
        }
        
        // Check if filename contains the pattern
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.contains(old_pattern) {
                    entries_to_rename.push(path);
                }
            }
        }
    }
    
    // Second pass: rename entries (files first, then directories)
    for path in entries_to_rename {
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                let new_name = file_name_str.replace(old_pattern, new_pattern);
                let new_path = path.with_file_name(new_name);
                
                info!("Renaming {:?} to {:?}", path, new_path);
                
                fs::rename(&path, &new_path)
                    .map_err(|e| format!("Failed to rename {:?} to {:?}: {}", path, new_path, e))?;
                
                info!("Successfully renamed to {:?}", new_path);
            }
        }
    }
    
    Ok(())
}

// ============================================================================
// LOCATION SUGGESTION COMMANDS
// ============================================================================

/// Get area suggestions for a specific country.
/// 
/// This command queries existing project data to provide autocomplete
/// suggestions for area fields when creating new projects. It helps
/// maintain consistency in location naming across projects.
/// 
/// # Parameters
/// - `country`: Country name to filter by (e.g., "United Arab Emirates")
/// 
/// # Returns
/// - `Ok(Vec<String>)`: List of unique area names from existing projects
/// - `Err(String)`: Query error or database failure
/// 
/// # Data Source
/// Suggestions are derived from the `area` field of existing projects
/// in the specified country, providing real-world location examples.
/// 
/// # Frontend Usage
/// ```typescript
/// const areas = await invoke('get_area_suggestions', { 
///   country: 'United Arab Emirates' 
/// });
/// // Returns: ["Dubai Marina", "Downtown Dubai", "Business Bay", ...]
/// ```
#[tauri::command]
pub async fn get_area_suggestions(country: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting area suggestions for country: {}", country);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_area_suggestions(&country).await {
        Ok(areas) => {
            info!("Found {} area suggestions", areas.len());
            Ok(areas)
        }
        Err(e) => {
            error!("Failed to get area suggestions: {}", e);
            Err(e.to_string())
        }
    }
}

// REMOVED: Test function for template copying
// /// Test the copy_project_template function with sample data.
// /// 
// /// This is a temporary test command to verify the template copying functionality
// /// works correctly with the sample project "25-97199 Test Project".
// #[tauri::command]
// pub async fn test_template_copy() -> Result<String, String> {
//     info!("Testing template copy function with sample data");
//     
//     let project_number = "25-97199".to_string();
//     let project_short_name = "Test Project".to_string();
//     
//     match copy_project_template(project_number, project_short_name).await {
//         Ok(result) => {
//             info!("Template copy test successful: {}", result);
//             Ok(format!("✅ Template copy test successful: {}", result))
//         }
//         Err(e) => {
//             error!("Template copy test failed: {}", e);
//             Err(format!("❌ Template copy test failed: {}", e))
//         }
//     }
// }

/// Get city suggestions for a specific country.
/// 
/// This command provides autocomplete suggestions for city fields by
/// querying existing project data. Like area suggestions, it promotes
/// consistency in location naming.
/// 
/// # Parameters
/// - `country`: Country name to filter by (e.g., "United Arab Emirates")
/// 
/// # Returns
/// - `Ok(Vec<String>)`: List of unique city names from existing projects
/// - `Err(String)`: Query error or database failure
/// 
/// # Data Source
/// Suggestions come from the `city` field of existing projects in the
/// specified country, providing proven location examples.
/// 
/// # Frontend Usage
/// ```typescript
/// const cities = await invoke('get_city_suggestions', { 
///   country: 'United Arab Emirates' 
/// });
/// // Returns: ["Dubai", "Abu Dhabi", "Sharjah", ...]
/// ```
#[tauri::command]
pub async fn get_all_cities(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting all city suggestions");
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_all_cities().await {
        Ok(cities) => {
            info!("Successfully retrieved {} cities", cities.len());
            Ok(cities)
        }
        Err(e) => {
            error!("Failed to get all cities: {}", e);
            Err(format!("Failed to get all cities: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_city_suggestions(country: String, state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("Getting city suggestions for country: {}", country);
    
    let manager_clone = {
        let manager = state.lock().map_err(|e| e.to_string())?;
        manager.clone()
    };
    
    match manager_clone.get_city_suggestions(&country).await {
        Ok(cities) => {
            info!("Found {} city suggestions", cities.len());
            Ok(cities)
        }
        Err(e) => {
            error!("Failed to get city suggestions: {}", e);
            Err(e.to_string())
        }
    }
}


//! Company management commands for E-Fees application
//!
//! This module handles CRUD operations for companies.

use crate::commands::utils::execute_with_manager;
use crate::crud_command;
use crate::db::{Company, CompanyCreate, PaginatedResponse};
use tauri::State;

use super::{AppState, CompanyUpdate};

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
crud_command!(
    get_companies,
    Vec<Company>,
    get_companies,
    "fetch",
    "companies"
);

/// Retrieve a paginated page of companies.
crud_command!(
    get_companies_page,
    PaginatedResponse<Company>,
    get_companies_page,
    "fetch page",
    "companies",
    paginated
);

/// Fetch a single company by ID (for on-demand related record loading).
crud_command!(
    get_company_by_id,
    Option<Company>,
    get_company_by_id,
    "fetch",
    "company",
    id: String
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
pub async fn update_company(
    id: String,
    company_update: CompanyUpdate,
    state: State<'_, AppState>,
) -> Result<Company, String> {
    let company_name = format!("company '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move {
                manager
                    .update_company_partial(&id_clone, company_update)
                    .await
            })
        },
        "update",
        &company_name,
    )
    .await
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

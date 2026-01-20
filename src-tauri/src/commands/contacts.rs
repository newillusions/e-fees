//! Contact management commands for E-Fees application
//!
//! This module handles CRUD operations for contacts.

use crate::crud_command;
use crate::db::{Contact, ContactCreate, PaginatedResponse};
use crate::commands::utils::execute_with_manager;
use tauri::State;

use super::{AppState, ContactUpdate};

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
crud_command!(
    get_contacts,
    Vec<Contact>,
    get_contacts,
    "fetch",
    "contacts"
);

/// Retrieve a paginated page of contacts.
crud_command!(
    get_contacts_page,
    PaginatedResponse<Contact>,
    get_contacts_page,
    "fetch page",
    "contacts",
    paginated
);

/// Fetch a single contact by ID.
crud_command!(
    get_contact_by_id,
    Option<Contact>,
    get_contact_by_id,
    "fetch",
    "contact",
    id: String
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
#[tauri::command]
pub async fn delete_contact(id: String, state: State<'_, AppState>) -> Result<Contact, String> {
    let contact_name = format!("contact '{}'", id);
    execute_with_manager(
        &state,
        |manager| {
            let id_clone = id.clone();
            Box::pin(async move {
                manager.delete_contact(&id_clone).await
            })
        },
        "delete",
        &contact_name
    ).await
}

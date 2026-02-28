//! Contact route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Contact};

use crate::error::ApiError;
use crate::AppState;

/// List all contacts.
///
/// Returns `{ "data": [...], "count": N }` with contact fields.
/// The `company` RecordId field is returned as `company_id` string.
pub async fn list_contacts(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let contacts: Vec<Contact> = state.db.select("contacts").await?;
    let count = contacts.len();

    let data: Vec<Value> = contacts.iter().map(contact_to_json).collect();

    Ok(Json(json!({
        "data": data,
        "count": count
    })))
}

/// Get a single contact by ID.
///
/// Returns 404 if not found.
pub async fn get_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("contacts:").unwrap_or(&id);

    let contact: Option<Contact> = state.db.select(("contacts", key)).await?;

    match contact {
        Some(c) => Ok(Json(json!({ "data": contact_to_json(&c) }))),
        None => Err(ApiError::not_found("Contact", &id)),
    }
}

/// Convert a Contact to a JSON value for API response.
fn contact_to_json(c: &Contact) -> Value {
    json!({
        "id": c.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "first_name": c.first_name,
        "last_name": c.last_name,
        "full_name": c.full_name,
        "email": c.email,
        "phone": c.phone,
        "position": c.position,
        "company_id": c.company.as_ref().map(|id| record_id_string(id)),
    })
}

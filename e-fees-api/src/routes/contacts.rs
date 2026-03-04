//! Contact route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Contact, ContactCreate};

use crate::error::ApiError;
use crate::pagination::{db_paginate_filtered, paginated_json_raw, FilterClause};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id};
use crate::AppState;

/// Combined query parameters for contact listing (pagination + filters).
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ContactListParams {
    /// Page number (1-indexed, default: 1).
    pub page: Option<u64>,
    /// Items per page (default: 50, max: 100).
    pub page_size: Option<u64>,
    /// Filter by company record key (e.g. "acme_corp"). Strips "company:" prefix if present.
    pub company: Option<String>,
}

/// List contacts with pagination and optional company filter.
#[utoipa::path(
    get,
    path = "/contacts",
    tag = "Contacts",
    params(ContactListParams),
    responses(
        (status = 200, description = "Paginated list of contacts", body = schemas::PaginatedResponse<schemas::ContactResponse>),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_contacts(
    State(state): State<Arc<AppState>>,
    params: Query<ContactListParams>,
) -> Result<Json<Value>, ApiError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50);

    let filter = if let Some(ref company) = params.company {
        let key = company.strip_prefix("company:").unwrap_or(company);
        Some(FilterClause {
            clause: "company = type::record('company', $filter_company)".into(),
            binds: vec![("filter_company".into(), json!(key))],
        })
    } else {
        None
    };

    let (contacts, total): (Vec<Contact>, u64) =
        db_paginate_filtered(&state.db, "contacts", page, page_size, filter).await?;

    let data: Vec<Value> = contacts.iter().map(contact_to_json).collect();

    Ok(Json(paginated_json_raw(data, total, page, page_size)))
}

/// Get a single contact by ID.
#[utoipa::path(
    get,
    path = "/contacts/{id}",
    tag = "Contacts",
    params(("id" = String, Path, description = "Contact record key (contacts: prefix stripped if present)")),
    responses(
        (status = 200, description = "Contact found", body = schemas::SingleResponse<schemas::ContactResponse>),
        (status = 404, description = "Contact not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
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

/// Create a new contact.
#[utoipa::path(
    post,
    path = "/contacts",
    tag = "Contacts",
    request_body = ContactCreate,
    responses(
        (status = 200, description = "Contact created", body = schemas::SingleResponse<schemas::ContactResponse>),
        (status = 400, description = "Validation error", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_contact(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ContactCreate>,
) -> Result<Json<Value>, ApiError> {
    require_non_empty(&body.first_name, "first_name")?;
    require_non_empty(&body.last_name, "last_name")?;

    let full_name = format!("{} {}", body.first_name, body.last_name);

    let mut response = state
        .db
        .query("CREATE contacts SET first_name = $first_name, last_name = $last_name, \
                full_name = $full_name, email = $email, phone = $phone, position = $position, \
                company = type::record('company', $company), \
                time = { created_at: time::now(), updated_at: time::now() }")
        .bind(("first_name", body.first_name))
        .bind(("last_name", body.last_name))
        .bind(("full_name", full_name))
        .bind(("email", body.email))
        .bind(("phone", body.phone))
        .bind(("position", body.position))
        .bind(("company", body.company))
        .await?;
    let created: Option<Contact> = response.take(0)?;

    match created {
        Some(c) => Ok(Json(json!({ "data": contact_to_json(&c) }))),
        None => Err(ApiError::bad_request("Failed to create contact")),
    }
}

/// Update an existing contact (merge).
#[utoipa::path(
    put,
    path = "/contacts/{id}",
    tag = "Contacts",
    params(("id" = String, Path, description = "Contact record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Contact updated", body = schemas::SingleResponse<schemas::ContactResponse>),
        (status = 404, description = "Contact not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("contacts:").unwrap_or(&id);
    validate_id(key)?;

    let query = format!(
        "UPDATE contacts:{key} MERGE $data RETURN NONE; \
         UPDATE contacts:{key} SET time.updated_at = time::now() RETURN NONE; \
         SELECT * FROM contacts:{key};",
        key = key
    );
    let mut response = state.db.query(&query).bind(("data", body)).await?;
    let updated: Option<Contact> = response.take(2)?;

    match updated {
        Some(c) => Ok(Json(json!({ "data": contact_to_json(&c) }))),
        None => Err(ApiError::not_found("Contact", &id)),
    }
}

/// Delete a contact by ID.
#[utoipa::path(
    delete,
    path = "/contacts/{id}",
    tag = "Contacts",
    params(("id" = String, Path, description = "Contact record key")),
    responses(
        (status = 200, description = "Contact deleted", body = schemas::DeleteResponse),
        (status = 404, description = "Contact not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_contact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("contacts:").unwrap_or(&id);
    validate_id(key)?;

    let deleted: Option<Contact> = state.db.delete(("contacts", key)).await?;

    match deleted {
        Some(_) => Ok(Json(json!({ "deleted": true, "id": format!("contacts:{}", key) }))),
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

//! Company route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Company};

use crate::error::ApiError;
use crate::AppState;

/// List all companies.
///
/// Returns `{ "data": [...], "count": N }` with core company fields.
pub async fn list_companies(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let companies: Vec<Company> = state.db.select("company").await?;
    let count = companies.len();

    let data: Vec<Value> = companies.iter().map(company_to_json).collect();

    Ok(Json(json!({
        "data": data,
        "count": count
    })))
}

/// Get a single company by ID.
///
/// Path parameter is the record key.
/// Returns 404 if not found.
pub async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let company: Option<Company> = state.db.select(("company", &*id)).await?;

    match company {
        Some(c) => Ok(Json(json!({ "data": company_to_json(&c) }))),
        None => Err(ApiError::not_found("Company", &id)),
    }
}

/// Convert a Company to a JSON value for API response.
fn company_to_json(c: &Company) -> Value {
    json!({
        "id": c.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "name": c.name,
        "name_short": c.name_short,
        "abbreviation": c.abbreviation,
        "city": c.city,
        "country": c.country,
    })
}

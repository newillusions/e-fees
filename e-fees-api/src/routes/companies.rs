//! Company route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Company, CompanyCreate};

use crate::error::ApiError;
use crate::pagination::{db_paginate, paginated_json, PaginationParams};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id};
use crate::AppState;

/// List companies with pagination.
#[utoipa::path(
    get,
    path = "/companies",
    tag = "Companies",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of companies", body = schemas::PaginatedResponse<schemas::CompanyResponse>),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_companies(
    State(state): State<Arc<AppState>>,
    params: Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let (companies, total): (Vec<Company>, u64) =
        db_paginate(&state.db, "company", &params).await?;

    let data: Vec<Value> = companies.iter().map(company_to_json).collect();

    Ok(Json(paginated_json(data, total, &params)))
}

/// Get a single company by ID.
#[utoipa::path(
    get,
    path = "/companies/{id}",
    tag = "Companies",
    params(("id" = String, Path, description = "Company record key")),
    responses(
        (status = 200, description = "Company found", body = schemas::SingleResponse<schemas::CompanyResponse>),
        (status = 404, description = "Company not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
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

/// Create a new company.
#[utoipa::path(
    post,
    path = "/companies",
    tag = "Companies",
    request_body = CompanyCreate,
    responses(
        (status = 200, description = "Company created", body = schemas::SingleResponse<schemas::CompanyResponse>),
        (status = 400, description = "Validation error", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_company(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CompanyCreate>,
) -> Result<Json<Value>, ApiError> {
    require_non_empty(&body.name, "name")?;

    let created: Option<Company> = state.db.create("company").content(body).await?;

    match created {
        Some(c) => Ok(Json(json!({ "data": company_to_json(&c) }))),
        None => Err(ApiError::bad_request("Failed to create company")),
    }
}

/// Update an existing company (merge).
#[utoipa::path(
    put,
    path = "/companies/{id}",
    tag = "Companies",
    params(("id" = String, Path, description = "Company record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Company updated", body = schemas::SingleResponse<schemas::CompanyResponse>),
        (status = 404, description = "Company not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    let updated: Option<Company> = state.db.update(("company", &*id)).merge(body).await?;

    match updated {
        Some(c) => Ok(Json(json!({ "data": company_to_json(&c) }))),
        None => Err(ApiError::not_found("Company", &id)),
    }
}

/// Delete a company by ID.
#[utoipa::path(
    delete,
    path = "/companies/{id}",
    tag = "Companies",
    params(("id" = String, Path, description = "Company record key")),
    responses(
        (status = 200, description = "Company deleted", body = schemas::DeleteResponse),
        (status = 404, description = "Company not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_company(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    let deleted: Option<Company> = state.db.delete(("company", &*id)).await?;

    match deleted {
        Some(_) => Ok(Json(json!({ "deleted": true, "id": format!("company:{}", id) }))),
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

//! Company route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Company, CompanyCreate};

use crate::error::ApiError;
use crate::pagination::{db_paginate_filtered, paginated_json_raw, FilterClause};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id};
use crate::AppState;

/// Combined query parameters for company listing (pagination + filters).
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct CompanyListParams {
    /// Page number (1-indexed, default: 1).
    pub page: Option<u64>,
    /// Items per page (default: 50, max: 100).
    pub page_size: Option<u64>,
    /// Filter by company name (case-insensitive substring match).
    pub name: Option<String>,
}

/// List companies with pagination and optional name filter.
#[utoipa::path(
    get,
    path = "/companies",
    tag = "Companies",
    params(CompanyListParams),
    responses(
        (status = 200, description = "Paginated list of companies", body = schemas::PaginatedResponse<schemas::CompanyResponse>),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_companies(
    State(state): State<Arc<AppState>>,
    params: Query<CompanyListParams>,
) -> Result<Json<Value>, ApiError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50);

    let filter = if let Some(ref name) = params.name {
        Some(FilterClause {
            clause: "string::lowercase(name) CONTAINS string::lowercase($filter_name)".into(),
            binds: vec![("filter_name".into(), json!(name))],
        })
    } else {
        None
    };

    let (companies, total): (Vec<Company>, u64) =
        db_paginate_filtered(&state.db, "company", page, page_size, filter).await?;

    let data: Vec<Value> = companies.iter().map(company_to_json).collect();

    Ok(Json(paginated_json_raw(data, total, page, page_size)))
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

    let mut response = state
        .db
        .query("CREATE company SET name = $name, name_short = $name_short, abbreviation = $abbreviation, city = $city, country = $country, reg_no = $reg_no, tax_no = $tax_no, time = { created_at: time::now(), updated_at: time::now() }")
        .bind(("name", body.name))
        .bind(("name_short", body.name_short))
        .bind(("abbreviation", body.abbreviation))
        .bind(("city", body.city))
        .bind(("country", body.country))
        .bind(("reg_no", body.reg_no))
        .bind(("tax_no", body.tax_no))
        .await?;
    let created: Option<Company> = response.take(0)?;

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

    let query = format!(
        "UPDATE company:{id} MERGE $data RETURN NONE; \
         UPDATE company:{id} SET time.updated_at = time::now() RETURN NONE; \
         SELECT * FROM company:{id};",
        id = id
    );
    let mut response = state.db.query(&query).bind(("data", body)).await?;
    let updated: Option<Company> = response.take(2)?;

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

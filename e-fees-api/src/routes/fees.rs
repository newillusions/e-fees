//! Fee route handlers.
//!
//! The SurrealDB table is named "fee" (renamed from "rfp" in v0.13.0).
//! Path parameters may include the "fee:" prefix which is stripped.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Fee, FeeCreate};

use crate::error::ApiError;
use crate::pagination::{db_paginate, paginated_json, PaginationParams};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id, validate_status, FEE_STATUSES};
use crate::AppState;

/// List fees with pagination.
#[utoipa::path(
    get,
    path = "/fees",
    tag = "Fees",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of fees", body = schemas::PaginatedResponse<schemas::FeeSummaryResponse>),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_fees(
    State(state): State<Arc<AppState>>,
    params: Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let (fees, total): (Vec<Fee>, u64) = db_paginate(&state.db, "fee", &params).await?;

    let data: Vec<Value> = fees.iter().map(fee_to_summary_json).collect();

    Ok(Json(paginated_json(data, total, &params)))
}

/// Get a single fee by ID.
#[utoipa::path(
    get,
    path = "/fees/{id}",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key (fee: prefix stripped if present)")),
    responses(
        (status = 200, description = "Fee found", body = schemas::SingleResponse<schemas::FeeDetailResponse>),
        (status = 404, description = "Fee not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn get_fee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);

    let fee: Option<Fee> = state.db.select(("fee", key)).await?;

    match fee {
        Some(f) => Ok(Json(json!({ "data": fee_to_detail_json(&f) }))),
        None => Err(ApiError::not_found("Fee", &id)),
    }
}

/// Create a new fee.
#[utoipa::path(
    post,
    path = "/fees",
    tag = "Fees",
    request_body = FeeCreate,
    responses(
        (status = 200, description = "Fee created", body = schemas::SingleResponse<schemas::FeeDetailResponse>),
        (status = 400, description = "Validation error", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_fee(
    State(state): State<Arc<AppState>>,
    Json(body): Json<FeeCreate>,
) -> Result<Json<Value>, ApiError> {
    require_non_empty(&body.name, "name")?;
    validate_status(&body.status, FEE_STATUSES, "fee")?;

    let created: Option<Fee> = state.db.create("fee").content(body).await?;

    match created {
        Some(f) => Ok(Json(json!({ "data": fee_to_detail_json(&f) }))),
        None => Err(ApiError::bad_request("Failed to create fee")),
    }
}

/// Update an existing fee (merge).
#[utoipa::path(
    put,
    path = "/fees/{id}",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Fee updated", body = schemas::SingleResponse<schemas::FeeDetailResponse>),
        (status = 404, description = "Fee not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_fee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    if let Some(status) = body.get("status").and_then(|s| s.as_str()) {
        validate_status(status, FEE_STATUSES, "fee")?;
    }

    let updated: Option<Fee> = state.db.update(("fee", key)).merge(body).await?;

    match updated {
        Some(f) => Ok(Json(json!({ "data": fee_to_detail_json(&f) }))),
        None => Err(ApiError::not_found("Fee", &id)),
    }
}

/// Delete a fee by ID.
#[utoipa::path(
    delete,
    path = "/fees/{id}",
    tag = "Fees",
    params(("id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Fee deleted", body = schemas::DeleteResponse),
        (status = 404, description = "Fee not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_fee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("fee:").unwrap_or(&id);
    validate_id(key)?;

    let deleted: Option<Fee> = state.db.delete(("fee", key)).await?;

    match deleted {
        Some(_) => Ok(Json(json!({ "deleted": true, "id": format!("fee:{}", key) }))),
        None => Err(ApiError::not_found("Fee", &id)),
    }
}

/// Convert a Fee to summary JSON (used in list view).
fn fee_to_summary_json(f: &Fee) -> Value {
    let (total_fee, currency) = extract_pricing(f);

    json!({
        "id": f.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "name": f.name,
        "number": f.number,
        "rev": f.rev,
        "status": f.status,
        "project_id": record_id_string(&f.project_id),
        "company_id": record_id_string(&f.company_id),
        "total_fee": total_fee,
        "currency": currency,
    })
}

/// Convert a Fee to detail JSON (used in single-item view).
fn fee_to_detail_json(f: &Fee) -> Value {
    let (total_fee, currency) = extract_pricing(f);

    json!({
        "id": f.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "name": f.name,
        "number": f.number,
        "rev": f.rev,
        "status": f.status,
        "issue_date": f.issue_date,
        "activity": f.activity,
        "package": f.package,
        "project_id": record_id_string(&f.project_id),
        "company_id": record_id_string(&f.company_id),
        "contact_id": record_id_string(&f.contact_id),
        "staff_name": f.staff_name,
        "staff_email": f.staff_email,
        "staff_phone": f.staff_phone,
        "staff_position": f.staff_position,
        "strap_line": f.strap_line,
        "total_fee": total_fee,
        "currency": currency,
    })
}

/// Extract total fee and currency from pricing breakdown.
/// Returns (0.0, "AED") if pricing is absent.
fn extract_pricing(f: &Fee) -> (f64, String) {
    match f.pricing_typed() {
        Some(p) => (p.config.quoted_fee, if p.config.currency.is_empty() { "AED".into() } else { p.config.currency }),
        None => (0.0, "AED".into()),
    }
}

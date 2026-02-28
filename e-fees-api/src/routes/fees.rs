//! Fee route handlers.
//!
//! The SurrealDB table is named "rfp" (not "fee").
//! Path parameters may include the "rfp:" prefix which is stripped.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Fee};

use crate::error::ApiError;
use crate::AppState;

/// List all fees.
///
/// Returns `{ "data": [...], "count": N }` with summary fields
/// including pricing total and currency extracted from the pricing breakdown.
pub async fn list_fees(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let fees: Vec<Fee> = state.db.select("rfp").await?;
    let count = fees.len();

    let data: Vec<Value> = fees.iter().map(fee_to_summary_json).collect();

    Ok(Json(json!({
        "data": data,
        "count": count
    })))
}

/// Get a single fee by ID.
///
/// Path parameter is the record key (e.g. "abc123").
/// The "rfp:" prefix is stripped if present.
/// Returns 404 if not found.
pub async fn get_fee(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Strip "rfp:" prefix if present
    let key = id.strip_prefix("rfp:").unwrap_or(&id);

    let fee: Option<Fee> = state.db.select(("rfp", key)).await?;

    match fee {
        Some(f) => Ok(Json(json!({ "data": fee_to_detail_json(&f) }))),
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

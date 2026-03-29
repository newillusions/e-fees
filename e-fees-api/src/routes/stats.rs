//! Statistics route handler.

use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{Company, Contact, Fee, Project};

use crate::error::ApiError;
use crate::schemas;
use crate::AppState;

/// Active fee statuses — fees in these states count as "active".
const ACTIVE_FEE_STATUSES: &[&str] = &["Draft", "Sent", "Negotiation"];

/// Get entity counts for dashboard statistics.
#[utoipa::path(
    get,
    path = "/stats",
    tag = "Statistics",
    responses(
        (status = 200, description = "Entity counts", body = schemas::StatsResponse),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn get_stats(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let projects: Vec<Project> = state.db.select("projects").await?;
    let companies: Vec<Company> = state.db.select("company").await?;
    let contacts: Vec<Contact> = state.db.select("contacts").await?;
    let fees: Vec<Fee> = state.db.select("fee").await?;

    let active_fees = fees
        .iter()
        .filter(|f| ACTIVE_FEE_STATUSES.contains(&f.status.as_str()))
        .count();

    Ok(Json(json!({
        "total_projects": projects.len(),
        "total_companies": companies.len(),
        "total_contacts": contacts.len(),
        "total_fees": fees.len(),
        "active_fees": active_fees,
    })))
}

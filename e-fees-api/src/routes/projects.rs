//! Project route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, Project};

use crate::error::ApiError;
use crate::AppState;

/// List all projects.
///
/// Returns `{ "data": [...], "count": N }` with each project containing
/// id, name, name_short, status, country, city, area, folder, and number.
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let projects: Vec<Project> = state.db.select("projects").await?;
    let count = projects.len();

    let data: Vec<Value> = projects.iter().map(project_to_json).collect();

    Ok(Json(json!({
        "data": data,
        "count": count
    })))
}

/// Get a single project by ID.
///
/// Path parameter is the record key (e.g. "25-97105").
/// Returns 404 if not found.
pub async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project: Option<Project> = state.db.select(("projects", &*id)).await?;

    match project {
        Some(p) => Ok(Json(json!({ "data": project_to_json(&p) }))),
        None => Err(ApiError::not_found("Project", &id)),
    }
}

/// Convert a Project to a JSON value for API response.
fn project_to_json(p: &Project) -> Value {
    json!({
        "id": p.id.as_ref().map(|id| record_id_string(id)).unwrap_or_default(),
        "name": p.name,
        "name_short": p.name_short,
        "status": p.status,
        "country": p.country,
        "city": p.city,
        "area": p.area,
        "folder": p.folder,
        "number": p.number.id,
    })
}

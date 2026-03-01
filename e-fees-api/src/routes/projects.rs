//! Project route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{record_id_string, NewProject, Project};

use crate::error::ApiError;
use crate::pagination::{db_paginate, paginated_json, PaginationParams};
use crate::schemas;
use crate::validation::{require_non_empty, validate_id, validate_status, PROJECT_STATUSES};
use crate::AppState;

/// List projects with pagination.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "Projects",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of projects", body = schemas::PaginatedResponse<schemas::ProjectResponse>),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_projects(
    State(state): State<Arc<AppState>>,
    params: Query<PaginationParams>,
) -> Result<Json<Value>, ApiError> {
    let (projects, total): (Vec<Project>, u64) =
        db_paginate(&state.db, "projects", &params).await?;

    let data: Vec<Value> = projects.iter().map(project_to_json).collect();

    Ok(Json(paginated_json(data, total, &params)))
}

/// Get a single project by ID.
#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key (e.g. 25_97105)")),
    responses(
        (status = 200, description = "Project found", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
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

/// Create a new project.
#[utoipa::path(
    post,
    path = "/projects",
    tag = "Projects",
    request_body = NewProject,
    responses(
        (status = 200, description = "Project created", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 400, description = "Validation error", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewProject>,
) -> Result<Json<Value>, ApiError> {
    require_non_empty(&body.name, "name")?;
    validate_status(&body.status, PROJECT_STATUSES, "project")?;

    let record_key = body.number.id.replace('-', "_");

    let query = format!(
        "CREATE projects:{key} SET name = $name, name_short = $name_short, \
         status = $status, area = $area, city = $city, country = $country, \
         folder = $folder, number = $number, \
         time = {{ created_at: time::now(), updated_at: time::now() }}",
        key = record_key
    );
    let mut response = state
        .db
        .query(&query)
        .bind(("name", body.name))
        .bind(("name_short", body.name_short))
        .bind(("status", body.status))
        .bind(("area", body.area))
        .bind(("city", body.city))
        .bind(("country", body.country))
        .bind(("folder", body.folder))
        .bind(("number", serde_json::json!({
            "year": body.number.year,
            "country": body.number.country,
            "seq": body.number.seq,
            "id": body.number.id,
        })))
        .await?;
    let created: Option<Project> = response.take(0)?;

    match created {
        Some(p) => Ok(Json(json!({ "data": project_to_json(&p) }))),
        None => Err(ApiError::bad_request("Failed to create project")),
    }
}

/// Update an existing project (merge).
#[utoipa::path(
    put,
    path = "/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Project updated", body = schemas::SingleResponse<schemas::ProjectResponse>),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    if let Some(status) = body.get("status").and_then(|s| s.as_str()) {
        validate_status(status, PROJECT_STATUSES, "project")?;
    }

    let query = format!(
        "UPDATE projects:{id} MERGE $data RETURN NONE; \
         UPDATE projects:{id} SET time.updated_at = time::now() RETURN NONE; \
         SELECT * FROM projects:{id};",
        id = id
    );
    let mut response = state.db.query(&query).bind(("data", body)).await?;
    let updated: Option<Project> = response.take(2)?;

    match updated {
        Some(p) => Ok(Json(json!({ "data": project_to_json(&p) }))),
        None => Err(ApiError::not_found("Project", &id)),
    }
}

/// Delete a project by ID.
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "Projects",
    params(("id" = String, Path, description = "Project record key")),
    responses(
        (status = 200, description = "Project deleted", body = schemas::DeleteResponse),
        (status = 404, description = "Project not found", body = schemas::ErrorResponse),
    ),
    security(("api_key" = []))
)]
pub async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_id(&id)?;

    let deleted: Option<Project> = state.db.delete(("projects", &*id)).await?;

    match deleted {
        Some(_) => Ok(Json(json!({ "deleted": true, "id": format!("projects:{}", id) }))),
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

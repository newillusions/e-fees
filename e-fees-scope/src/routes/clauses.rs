use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::AppState;

#[utoipa::path(get, path = "/clauses", tag = "Clauses",
    responses((status = 200, description = "List clauses")),
    security(("api_key" = [])))]
pub async fn list_clauses(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": [], "total": 0 })))
}

#[utoipa::path(get, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Get clause")),
    security(("api_key" = [])))]
pub async fn get_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Err(ApiError::not_found("Clause", "stub"))
}

#[utoipa::path(post, path = "/clauses", tag = "Clauses",
    responses((status = 201, description = "Create clause")),
    security(("api_key" = [])))]
pub async fn create_clause(
    State(_state): State<Arc<AppState>>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": {} })))
}

#[utoipa::path(put, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Update clause")),
    security(("api_key" = [])))]
pub async fn update_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    Json(_body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": {} })))
}

#[utoipa::path(delete, path = "/clauses/{id}", tag = "Clauses",
    params(("id" = String, Path, description = "Clause ID")),
    responses((status = 200, description = "Archive clause")),
    security(("api_key" = [])))]
pub async fn delete_clause(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "deleted": true })))
}

#[utoipa::path(get, path = "/clauses/categories", tag = "Clauses",
    responses((status = 200, description = "List categories")),
    security(("api_key" = [])))]
pub async fn list_categories(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(json!({ "data": [] })))
}

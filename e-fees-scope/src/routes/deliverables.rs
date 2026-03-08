//! Deliverable library CRUD route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use e_fees_core::models::{dbvalue_to_json, json_to_dbvalue, record_key_string};

use crate::error::ApiError;
use crate::models::{Deliverable, NewDeliverable, UpdateDeliverable};
use crate::AppState;

/// Query parameters for listing deliverables.
#[derive(Debug, Deserialize)]
pub struct DeliverableListParams {
    /// Filter by stage (exact match).
    pub stage: Option<String>,
    /// Filter by layer (exact match).
    pub layer: Option<String>,
    /// Filter by discipline (exact match).
    pub discipline: Option<String>,
    /// Filter by status (default: "active").
    pub status: Option<String>,
    /// Fuzzy text search across title, body, short_name.
    pub search: Option<String>,
}

/// Convert a Deliverable (SurrealDB native types) to a plain JSON Value.
pub fn deliverable_to_json(d: &Deliverable) -> Value {
    let mut obj = json!({
        "id": record_key_string(&d.id.key),
        "title": d.title,
        "short_name": d.short_name,
        "body": d.body,
        "stage": d.stage,
        "layer": d.layer,
        "sort_order": d.sort_order,
        "status": d.status,
        "version": d.version,
        "created_at": d.created_at.to_string(),
        "updated_at": d.updated_at.to_string(),
    });

    if let Some(ref disc) = d.discipline {
        obj["discipline"] = json!(disc);
    }

    if let Some(ref cond) = d.condition {
        obj["condition"] = dbvalue_to_json(cond);
    }

    if let Some(ref replaces) = d.replaces {
        obj["replaces"] = json!(record_key_string(&replaces.key));
    }

    if let Some(ref source) = d.source_proposals {
        obj["source_proposals"] = json!(source);
    }

    if let Some(ref history) = d.usage_history {
        obj["usage_history"] = dbvalue_to_json(history);
    }

    if let Some(ref tags) = d.tags {
        obj["tags"] = json!(tags);
    }

    obj
}

/// List deliverables with optional filters.
#[utoipa::path(
    get,
    path = "/deliverables",
    tag = "Deliverables",
    params(
        ("stage" = Option<String>, Query, description = "Filter by stage"),
        ("layer" = Option<String>, Query, description = "Filter by layer"),
        ("discipline" = Option<String>, Query, description = "Filter by discipline"),
        ("status" = Option<String>, Query, description = "Filter by status (default: active)"),
        ("search" = Option<String>, Query, description = "Fuzzy text search across title, body, short_name"),
    ),
    responses(
        (status = 200, description = "List of deliverables"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_deliverables(
    State(state): State<Arc<AppState>>,
    params: Query<DeliverableListParams>,
) -> Result<Json<Value>, ApiError> {
    let status = params.status.clone().unwrap_or_else(|| "active".to_string());

    let mut where_clauses = vec!["status = $status".to_string()];
    let mut binds: Vec<(&str, Value)> = vec![("status", json!(status))];

    if let Some(ref stage) = params.stage {
        where_clauses.push("stage = $stage".to_string());
        binds.push(("stage", json!(stage)));
    }

    if let Some(ref layer) = params.layer {
        where_clauses.push("layer = $layer".to_string());
        binds.push(("layer", json!(layer)));
    }

    if let Some(ref discipline) = params.discipline {
        where_clauses.push("discipline = $discipline".to_string());
        binds.push(("discipline", json!(discipline)));
    }

    if let Some(ref search) = params.search {
        where_clauses.push(
            "(string::lowercase(title) CONTAINS string::lowercase($search) \
             OR string::lowercase(body) CONTAINS string::lowercase($search) \
             OR string::lowercase(short_name) CONTAINS string::lowercase($search))"
                .to_string(),
        );
        binds.push(("search", json!(search)));
    }

    let where_str = where_clauses.join(" AND ");
    let query = format!(
        "SELECT * FROM deliverable WHERE {} ORDER BY stage ASC, sort_order ASC",
        where_str
    );

    let mut q = state.db.query(&query);
    for (k, v) in &binds {
        q = q.bind((*k, v.clone()));
    }

    let mut response = q.await?;
    let deliverables: Vec<Deliverable> = response.take(0)?;
    let total = deliverables.len();

    let data: Vec<Value> = deliverables.iter().map(deliverable_to_json).collect();

    Ok(Json(json!({ "data": data, "total": total })))
}

/// Get a single deliverable by ID.
#[utoipa::path(
    get,
    path = "/deliverables/{id}",
    tag = "Deliverables",
    params(("id" = String, Path, description = "Deliverable record key")),
    responses(
        (status = 200, description = "Deliverable found"),
        (status = 404, description = "Deliverable not found"),
    ),
    security(("api_key" = []))
)]
pub async fn get_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("deliverable:").unwrap_or(&id);

    let deliverable: Option<Deliverable> = state.db.select(("deliverable", key)).await?;

    match deliverable {
        Some(d) => Ok(Json(json!({ "data": deliverable_to_json(&d) }))),
        None => Err(ApiError::not_found("Deliverable", &id)),
    }
}

/// Create a new deliverable.
#[utoipa::path(
    post,
    path = "/deliverables",
    tag = "Deliverables",
    request_body = Value,
    responses(
        (status = 201, description = "Deliverable created"),
        (status = 400, description = "Validation error"),
    ),
    security(("api_key" = []))
)]
pub async fn create_deliverable(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NewDeliverable>,
) -> Result<Json<Value>, ApiError> {
    // Validate required fields
    if body.title.trim().is_empty() {
        return Err(ApiError::bad_request("title must not be empty"));
    }
    if body.short_name.trim().is_empty() {
        return Err(ApiError::bad_request("short_name must not be empty"));
    }
    if body.body.trim().is_empty() {
        return Err(ApiError::bad_request("body must not be empty"));
    }
    if body.stage.trim().is_empty() {
        return Err(ApiError::bad_request("stage must not be empty"));
    }
    if body.layer.trim().is_empty() {
        return Err(ApiError::bad_request("layer must not be empty"));
    }

    // Validate layer-specific requirements
    if body.layer == "discipline" && body.discipline.is_none() {
        return Err(ApiError::bad_request(
            "discipline field is required when layer is 'discipline'",
        ));
    }
    if body.layer == "conditional" && body.condition.is_none() {
        return Err(ApiError::bad_request(
            "condition field is required when layer is 'conditional'",
        ));
    }

    // Build optional SET fragments — never bind NULL for option<T> fields
    let mut optional_sets = String::new();
    if body.discipline.is_some() {
        optional_sets.push_str(", discipline = $discipline");
    }
    if body.condition.is_some() {
        optional_sets.push_str(", condition = $condition");
    }
    if body.replaces.is_some() {
        optional_sets.push_str(", replaces = type::record('deliverable', $replaces)");
    }
    if body.source_proposals.is_some() {
        optional_sets.push_str(", source_proposals = $source_proposals");
    }
    if body.tags.is_some() {
        optional_sets.push_str(", tags = $tags");
    }

    let query = format!(
        "CREATE deliverable SET \
         title = $title, \
         short_name = $short_name, \
         body = $body_text, \
         stage = $stage, \
         layer = $layer, \
         sort_order = $sort_order, \
         status = 'active', \
         version = 1, \
         created_at = time::now(), \
         updated_at = time::now(){optional_sets};"
    );

    let mut q = state
        .db
        .query(&query)
        .bind(("title", body.title.clone()))
        .bind(("short_name", body.short_name.clone()))
        .bind(("body_text", body.body.clone()))
        .bind(("stage", body.stage.clone()))
        .bind(("layer", body.layer.clone()))
        .bind(("sort_order", body.sort_order));

    if let Some(ref disc) = body.discipline {
        q = q.bind(("discipline", disc.clone()));
    }
    if let Some(ref cond) = body.condition {
        q = q.bind(("condition", json_to_dbvalue(cond)));
    }
    if let Some(ref replaces) = body.replaces {
        q = q.bind(("replaces", replaces.clone()));
    }
    if let Some(ref source) = body.source_proposals {
        q = q.bind(("source_proposals", source.clone()));
    }
    if let Some(ref tags) = body.tags {
        q = q.bind(("tags", tags.clone()));
    }

    let mut response = q.await?;
    let deliverables: Vec<Deliverable> = response.take(0)?;

    match deliverables.into_iter().next() {
        Some(d) => Ok(Json(json!({ "data": deliverable_to_json(&d) }))),
        None => Err(ApiError::internal(
            "Failed to create deliverable — no record returned",
        )),
    }
}

/// Update an existing deliverable (partial update).
#[utoipa::path(
    put,
    path = "/deliverables/{id}",
    tag = "Deliverables",
    params(("id" = String, Path, description = "Deliverable record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Deliverable updated"),
        (status = 404, description = "Deliverable not found"),
    ),
    security(("api_key" = []))
)]
pub async fn update_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateDeliverable>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("deliverable:").unwrap_or(&id);

    // Build dynamic SET clauses for provided fields
    let mut sets: Vec<String> = Vec::new();

    if let Some(ref title) = body.title {
        if title.trim().is_empty() {
            return Err(ApiError::bad_request("title must not be empty"));
        }
        sets.push("title = $title".into());
    }
    if let Some(ref short_name) = body.short_name {
        if short_name.trim().is_empty() {
            return Err(ApiError::bad_request("short_name must not be empty"));
        }
        sets.push("short_name = $short_name".into());
    }
    if let Some(ref body_text) = body.body {
        if body_text.trim().is_empty() {
            return Err(ApiError::bad_request("body must not be empty"));
        }
        sets.push("body = $body_text".into());
    }
    if body.stage.is_some() {
        sets.push("stage = $stage".into());
    }
    if body.layer.is_some() {
        sets.push("layer = $layer".into());
    }
    if body.discipline.is_some() {
        sets.push("discipline = $discipline".into());
    }
    if body.condition.is_some() {
        sets.push("condition = $condition".into());
    }
    if body.replaces.is_some() {
        sets.push("replaces = type::record('deliverable', $replaces)".into());
    }
    if body.sort_order.is_some() {
        sets.push("sort_order = $sort_order".into());
    }
    if body.source_proposals.is_some() {
        sets.push("source_proposals = $source_proposals".into());
    }
    if body.tags.is_some() {
        sets.push("tags = $tags".into());
    }

    // Always bump version and updated_at
    sets.push("version = version + 1".into());
    sets.push("updated_at = time::now()".into());

    let set_str = sets.join(", ");
    let query = format!(
        "UPDATE deliverable:{key} SET {set_str}; SELECT * FROM deliverable:{key};",
        key = key,
        set_str = set_str,
    );

    // Bind each field individually
    let mut q = state.db.query(&query);
    if let Some(ref title) = body.title {
        q = q.bind(("title", title.clone()));
    }
    if let Some(ref short_name) = body.short_name {
        q = q.bind(("short_name", short_name.clone()));
    }
    if let Some(ref body_text) = body.body {
        q = q.bind(("body_text", body_text.clone()));
    }
    if let Some(ref stage) = body.stage {
        q = q.bind(("stage", stage.clone()));
    }
    if let Some(ref layer) = body.layer {
        q = q.bind(("layer", layer.clone()));
    }
    if let Some(ref disc) = body.discipline {
        q = q.bind(("discipline", disc.clone()));
    }
    if let Some(ref cond) = body.condition {
        q = q.bind(("condition", json_to_dbvalue(cond)));
    }
    if let Some(ref replaces) = body.replaces {
        q = q.bind(("replaces", replaces.clone()));
    }
    if let Some(sort) = body.sort_order {
        q = q.bind(("sort_order", sort));
    }
    if let Some(ref source) = body.source_proposals {
        q = q.bind(("source_proposals", source.clone()));
    }
    if let Some(ref tags) = body.tags {
        q = q.bind(("tags", tags.clone()));
    }

    let mut response = q.await?;
    // Statement 0 is UPDATE, statement 1 is SELECT
    let deliverables: Vec<Deliverable> = response.take(1)?;

    match deliverables.into_iter().next() {
        Some(d) => Ok(Json(json!({ "data": deliverable_to_json(&d) }))),
        None => Err(ApiError::not_found("Deliverable", &id)),
    }
}

/// Soft-delete (archive) a deliverable.
#[utoipa::path(
    delete,
    path = "/deliverables/{id}",
    tag = "Deliverables",
    params(("id" = String, Path, description = "Deliverable record key")),
    responses(
        (status = 200, description = "Deliverable archived"),
        (status = 404, description = "Deliverable not found"),
    ),
    security(("api_key" = []))
)]
pub async fn delete_deliverable(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("deliverable:").unwrap_or(&id);

    let query = format!(
        "UPDATE deliverable:{key} SET status = 'archived', updated_at = time::now(); \
         SELECT * FROM deliverable:{key};",
        key = key,
    );

    let mut response = state.db.query(&query).await?;
    let deliverables: Vec<Deliverable> = response.take(1)?;

    match deliverables.into_iter().next() {
        Some(d) => Ok(Json(json!({ "data": deliverable_to_json(&d) }))),
        None => Err(ApiError::not_found("Deliverable", &id)),
    }
}

/// Get aggregated analytics for deliverables.
#[utoipa::path(
    get,
    path = "/deliverables/analytics",
    tag = "Deliverables",
    responses(
        (status = 200, description = "Deliverable analytics by stage, layer, discipline"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn deliverable_analytics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let query = "SELECT stage, layer, discipline, count() AS count FROM deliverable \
                 WHERE status = 'active' GROUP BY stage, layer, discipline";

    let mut response = state.db.query(query).await?;
    let rows: Vec<Value> = response.take(0)?;

    Ok(Json(json!({ "data": rows })))
}

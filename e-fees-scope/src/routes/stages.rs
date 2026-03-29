//! Stage configuration route handlers.

use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::record_key_string;

use crate::error::ApiError;
use crate::models::{StageConfig, UpdateStageConfig};
use crate::AppState;

/// Convert a StageConfig (SurrealDB native types) to a plain JSON Value.
fn stage_to_json(s: &StageConfig) -> Value {
    let mut obj = json!({
        "id": record_key_string(&s.id.key),
        "canonical_name": s.canonical_name,
        "default_label": s.default_label,
        "sort_order": s.sort_order,
        "status": s.status,
    });

    if let Some(ref aliases) = s.aliases {
        obj["aliases"] = json!(aliases);
    }

    if let Some(ref intro) = s.intro_text {
        obj["intro_text"] = json!(intro);
    }

    obj
}

/// List all active stage configurations ordered by sort_order.
#[utoipa::path(
    get,
    path = "/stages",
    tag = "Stages",
    responses(
        (status = 200, description = "List of stage configurations"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_stages(State(state): State<Arc<AppState>>) -> Result<Json<Value>, ApiError> {
    let query = "SELECT * FROM stage_config WHERE status = 'active' ORDER BY sort_order ASC";

    let mut response = state.db.query(query).await?;
    let stages: Vec<StageConfig> = response.take(0)?;
    let total = stages.len();

    let data: Vec<Value> = stages.iter().map(stage_to_json).collect();

    Ok(Json(json!({ "data": data, "total": total })))
}

/// Update a stage configuration by canonical_name (partial update).
#[utoipa::path(
    put,
    path = "/stages/{canonical_name}",
    tag = "Stages",
    params(("canonical_name" = String, Path, description = "Stage canonical name")),
    request_body = Value,
    responses(
        (status = 200, description = "Stage config updated"),
        (status = 404, description = "Stage config not found"),
    ),
    security(("api_key" = []))
)]
pub async fn update_stage(
    State(state): State<Arc<AppState>>,
    Path(canonical_name): Path<String>,
    Json(body): Json<UpdateStageConfig>,
) -> Result<Json<Value>, ApiError> {
    // Find the stage by canonical_name
    let mut find_response = state
        .db
        .query("SELECT * FROM stage_config WHERE canonical_name = $name LIMIT 1")
        .bind(("name", canonical_name.clone()))
        .await?;
    let existing: Vec<StageConfig> = find_response.take(0)?;

    let stage = match existing.into_iter().next() {
        Some(s) => s,
        None => return Err(ApiError::not_found("StageConfig", &canonical_name)),
    };

    let key = record_key_string(&stage.id.key);

    // Build dynamic SET clauses for provided fields
    let mut sets: Vec<String> = Vec::new();

    if let Some(ref label) = body.default_label {
        if label.trim().is_empty() {
            return Err(ApiError::bad_request("default_label must not be empty"));
        }
        sets.push("default_label = $default_label".into());
    }
    if body.aliases.is_some() {
        sets.push("aliases = $aliases".into());
    }
    if body.sort_order.is_some() {
        sets.push("sort_order = $sort_order".into());
    }
    if body.intro_text.is_some() {
        sets.push("intro_text = $intro_text".into());
    }

    if sets.is_empty() {
        // Nothing to update — return current state
        return Ok(Json(json!({ "data": stage_to_json(&stage) })));
    }

    let set_str = sets.join(", ");
    let query = format!(
        "UPDATE stage_config:`{key}` SET {set_str}; SELECT * FROM stage_config:`{key}`;",
        key = key,
        set_str = set_str,
    );

    let mut q = state.db.query(&query);
    if let Some(ref label) = body.default_label {
        q = q.bind(("default_label", label.clone()));
    }
    if let Some(ref aliases) = body.aliases {
        q = q.bind(("aliases", aliases.clone()));
    }
    if let Some(sort) = body.sort_order {
        q = q.bind(("sort_order", sort));
    }
    if let Some(ref intro) = body.intro_text {
        q = q.bind(("intro_text", intro.clone()));
    }

    let mut response = q.await?;
    // Statement 0 is UPDATE, statement 1 is SELECT
    let stages: Vec<StageConfig> = response.take(1)?;

    match stages.into_iter().next() {
        Some(s) => Ok(Json(json!({ "data": stage_to_json(&s) }))),
        None => Err(ApiError::not_found("StageConfig", &canonical_name)),
    }
}

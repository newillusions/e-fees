//! Assembly engine — auto-populate deliverables for a fee.

use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{dbvalue_to_json, record_key_string};

use crate::error::ApiError;
use crate::models::{
    AssembleRequest, Deliverable, SaveScopeBuilderRequest, StageConfig,
};
use crate::AppState;

/// Helper to strip "fee:" prefix from a fee ID string.
fn strip_fee_prefix(fee_id: &str) -> &str {
    fee_id.strip_prefix("fee:").unwrap_or(fee_id)
}

/// Check if a deliverable's condition object is satisfied by the request conditions.
///
/// For each key in the deliverable's condition, the request conditions must
/// contain the same key with the same value (subset matching).
fn condition_matches(
    deliverable_condition: &surrealdb_types::Value,
    request_conditions: &Option<Value>,
) -> bool {
    let req_conds = match request_conditions {
        Some(v) => v,
        None => return false, // no request conditions → exclude all conditionals
    };

    let req_obj = match req_conds.as_object() {
        Some(o) => o,
        None => return false,
    };

    // Convert the SurrealDB Value to JSON for comparison
    let del_json = dbvalue_to_json(deliverable_condition);
    let del_obj = match del_json.as_object() {
        Some(o) => o,
        None => return false,
    };

    // Every key in the deliverable condition must match the request
    for (key, val) in del_obj {
        match req_obj.get(key) {
            Some(req_val) if req_val == val => continue,
            _ => return false,
        }
    }

    true
}

/// Assemble deliverables for a fee, grouped by stage with layered composition.
#[utoipa::path(
    post,
    path = "/scope/assemble",
    tag = "Assembly",
    request_body = Value,
    responses(
        (status = 200, description = "Assembled deliverables grouped by stage"),
        (status = 400, description = "Invalid request"),
    ),
    security(("api_key" = []))
)]
pub async fn assemble_deliverables(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AssembleRequest>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&body.fee_id);
    if fee_key.is_empty() {
        return Err(ApiError::bad_request("fee_id must not be empty"));
    }
    if body.disciplines.is_empty() {
        return Err(ApiError::bad_request(
            "disciplines must contain at least one entry",
        ));
    }

    // 1. Fetch stage configs
    let stages: Vec<StageConfig> = if let Some(ref stage_names) = body.stages {
        if stage_names.is_empty() {
            // Empty list → all active stages
            let mut res = state
                .db
                .query(
                    "SELECT * FROM stage_config WHERE status = 'active' ORDER BY sort_order ASC",
                )
                .await?;
            res.take(0)?
        } else {
            let mut res = state
                .db
                .query(
                    "SELECT * FROM stage_config WHERE canonical_name IN $stages \
                     AND status = 'active' ORDER BY sort_order ASC",
                )
                .bind(("stages", stage_names.clone()))
                .await?;
            res.take(0)?
        }
    } else {
        let mut res = state
            .db
            .query("SELECT * FROM stage_config WHERE status = 'active' ORDER BY sort_order ASC")
            .await?;
        res.take(0)?
    };

    if stages.is_empty() {
        return Err(ApiError::bad_request(
            "No active stage configurations found",
        ));
    }

    // 2. For each stage, query deliverables with layered filtering
    let mut result_stages: Vec<Value> = Vec::new();

    for stage in &stages {
        let mut res = state
            .db
            .query(
                "SELECT * FROM deliverable \
                 WHERE stage = $stage AND status = 'active' \
                   AND (layer = 'generic' \
                     OR (layer = 'discipline' AND discipline IN $disciplines) \
                     OR layer = 'conditional') \
                 ORDER BY sort_order ASC",
            )
            .bind(("stage", stage.canonical_name.clone()))
            .bind(("disciplines", body.disciplines.clone()))
            .await?;

        let deliverables: Vec<Deliverable> = res.take(0)?;

        // 3. Filter conditionals
        let filtered: Vec<&Deliverable> = deliverables
            .iter()
            .filter(|d| {
                if d.layer == "conditional" {
                    match &d.condition {
                        Some(cond) => condition_matches(cond, &body.conditions),
                        None => false, // conditional without condition field → exclude
                    }
                } else {
                    true
                }
            })
            .collect();

        // 4. Dedup: collect replaced IDs, then filter out replaced deliverables
        let replaced_ids: HashSet<String> = filtered
            .iter()
            .filter_map(|d| d.replaces.as_ref())
            .map(|rid| record_key_string(&rid.key))
            .collect();

        let deduped: Vec<&Deliverable> = filtered
            .into_iter()
            .filter(|d| {
                let did = record_key_string(&d.id.key);
                !replaced_ids.contains(&did)
            })
            .collect();

        // 5. Build stage output
        let deliverable_items: Vec<Value> = deduped
            .iter()
            .map(|d| {
                let mut obj = json!({
                    "id": record_key_string(&d.id.key),
                    "short_name": d.short_name,
                    "title": d.title,
                    "body": d.body,
                    "layer": d.layer,
                    "sort_order": d.sort_order,
                });
                if let Some(ref disc) = d.discipline {
                    obj["discipline"] = json!(disc);
                }
                obj
            })
            .collect();

        // Apply stage_labels override if provided
        let label = if let Some(ref labels) = body.stage_labels {
            labels
                .get(&stage.canonical_name)
                .and_then(|v| v.as_str())
                .unwrap_or(&stage.default_label)
                .to_string()
        } else {
            stage.default_label.clone()
        };

        let mut stage_obj = json!({
            "canonical_name": stage.canonical_name,
            "label": label,
            "deliverables": deliverable_items,
        });

        if let Some(ref intro) = stage.intro_text {
            stage_obj["intro_text"] = json!(intro);
        } else {
            stage_obj["intro_text"] = Value::Null;
        }

        result_stages.push(stage_obj);
    }

    Ok(Json(json!({ "stages": result_stages })))
}

/// Save scope builder state for a fee.
#[utoipa::path(
    post,
    path = "/scope/save",
    tag = "Assembly",
    request_body = Value,
    responses(
        (status = 200, description = "Scope builder state saved"),
        (status = 400, description = "Invalid request"),
    ),
    security(("api_key" = []))
)]
pub async fn save_scope_builder(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveScopeBuilderRequest>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&body.fee_id);
    if fee_key.is_empty() {
        return Err(ApiError::bad_request("fee_id must not be empty"));
    }

    // Build deliverables_used array by fetching master body for each entry
    let mut deliverables_used: Vec<Value> = Vec::new();

    for entry in &body.deliverables {
        let del_key = entry
            .deliverable_id
            .strip_prefix("deliverable:")
            .unwrap_or(&entry.deliverable_id);

        // Fetch the master deliverable for its body text
        let del: Option<Deliverable> = state.db.select(("deliverable", del_key)).await?;

        let wording_snapshot = if let Some(ref override_text) = entry.wording_override {
            override_text.clone()
        } else {
            match &del {
                Some(d) => d.body.clone(),
                None => {
                    return Err(ApiError::not_found("Deliverable", &entry.deliverable_id));
                }
            }
        };

        let short_name = match &del {
            Some(d) => d.short_name.clone(),
            None => {
                return Err(ApiError::not_found("Deliverable", &entry.deliverable_id));
            }
        };

        deliverables_used.push(json!({
            "deliverable_id": del_key,
            "stage": entry.stage,
            "short_name": short_name,
            "sort_order": entry.sort_order,
            "wording_snapshot": wording_snapshot,
        }));
    }

    // Build optional SET fragments
    let mut optional_sets = String::new();
    if body.stage_labels.is_some() {
        optional_sets.push_str(", stage_labels = $stage_labels");
    }
    if body.manual_items.is_some() {
        optional_sets.push_str(", manual_items = $manual_items");
    }

    // Upsert scope_assembly
    let upsert_query = format!(
        "DELETE scope_assembly WHERE fee_id = type::record('fee', $fee_key); \
         CREATE scope_assembly SET \
         fee_id = type::record('fee', $fee_key), \
         deliverables_used = $deliverables_used, \
         updated_at = time::now(), \
         created_at = time::now(){optional_sets};"
    );

    let deliverables_json: Value = json!(deliverables_used);

    let mut q = state
        .db
        .query(&upsert_query)
        .bind(("fee_key", fee_key.to_string()))
        .bind(("deliverables_used", deliverables_json));

    if let Some(ref labels) = body.stage_labels {
        q = q.bind(("stage_labels", json!(labels)));
    }
    if let Some(ref manual) = body.manual_items {
        q = q.bind(("manual_items", json!(manual)));
    }

    q.await?;

    // Stamp usage_history on each deliverable used
    for entry in &body.deliverables {
        let del_key = entry
            .deliverable_id
            .strip_prefix("deliverable:")
            .unwrap_or(&entry.deliverable_id);

        let usage_entry = json!({
            "fee_id": fee_key,
            "date": chrono::Utc::now().to_rfc3339(),
        });

        let _ = state
            .db
            .query(
                "UPDATE type::record('deliverable', $did) SET \
                 usage_history = array::union(usage_history OR [], [$entry])",
            )
            .bind(("did", del_key.to_string()))
            .bind(("entry", usage_entry))
            .await;
    }

    Ok(Json(json!({
        "status": "saved",
        "fee_id": fee_key,
        "deliverables_count": body.deliverables.len(),
    })))
}

/// Retrieve saved scope builder state for a fee.
#[utoipa::path(
    get,
    path = "/scope/{fee_id}/deliverables",
    tag = "Assembly",
    params(("fee_id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Scope builder state"),
        (status = 404, description = "No saved assembly for this fee"),
    ),
    security(("api_key" = []))
)]
pub async fn get_scope_deliverables(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    let mut res = state
        .db
        .query(
            "SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key) LIMIT 1",
        )
        .bind(("fee_key", fee_key.to_string()))
        .await?;

    let rows: Vec<Value> = res.take(0)?;

    match rows.into_iter().next() {
        Some(row) => {
            let deliverables_used = row.get("deliverables_used").cloned().unwrap_or(json!([]));
            let stage_labels = row.get("stage_labels").cloned().unwrap_or(json!(null));
            let manual_items = row.get("manual_items").cloned().unwrap_or(json!([]));

            Ok(Json(json!({
                "fee_id": fee_key,
                "deliverables_used": deliverables_used,
                "stage_labels": stage_labels,
                "manual_items": manual_items,
            })))
        }
        None => Ok(Json(json!({
            "fee_id": fee_key,
            "deliverables_used": [],
            "stage_labels": null,
            "manual_items": [],
        }))),
    }
}

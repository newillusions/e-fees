//! Assembly engine — auto-populate deliverables for a fee.

use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::{dbvalue_to_json, record_key_string};

use crate::error::ApiError;
use crate::models::{AssembleRequest, Deliverable, SaveScopeBuilderRequest, StageConfig};
use crate::scope_store::{self, BuilderState};
use crate::AppState;

/// One saved-scope row resolved against the library.
#[derive(Debug, PartialEq)]
pub(crate) struct ResolvedEntry {
    pub short_name: String,
    pub wording_snapshot: String,
    /// True only when the master deliverable exists. A usage_history stamp on a
    /// missing record would UPDATE-create a ghost deliverable, so snapshot-only
    /// rows are never stamped.
    pub stamp_usage: bool,
}

/// Resolve one saved-scope row.
///
/// `master` is the library deliverable's `(short_name, body)`, or `None` when
/// it no longer exists. A row whose library item is gone is still saved when
/// it carries its own wording and name (a snapshot-only row), so removing a
/// library item never makes the proposal's saved scope unsavable or loses the
/// wording. Its wording must be non-blank (400 otherwise: a blank snapshot
/// would be a row with nothing in it). Without a name it is a 404.
pub(crate) fn resolve_entry(
    master: Option<(&str, &str)>,
    entry: &crate::models::ScopeDeliverableEntry,
) -> Result<ResolvedEntry, ApiError> {
    match (master, entry.wording_override.as_ref()) {
        (Some((short_name, _)), Some(text)) => Ok(ResolvedEntry {
            short_name: short_name.to_string(),
            wording_snapshot: text.clone(),
            stamp_usage: true,
        }),
        (Some((short_name, body)), None) => Ok(ResolvedEntry {
            short_name: short_name.to_string(),
            wording_snapshot: body.to_string(),
            stamp_usage: true,
        }),
        (None, Some(text)) => {
            if text.trim().is_empty() {
                return Err(ApiError::bad_request(format!(
                    "Deliverable '{}' is no longer in the library; its saved wording must not be empty",
                    entry.deliverable_id
                )));
            }
            match entry.short_name.as_ref().filter(|n| !n.is_empty()) {
                Some(name) => Ok(ResolvedEntry {
                    short_name: name.clone(),
                    wording_snapshot: text.clone(),
                    stamp_usage: false,
                }),
                None => Err(ApiError::not_found("Deliverable", &entry.deliverable_id)),
            }
        }
        (None, None) => Err(ApiError::not_found("Deliverable", &entry.deliverable_id)),
    }
}

/// Helper to strip "fee:" prefix from a fee ID string.
fn strip_fee_prefix(fee_id: &str) -> &str {
    fee_id.strip_prefix("fee:").unwrap_or(fee_id)
}

/// Check if a condition object (deliverable or clause) is satisfied by the
/// request conditions. For each key in the stored condition, the request
/// conditions must contain the same key with the same value (subset matching).
///
/// Shared with `routes::clause_selection` for Stage 2 clause-selection
/// pre-fill (is_default + conditions gate) - same subset-match semantics.
pub(crate) fn condition_matches(
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
                .query("SELECT * FROM stage_config WHERE status = 'active' ORDER BY sort_order ASC")
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
    // Deliverables that exist in the library (snapshot-only rows are excluded).
    let mut stamp_keys: Vec<String> = Vec::new();

    for entry in &body.deliverables {
        let del_key = entry
            .deliverable_id
            .strip_prefix("deliverable:")
            .unwrap_or(&entry.deliverable_id);

        // Fetch the master deliverable for its body text
        let del: Option<Deliverable> = state.db.select(("deliverable", del_key)).await?;

        let master = del
            .as_ref()
            .map(|d| (d.short_name.as_str(), d.body.as_str()));
        let ResolvedEntry {
            short_name,
            wording_snapshot,
            stamp_usage,
        } = resolve_entry(master, entry)?;
        if stamp_usage {
            stamp_keys.push(del_key.to_string());
        }

        deliverables_used.push(json!({
            "deliverable_id": del_key,
            "stage": entry.stage,
            "short_name": short_name,
            "sort_order": entry.sort_order,
            "wording_snapshot": wording_snapshot,
        }));
    }

    // Update the assembly in place: deliverables_used is replaced; stage_labels and
    // manual_items only when sent; generated text, clauses, clause selection and the
    // revision counter are preserved (see scope_store for the field-ownership table).
    scope_store::save_builder(
        &state.db,
        fee_key,
        BuilderState {
            deliverables_used: json!(deliverables_used),
            stage_labels: body.stage_labels.as_ref().map(|l| json!(l)),
            manual_items: body.manual_items.as_ref().map(|m| json!(m)),
        },
    )
    .await?;

    tracing::info!(
        "save_scope_builder: saved {} deliverables for fee_key={}",
        body.deliverables.len(),
        fee_key
    );

    // Stamp usage_history on each library deliverable used. Never on a missing
    // one: UPDATE of an unknown id would create a ghost deliverable record.
    for del_key in &stamp_keys {
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
            .bind(("did", del_key.clone()))
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
            "SELECT deliverables_used, stage_labels, manual_items FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key) LIMIT 1",
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

#[cfg(test)]
mod resolve_entry_tests {
    use super::*;
    use crate::models::ScopeDeliverableEntry;

    fn entry(over: Option<&str>, name: Option<&str>) -> ScopeDeliverableEntry {
        ScopeDeliverableEntry {
            deliverable_id: "d1".into(),
            stage: "Concept".into(),
            sort_order: 1,
            wording_override: over.map(String::from),
            short_name: name.map(String::from),
        }
    }

    fn resolved(name: &str, text: &str, stamp: bool) -> ResolvedEntry {
        ResolvedEntry {
            short_name: name.into(),
            wording_snapshot: text.into(),
            stamp_usage: stamp,
        }
    }

    #[test]
    fn master_body_is_used_without_an_override() {
        let r = resolve_entry(Some(("Short", "Master body")), &entry(None, None)).unwrap();
        assert_eq!(r, resolved("Short", "Master body", true));
    }

    #[test]
    fn override_wins_over_master_body() {
        let r = resolve_entry(Some(("Short", "Master body")), &entry(Some("Mine"), None)).unwrap();
        assert_eq!(r, resolved("Short", "Mine", true));
    }

    #[test]
    fn removed_library_item_is_kept_as_a_snapshot_only_row_and_never_stamped() {
        let r = resolve_entry(None, &entry(Some("Saved wording"), Some("Old item"))).unwrap();
        assert_eq!(r, resolved("Old item", "Saved wording", false));
    }

    #[test]
    fn orphan_with_blank_wording_is_a_400() {
        for blank in ["", "   ", "\n\t "] {
            let e = resolve_entry(None, &entry(Some(blank), Some("Old item"))).unwrap_err();
            assert_eq!(e.status, axum::http::StatusCode::BAD_REQUEST, "{blank:?}");
        }
        // A blank override on a deliverable that still exists is unchanged (master row).
        assert!(resolve_entry(Some(("S", "B")), &entry(Some(""), None)).is_ok());
    }

    #[test]
    fn orphan_without_name_or_wording_is_404() {
        assert!(resolve_entry(None, &entry(None, None)).is_err());
        assert!(resolve_entry(None, &entry(Some("w"), None)).is_err());
        assert!(resolve_entry(None, &entry(Some("w"), Some(""))).is_err());
        assert!(resolve_entry(None, &entry(None, Some("n"))).is_err());
    }
}

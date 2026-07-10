//! Clause selection per-proposal: save and retrieve the `selected_clauses[]` record.
//!
//! Provides two endpoints:
//!
//! * `POST /scope/clause-selection` - save `[{clause_id, included, override_body}]`
//!   onto the `scope_assembly` for a fee, creating the assembly record if it does
//!   not exist yet.
//!
//! * `GET /scope/{fee_id}/clause-selection` - return the current selection
//!   merged with live clause metadata. When no selection has been saved yet
//!   (Stage 2, `is_default` + `conditions` pre-fill): each clause defaults to
//!   included/excluded per its `is_default` flag, gated by its `conditions`
//!   object (if any) against the optional `?conditions=` query param - see
//!   `stage2_default_included` below.
//!
//! `override_body` is structural: once written it is never silently dropped.
//! Downstream generation reads it and uses it in preference to the master body.

use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use e_fees_core::models::record_key_string;

use crate::error::ApiError;
use crate::models::{Clause, SaveClauseSelectionRequest};
use crate::routes::assembly::condition_matches;
use crate::AppState;

/// Helper to strip "fee:" prefix from a fee ID string.
fn strip_fee_prefix(fee_id: &str) -> &str {
    fee_id.strip_prefix("fee:").unwrap_or(fee_id)
}

/// Query parameters for `GET /scope/{fee_id}/clause-selection`.
#[derive(Debug, Deserialize)]
pub struct ClauseSelectionQuery {
    /// JSON-encoded project conditions object (e.g. `{"project_type":"hospitality"}`)
    /// used to gate conditional clause defaults (Stage 2). Absent means "no known
    /// conditions" - conditional clauses default to excluded.
    pub conditions: Option<String>,
}

/// Stage 2 pre-fill default: whether a clause should be included when no
/// custom selection has been saved yet, based on its `is_default` flag and
/// (if present) its `conditions` object matched against the caller's parsed
/// `conditions` query param.
///
/// - `is_default = false` -> always excluded, regardless of conditions.
/// - `is_default = true`, clause has no `conditions` -> always included.
/// - `is_default = true`, clause has `conditions` -> included only if the
///   caller's conditions satisfy every key in the clause's condition object
///   (subset match via `condition_matches`; no caller conditions -> excluded).
fn stage2_default_included(
    is_default: bool,
    clause_conditions: &Option<surrealdb_types::Value>,
    requested_conditions: &Option<Value>,
) -> bool {
    if !is_default {
        return false;
    }
    match clause_conditions {
        None => true,
        Some(cond) => condition_matches(cond, requested_conditions),
    }
}

/// Save clause selections for a fee proposal.
///
/// Creates the `scope_assembly` record if it does not exist yet (with empty
/// defaults for all other fields so that a subsequent `POST /scope/generate`
/// can run immediately).  If the record already exists, only `selected_clauses`
/// and `updated_at` are touched — all other fields are preserved.
#[utoipa::path(
    post,
    path = "/scope/clause-selection",
    tag = "Scope",
    request_body = Value,
    responses(
        (status = 200, description = "Clause selection saved"),
        (status = 400, description = "Invalid request"),
    ),
    security(("api_key" = []))
)]
pub async fn save_clause_selection(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SaveClauseSelectionRequest>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&body.fee_id);
    if fee_key.is_empty() {
        return Err(ApiError::bad_request("fee_id must not be empty"));
    }
    if body.selections.is_empty() {
        return Err(ApiError::bad_request("selections must not be empty"));
    }

    // Normalise clause_id values (strip "clause:" prefix for consistency)
    let sel_json: Vec<Value> = body
        .selections
        .iter()
        .map(|s| {
            let cid = s
                .clause_id
                .strip_prefix("clause:")
                .unwrap_or(&s.clause_id)
                .to_string();
            json!({
                "clause_id": cid,
                "included": s.included,
                "override_body": s.override_body,
            })
        })
        .collect();

    let included_count = body.selections.iter().filter(|s| s.included).count();

    // Determine whether a scope_assembly already exists for this fee
    let mut check_res = state
        .db
        .query(
            "SELECT id FROM scope_assembly \
             WHERE fee_id = type::record('fee', $fee_key) LIMIT 1",
        )
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let existing: Vec<Value> = check_res.take(0)?;

    if existing.is_empty() {
        // Create a minimal scope_assembly so subsequent generate calls work
        state
            .db
            .query(
                "CREATE scope_assembly SET \
                 fee_id          = type::record('fee', $fee_key), \
                 selected_clauses = $sel, \
                 clauses          = [], \
                 generated_text   = '', \
                 llm_polished     = false, \
                 current_revision = 0, \
                 created_at       = time::now(), \
                 updated_at       = time::now()",
            )
            .bind(("fee_key", fee_key.to_string()))
            .bind(("sel", json!(sel_json)))
            .await?
            .check()
            .map_err(|e| {
                tracing::error!(
                    "save_clause_selection: CREATE failed for fee_key={}: {}",
                    fee_key,
                    e
                );
                ApiError::internal(&format!("Failed to create scope assembly: {}", e))
            })?;
    } else {
        // Patch only the selection — preserve everything else
        state
            .db
            .query(
                "UPDATE scope_assembly \
                 SET selected_clauses = $sel, updated_at = time::now() \
                 WHERE fee_id = type::record('fee', $fee_key)",
            )
            .bind(("fee_key", fee_key.to_string()))
            .bind(("sel", json!(sel_json)))
            .await?;
    }

    tracing::info!(
        "save_clause_selection: {} selections ({} included) saved for fee_key={}",
        body.selections.len(),
        included_count,
        fee_key
    );

    Ok(Json(json!({
        "status": "saved",
        "fee_id": fee_key,
        "selections_count": body.selections.len(),
        "included_count": included_count,
    })))
}

/// Get clause selection for a fee proposal.
///
/// Returns all active library clauses merged with the saved selection.
///
/// * If no selection has been saved yet (`has_custom_selection: false`),
///   each clause defaults to its `is_default` flag, gated by `conditions`
///   (Stage 2) - see `stage2_default_included`.
/// * If a selection exists, clauses present in the saved list use the saved
///   `included` / `override_body` values. Active clauses added to the library
///   after the selection was saved default to `included = false`.
#[utoipa::path(
    get,
    path = "/scope/{fee_id}/clause-selection",
    tag = "Scope",
    params(
        ("fee_id" = String, Path, description = "Fee record key"),
        ("conditions" = Option<String>, Query, description = "JSON-encoded project conditions object for Stage 2 conditional-clause pre-fill gating"),
    ),
    responses(
        (status = 200, description = "Clause selection for this fee"),
        (status = 400, description = "Invalid conditions query parameter"),
    ),
    security(("api_key" = []))
)]
pub async fn get_clause_selection(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
    Query(query): Query<ClauseSelectionQuery>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    // Stage 2: parse the optional conditions query param up front so a
    // malformed value fails fast with 400 rather than silently degrading.
    let requested_conditions: Option<Value> = match &query.conditions {
        Some(raw) => {
            let parsed: Value = serde_json::from_str(raw).map_err(|e| {
                ApiError::bad_request(format!("conditions must be valid JSON: {e}"))
            })?;
            Some(parsed)
        }
        None => None,
    };

    // Fetch all active library clauses
    let mut clause_res = state
        .db
        .query(
            "SELECT * FROM clause WHERE status = 'active' \
             ORDER BY sort_order ASC, category ASC",
        )
        .await?;
    let all_clauses: Vec<Clause> = clause_res.take(0)?;

    // Fetch saved selection from scope_assembly (may not exist yet)
    let mut sel_res = state
        .db
        .query(
            "SELECT selected_clauses FROM scope_assembly \
             WHERE fee_id = type::record('fee', $fee_key) LIMIT 1",
        )
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let sel_rows: Vec<Value> = sel_res.take(0)?;

    let raw_selection = sel_rows
        .into_iter()
        .next()
        .and_then(|row| row.get("selected_clauses").cloned())
        .filter(|v| !v.is_null());

    // Build a clause_id → (included, override_body) look-up from saved data
    let mut sel_map: std::collections::HashMap<String, (bool, Option<String>)> =
        std::collections::HashMap::new();

    if let Some(sel) = &raw_selection {
        if let Some(arr) = sel.as_array() {
            for item in arr {
                let cid = item
                    .get("clause_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if cid.is_empty() {
                    continue;
                }
                let included = item
                    .get("included")
                    .and_then(Value::as_bool)
                    .unwrap_or(true);
                let override_body = item
                    .get("override_body")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                sel_map.insert(cid, (included, override_body));
            }
        }
    }

    let has_custom_selection = !sel_map.is_empty();

    // Merge library clauses with saved state
    let selections: Vec<Value> = all_clauses
        .iter()
        .map(|c| {
            // record_key_string in SurrealDB SDK 3.0.4 returns "table:key" for
            // RecordId::String keys (e.g. "clause:abc123"). Strip the table prefix so the
            // cid matches what save_clause_selection stores (and what the test expects).
            let raw_cid = record_key_string(&c.id.key);
            let cid = raw_cid
                .strip_prefix("clause:")
                .unwrap_or(&raw_cid)
                .to_string();

            let (included, override_body) = if has_custom_selection {
                // Clauses added after the last save default to excluded
                sel_map.get(&cid).cloned().unwrap_or((false, None))
            } else {
                // No saved selection yet - Stage 2 pre-fill from is_default + conditions
                let included =
                    stage2_default_included(c.is_default, &c.conditions, &requested_conditions);
                (included, None)
            };

            json!({
                "clause_id":    cid,
                "included":     included,
                "override_body": override_body,
                "title":        c.title,
                "category":     c.category,
                "body":         c.body,
                "sort_order":   c.sort_order,
                "is_default":   c.is_default,
            })
        })
        .collect();

    Ok(Json(json!({
        "fee_id": fee_key,
        "has_custom_selection": has_custom_selection,
        "selections": selections,
    })))
}

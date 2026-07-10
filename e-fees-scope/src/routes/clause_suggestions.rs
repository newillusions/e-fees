//! Clause suggestions (Stage 3): rank unselected library clauses by mined
//! corpus usage frequency.
//!
//! `GET /scope/{fee_id}/clause-suggestions` - see
//! docs/plans/2026-07-10-clause-selection-stage3-design.md for the mining
//! job design. Stats come from `clause_corpus_stat`, populated by the
//! one-time `mine_clause_usage` binary (src/bin/mine_clause_usage.rs).
//!
//! Scope note: this endpoint ranks by `usage_count` and excludes clauses
//! already included in the fee's saved selection. It does NOT filter by
//! discipline as the design doc's recommendation described - the clause
//! library has no discipline-taxonomy field. `clause.category` is a
//! content-type grouping (Administrative/Commercial/Legal/Services) and
//! `clause.subcategory` groups by document structure (Appointment/Areas/
//! Design Stages); neither corresponds to `fee.pricing.disciplines[].id`
//! (short codes like "ld"/"av"). Adding a real discipline dimension to the
//! clause library is a curation decision (owner-gated, out of scope here) -
//! see the PR body.
//!
//! Never auto-includes anything: this is a read-only ranked list. The
//! frontend always requires an explicit user action to add a suggestion.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use e_fees_core::models::record_key_string;

use crate::error::ApiError;
use crate::models::{Clause, ClauseCorpusStat};
use crate::AppState;

fn strip_fee_prefix(fee_id: &str) -> &str {
    fee_id.strip_prefix("fee:").unwrap_or(fee_id)
}

/// Get ranked clause suggestions for a fee proposal.
///
/// Returns unselected clauses ranked by `usage_count` descending. Returns an
/// empty `suggestions` array (never an error) when `clause_corpus_stat` has
/// no rows yet, i.e. the mining job has not run. A clause with `included =
/// true` in the fee's current selection is excluded from its own
/// suggestions; a clause the mining job scored but that has since been
/// archived/deleted from the active library is also excluded.
#[utoipa::path(
    get,
    path = "/scope/{fee_id}/clause-suggestions",
    tag = "Scope",
    params(("fee_id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Ranked clause suggestions for this fee, excluding already-included clauses"),
    ),
    security(("api_key" = []))
)]
pub async fn get_clause_suggestions(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    // Currently-included clause_ids for this fee (Stage 1 selected_clauses).
    let mut sel_res = state
        .db
        .query(
            "SELECT selected_clauses FROM scope_assembly \
             WHERE fee_id = type::record('fee', $fee_key) LIMIT 1",
        )
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let sel_rows: Vec<Value> = sel_res.take(0)?;

    let mut included: HashSet<String> = HashSet::new();
    if let Some(row) = sel_rows.into_iter().next() {
        if let Some(arr) = row.get("selected_clauses").and_then(Value::as_array) {
            for item in arr {
                if item.get("included").and_then(Value::as_bool) == Some(true) {
                    if let Some(cid) = item.get("clause_id").and_then(Value::as_str) {
                        included.insert(cid.to_string());
                    }
                }
            }
        }
    }

    // Active library clauses, keyed by clause_id - a single query so the
    // ranking loop below never does per-row lookups (no N+1).
    let mut clause_res = state
        .db
        .query("SELECT * FROM clause WHERE status = 'active'")
        .await?;
    let active_clauses: Vec<Clause> = clause_res.take(0)?;
    let mut clause_by_id: HashMap<String, Clause> = HashMap::new();
    for c in active_clauses {
        let raw = record_key_string(&c.id.key);
        let cid = raw.strip_prefix("clause:").unwrap_or(&raw).to_string();
        clause_by_id.insert(cid, c);
    }

    // Mined usage stats, ranked. Empty when unmined - not an error.
    let mut stat_res = state
        .db
        .query("SELECT * FROM clause_corpus_stat ORDER BY usage_count DESC")
        .await?;
    let stats: Vec<ClauseCorpusStat> = stat_res.take(0)?;

    let mut suggestions: Vec<Value> = Vec::new();
    for stat in &stats {
        let raw_cid = record_key_string(&stat.clause_id.key);
        let cid = raw_cid.strip_prefix("clause:").unwrap_or(&raw_cid).to_string();

        if included.contains(&cid) {
            continue;
        }
        let Some(clause) = clause_by_id.get(&cid) else {
            continue;
        };

        suggestions.push(json!({
            "clause_id": cid,
            "title": clause.title,
            "category": stat.category,
            "usage_count": stat.usage_count,
            "sample_project_numbers": stat.sample_project_numbers,
            "classified_at": stat.classified_at.to_string(),
        }));
    }

    Ok(Json(json!({
        "fee_id": fee_key,
        "suggestions": suggestions,
    })))
}

//! Scope generation and assembly route handlers.

use std::collections::BTreeMap;
use std::sync::Arc;

use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};

use crate::error::ApiError;
use crate::llm;
use crate::models::{Clause, GenerateScopeRequest, ScopeAssembly, UpdateScopeRequest};
use crate::AppState;

use e_fees_core::models::{dbvalue_to_json, record_key_string};

/// Helper to strip "fee:" prefix from a fee ID string.
fn strip_fee_prefix(fee_id: &str) -> &str {
    fee_id.strip_prefix("fee:").unwrap_or(fee_id)
}

/// Convert a ScopeAssembly to a plain JSON Value.
fn assembly_to_json(a: &ScopeAssembly) -> Value {
    let mut obj = json!({
        "id": record_key_string(&a.id.key),
        "fee_id": record_key_string(&a.fee_id.key),
        "clauses": dbvalue_to_json(&a.clauses),
        "generated_text": a.generated_text,
        "llm_polished": a.llm_polished,
        "created_at": a.created_at.to_string(),
        "updated_at": a.updated_at.to_string(),
    });

    if let Some(ref numbering) = a.numbering {
        obj["numbering"] = dbvalue_to_json(numbering);
    }
    if let Some(ref model) = a.llm_model {
        obj["llm_model"] = json!(model);
    }
    if let Some(ref snapshot) = a.stages_snapshot {
        obj["stages_snapshot"] = json!(snapshot);
    }
    obj["current_revision"] = json!(a.current_revision);

    obj
}

/// Auto-number clauses by category.
///
/// Returns `(numbered_clauses_json, raw_text, numbering_map)`.
fn auto_number_clauses(clauses: &[Clause]) -> (Value, String, Value) {
    // Group clauses by category, preserving sort order
    let mut by_category: BTreeMap<String, Vec<&Clause>> = BTreeMap::new();
    for c in clauses {
        by_category.entry(c.category.clone()).or_default().push(c);
    }

    let mut sections: Vec<Value> = Vec::new();
    let mut raw_lines: Vec<String> = Vec::new();
    let mut numbering_map = json!({});

    for (cat_idx, (category, cat_clauses)) in by_category.iter().enumerate() {
        let cat_num = cat_idx + 1;
        let section_number = format!("{}.0", cat_num);

        raw_lines.push(format!("{} {}", section_number, category));

        let mut clause_items: Vec<Value> = Vec::new();

        for (cl_idx, clause) in cat_clauses.iter().enumerate() {
            let clause_number = format!("{}.{}", cat_num, cl_idx + 1);
            let key = record_key_string(&clause.id.key);

            numbering_map[&key] = json!(clause_number);

            raw_lines.push(format!(
                "{} {} — {}",
                clause_number, clause.title, clause.body
            ));

            clause_items.push(json!({
                "number": clause_number,
                "clause_id": key,
                "title": clause.title,
                "body": clause.body,
            }));
        }

        sections.push(json!({
            "number": section_number,
            "title": category,
            "clauses": clause_items,
        }));

        raw_lines.push(String::new()); // blank line between sections
    }

    let raw_text = raw_lines.join("\n").trim_end().to_string();
    (json!(sections), raw_text, numbering_map)
}

/// Fetch relevant corpus examples for a fee's project.
///
/// Looks up the fee's project_number, then finds corpus documents
/// with matching project numbers. Falls back to recent documents if none match.
/// Returns up to 3 truncated scope text samples.
async fn fetch_corpus_examples(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    fee_key: &str,
) -> Vec<String> {
    // Try to get project_number from the fee's linked project
    let query = format!(
        "SELECT project_id.number.id AS pn OMIT id FROM fee:`{}`;",
        fee_key
    );
    let mut res = match db.query(&query).await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let rows: Vec<Value> = res.take(0).unwrap_or_default();
    let project_number = rows.first().and_then(|r| r["pn"].as_str()).unwrap_or("");

    // Find corpus docs — prefer same project, fall back to recent
    // Get the country-year prefix (e.g., "24-971" from "24-97106")
    let prefix = if project_number.len() >= 6 {
        &project_number[..6]
    } else {
        project_number
    };

    let mut res = if project_number.is_empty() {
        match db
            .query("SELECT extracted_text, filename FROM proposal_corpus ORDER BY created_at DESC LIMIT 3")
            .await
        {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        }
    } else {
        match db
            .query("SELECT extracted_text, filename FROM proposal_corpus WHERE string::startsWith(project_number, $prefix) ORDER BY created_at DESC LIMIT 3")
            .bind(("prefix", prefix.to_string()))
            .await
        {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        }
    };
    let docs: Vec<Value> = res.take(0).unwrap_or_default();

    docs.iter()
        .filter_map(|d| {
            let text = d["extracted_text"].as_str()?;
            let filename = d["filename"].as_str().unwrap_or("unknown");
            // Truncate to ~3000 chars to fit LLM context
            let sample = if text.len() > 3000 {
                format!("{}...", &text[..3000])
            } else {
                text.to_string()
            };
            Some(format!("[From {}]\n{}", filename, sample))
        })
        .collect()
}

/// Create a revision record from the current scope_assembly state before updating.
async fn create_revision(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    fee_key: &str,
    trigger: &str,
) -> Result<i64, ApiError> {
    // Get current assembly
    let mut res = db
        .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let assemblies: Vec<ScopeAssembly> = res.take(0)?;

    let Some(assembly) = assemblies.into_iter().next() else {
        return Ok(0); // No existing assembly to revision
    };

    let next_rev = assembly.current_revision + 1;

    // Store clauses as JSON string (audit snapshot — not queried by field)
    let clauses_json = serde_json::to_string(&dbvalue_to_json(&assembly.clauses))
        .unwrap_or_else(|_| "[]".to_string());

    let rev_result = db
        .query(
            "CREATE scope_revision SET \
         fee_id = type::record('fee', $fee_key), \
         revision = $revision, \
         clauses = $clauses_str, \
         generated_text = $generated_text, \
         stages_at_time = $stages, \
         trigger = $trigger;",
        )
        .bind(("fee_key", fee_key.to_string()))
        .bind(("revision", next_rev))
        .bind(("clauses_str", clauses_json))
        .bind(("generated_text", assembly.generated_text.clone()))
        .bind((
            "stages",
            assembly.stages_snapshot.clone().unwrap_or_default(),
        ))
        .bind(("trigger", trigger.to_string()))
        .await;

    if let Err(ref e) = rev_result {
        tracing::warn!("Failed to create scope revision: {}", e);
    }
    rev_result?;

    // Update current_revision counter
    db.query(
        "UPDATE scope_assembly SET current_revision = $rev \
         WHERE fee_id = type::record('fee', $fee_key);",
    )
    .bind(("fee_key", fee_key.to_string()))
    .bind(("rev", next_rev))
    .await?;

    Ok(next_rev)
}

/// Generate scope assembly from clause library for a fee.
#[utoipa::path(
    post,
    path = "/scope/generate",
    tag = "Scope",
    request_body = Value,
    responses(
        (status = 200, description = "Scope generated"),
        (status = 400, description = "Invalid fee_id"),
        (status = 404, description = "Fee not found"),
        (status = 503, description = "Ollama unavailable (when polish=true)"),
    ),
    security(("api_key" = []))
)]
pub async fn generate_scope(
    State(state): State<Arc<AppState>>,
    Json(body): Json<GenerateScopeRequest>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&body.fee_id);
    if fee_key.is_empty() {
        return Err(ApiError::bad_request("fee_id must not be empty"));
    }

    // 1. Fetch fee for project context (OMIT id because SurrealDB v3 WebSocket SDK
    //    can't deserialize record types to serde_json::Value; traverse record links inline)
    let query = format!(
        "SELECT activity, number, issue_date, status, \
         project_id.name AS project_name, project_id.number.id AS project_number, \
         project_id.city AS city, project_id.country AS country, \
         company_id.name AS company_name, \
         contact_id.full_name AS contact_name \
         OMIT id FROM fee:`{}`",
        fee_key
    );
    let mut res = state.db.query(&query).await?;
    let rows: Vec<Value> = res.take(0)?;
    let fee = rows
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::not_found("Fee", fee_key))?;

    // 2. Fetch active clauses ordered by sort_order, category
    let mut res = state
        .db
        .query("SELECT * FROM clause WHERE status = 'active' ORDER BY sort_order ASC, category ASC")
        .await?;
    let clauses: Vec<Clause> = res.take(0)?;

    if clauses.is_empty() {
        return Err(ApiError::bad_request(
            "No active clauses found in library — add clauses first",
        ));
    }

    // 3. Auto-number by category
    let (sections_json, raw_text, numbering_map) = auto_number_clauses(&clauses);

    // 4. Build stage context string for LLM
    let stage_context = if let Some(ref stages) = body.stages {
        let design: Vec<String> = stages
            .iter()
            .filter(|s| !s.is_post_contract)
            .map(|s| format!("{} ({})", s.name, s.code))
            .collect();
        let post: Vec<String> = stages
            .iter()
            .filter(|s| s.is_post_contract)
            .map(|s| format!("{} ({})", s.name, s.code))
            .collect();
        let mut ctx = String::new();
        if !design.is_empty() {
            ctx.push_str(&format!("Design stages: {}", design.join(", ")));
        }
        if !post.is_empty() {
            if !ctx.is_empty() {
                ctx.push_str(". ");
            }
            ctx.push_str(&format!("Post-contract stages: {}", post.join(", ")));
        }
        Some(ctx)
    } else {
        None
    };

    // 5. Optionally polish with LLM (using corpus examples if available)
    let (generated_text, llm_polished, llm_model) = if body.polish {
        // Fetch similar proposal texts from corpus for context
        let corpus_examples = fetch_corpus_examples(&state.db, fee_key).await;

        let polished = llm::polish_scope(
            &state.http,
            &state.ollama_url,
            &state.ollama_model,
            &fee,
            &raw_text,
            &corpus_examples,
            stage_context.as_deref(),
        )
        .await?;
        (polished, true, Some(state.ollama_model.clone()))
    } else {
        (raw_text, false, None)
    };

    // 6. Build stages snapshot for storage
    let stages_snapshot: Vec<String> = body
        .stages
        .as_ref()
        .map(|s| s.iter().map(|st| st.name.clone()).collect())
        .unwrap_or_default();

    // 7. Save current state as a revision before overwriting
    let revision_num = create_revision(&state.db, fee_key, "regeneration").await?;

    // 8. Upsert into scope_assembly (preserve current_revision from create_revision)
    let mut optional_sets = String::new();
    if llm_model.is_some() {
        optional_sets.push_str(", llm_model = $llm_model");
    }

    let upsert_query = format!(
        "DELETE scope_assembly WHERE fee_id = type::record('fee', $fee_key); \
         CREATE scope_assembly SET \
         fee_id = type::record('fee', $fee_key), \
         clauses = $clauses, \
         generated_text = $generated_text, \
         numbering = $numbering, \
         llm_polished = $llm_polished, \
         stages_snapshot = $stages_snapshot, \
         current_revision = $current_revision, \
         created_at = time::now(), \
         updated_at = time::now(){optional_sets};"
    );

    let mut q = state
        .db
        .query(&upsert_query)
        .bind(("fee_key", fee_key.to_string()))
        .bind(("clauses", sections_json))
        .bind(("generated_text", generated_text.clone()))
        .bind(("numbering", numbering_map))
        .bind(("llm_polished", llm_polished))
        .bind(("stages_snapshot", stages_snapshot))
        .bind(("current_revision", revision_num));

    if let Some(ref model) = llm_model {
        q = q.bind(("llm_model", model.clone()));
    }

    let mut response = q.await?;
    // Statement 0 is DELETE, statement 1 is CREATE
    let assemblies: Vec<ScopeAssembly> = response.take(1)?;

    match assemblies.into_iter().next() {
        Some(a) => Ok(Json(json!({ "data": assembly_to_json(&a) }))),
        None => Err(ApiError::internal("Failed to create scope assembly")),
    }
}

/// Get existing scope assembly for a fee.
#[utoipa::path(
    get,
    path = "/scope/{fee_id}",
    tag = "Scope",
    params(("fee_id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Scope assembly found"),
        (status = 404, description = "No scope assembly for this fee"),
    ),
    security(("api_key" = []))
)]
pub async fn get_scope(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    let mut res = state
        .db
        .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
        .bind(("fee_key", fee_key.to_string()))
        .await?;

    let assemblies: Vec<ScopeAssembly> = res.take(0)?;

    match assemblies.into_iter().next() {
        Some(a) => Ok(Json(json!({ "data": assembly_to_json(&a) }))),
        None => Err(ApiError::not_found("Scope assembly for fee", &fee_id)),
    }
}

/// Manually update scope assembly text or clauses.
#[utoipa::path(
    put,
    path = "/scope/{fee_id}",
    tag = "Scope",
    params(("fee_id" = String, Path, description = "Fee record key")),
    request_body = Value,
    responses(
        (status = 200, description = "Scope updated"),
        (status = 404, description = "No scope assembly for this fee"),
    ),
    security(("api_key" = []))
)]
pub async fn update_scope(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
    Json(body): Json<UpdateScopeRequest>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    // Build dynamic SET clauses
    let mut sets: Vec<String> = Vec::new();

    if body.generated_text.is_some() {
        sets.push("generated_text = $generated_text".into());
    }
    if body.clauses.is_some() {
        sets.push("clauses = $clauses".into());
    }

    if sets.is_empty() {
        return Err(ApiError::bad_request(
            "At least one of generated_text or clauses must be provided",
        ));
    }

    // Save current state as a revision before overwriting
    let _revision = create_revision(&state.db, fee_key, "manual_edit").await?;

    // Manual edit clears the LLM polish flag
    sets.push("llm_polished = false".into());
    sets.push("updated_at = time::now()".into());

    let set_str = sets.join(", ");
    let query = format!(
        "UPDATE scope_assembly SET {set_str} WHERE fee_id = type::record('fee', $fee_key); \
         SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key);"
    );

    let mut q = state
        .db
        .query(&query)
        .bind(("fee_key", fee_key.to_string()));

    if let Some(ref text) = body.generated_text {
        q = q.bind(("generated_text", text.clone()));
    }
    if let Some(ref clauses) = body.clauses {
        q = q.bind(("clauses", clauses.to_string()));
    }

    let mut response = q.await?;
    let assemblies: Vec<ScopeAssembly> = response.take(1)?;

    match assemblies.into_iter().next() {
        Some(a) => Ok(Json(json!({ "data": assembly_to_json(&a) }))),
        None => Err(ApiError::not_found("Scope assembly for fee", &fee_id)),
    }
}

/// Re-polish existing scope text with LLM.
#[utoipa::path(
    post,
    path = "/scope/{fee_id}/regenerate",
    tag = "Scope",
    params(("fee_id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Scope re-polished"),
        (status = 404, description = "No scope assembly for this fee"),
        (status = 503, description = "Ollama unavailable"),
    ),
    security(("api_key" = []))
)]
pub async fn regenerate_scope(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    // Fetch existing assembly
    let mut res = state
        .db
        .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let assemblies: Vec<ScopeAssembly> = res.take(0)?;
    let assembly = assemblies
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::not_found("Scope assembly for fee", &fee_id))?;

    // Fetch fee for project context (OMIT id — record types break serde_json::Value)
    let query = format!(
        "SELECT activity, number, issue_date, status, \
         project_id.name AS project_name, project_id.number.id AS project_number, \
         project_id.city AS city, project_id.country AS country, \
         company_id.name AS company_name, \
         contact_id.full_name AS contact_name \
         OMIT id FROM fee:`{}`",
        fee_key
    );
    let mut res = state.db.query(&query).await?;
    let rows: Vec<Value> = res.take(0)?;
    let fee = rows.into_iter().next().unwrap_or(json!({}));

    // Fetch corpus examples and re-polish
    let corpus_examples = fetch_corpus_examples(&state.db, fee_key).await;

    let polished = llm::polish_scope(
        &state.http,
        &state.ollama_url,
        &state.ollama_model,
        &fee,
        &assembly.generated_text,
        &corpus_examples,
        None,
    )
    .await?;

    // Save current state as a revision before overwriting
    let _revision = create_revision(&state.db, fee_key, "regeneration").await?;

    // Update record
    let update_query = "UPDATE scope_assembly SET \
         generated_text = $generated_text, \
         llm_polished = true, \
         llm_model = $llm_model, \
         updated_at = time::now() \
         WHERE fee_id = type::record('fee', $fee_key); \
         SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key);";

    let mut response = state
        .db
        .query(update_query)
        .bind(("fee_key", fee_key.to_string()))
        .bind(("generated_text", polished))
        .bind(("llm_model", state.ollama_model.clone()))
        .await?;

    let updated: Vec<ScopeAssembly> = response.take(1)?;

    match updated.into_iter().next() {
        Some(a) => Ok(Json(json!({ "data": assembly_to_json(&a) }))),
        None => Err(ApiError::internal("Failed to update scope assembly")),
    }
}

/// Export scope as structured JSON for InDesign.
#[utoipa::path(
    get,
    path = "/scope/{fee_id}/export",
    tag = "Scope",
    params(("fee_id" = String, Path, description = "Fee record key")),
    responses(
        (status = 200, description = "Structured scope export"),
        (status = 404, description = "No scope assembly for this fee"),
    ),
    security(("api_key" = []))
)]
pub async fn export_scope(
    State(state): State<Arc<AppState>>,
    Path(fee_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let fee_key = strip_fee_prefix(&fee_id);

    let mut res = state
        .db
        .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let assemblies: Vec<ScopeAssembly> = res.take(0)?;
    let assembly = assemblies
        .into_iter()
        .next()
        .ok_or_else(|| ApiError::not_found("Scope assembly for fee", &fee_id))?;

    // Parse clauses back to structured sections
    let sections = dbvalue_to_json(&assembly.clauses);

    Ok(Json(json!({
        "fee_id": record_key_string(&assembly.fee_id.key),
        "sections": sections,
        "generated_text": assembly.generated_text,
        "metadata": {
            "generated_at": assembly.updated_at.to_string(),
            "llm_polished": assembly.llm_polished,
            "llm_model": assembly.llm_model,
        }
    })))
}

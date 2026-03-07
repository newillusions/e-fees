//! Proposal corpus ingestion and search route handlers.

use std::path::Path as FsPath;
use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use e_fees_core::models::{dbvalue_to_json, record_key_string};

use crate::error::ApiError;
use crate::models::{IngestBatchRequest, IngestRequest, ProposalCorpus};
use crate::AppState;

/// Regex pattern for project numbers like "25-97101".
fn extract_project_number(filename: &str) -> Option<String> {
    let re = regex::Regex::new(r"\d{2}-\d{3,5}").unwrap();
    re.find(filename).map(|m| m.as_str().to_string())
}

/// Convert a ProposalCorpus to a JSON Value (full representation).
fn corpus_to_json(doc: &ProposalCorpus) -> Value {
    let mut obj = json!({
        "id": record_key_string(&doc.id.key),
        "filename": doc.filename,
        "extracted_text": doc.extracted_text,
        "created_at": doc.created_at.to_string(),
    });

    if let Some(ref pn) = doc.project_number {
        obj["project_number"] = json!(pn);
    }
    if let Some(ref name) = doc.project_name {
        obj["project_name"] = json!(name);
    }
    if let Some(ref sections) = doc.sections {
        obj["sections"] = dbvalue_to_json(sections);
    }
    if let Some(ref meta) = doc.metadata {
        obj["metadata"] = dbvalue_to_json(meta);
    }
    if doc.embedding.is_some() {
        obj["has_embedding"] = json!(true);
    }

    obj
}

/// Convert a ProposalCorpus to a summary JSON (for list/search — no full text).
fn corpus_to_summary(doc: &ProposalCorpus) -> Value {
    let mut obj = json!({
        "id": record_key_string(&doc.id.key),
        "filename": doc.filename,
        "created_at": doc.created_at.to_string(),
        "text_length": doc.extracted_text.len(),
    });

    if let Some(ref pn) = doc.project_number {
        obj["project_number"] = json!(pn);
    }
    if let Some(ref name) = doc.project_name {
        obj["project_name"] = json!(name);
    }
    if doc.embedding.is_some() {
        obj["has_embedding"] = json!(true);
    }

    obj
}

/// Call Docling-Serve to extract text from a PDF file.
async fn extract_text_from_pdf(
    http: &reqwest::Client,
    docling_url: &str,
    file_path: &str,
) -> Result<String, String> {
    let url = format!("{}/v1/convert/source", docling_url);

    let res = http
        .post(&url)
        .json(&json!({
            "source": file_path,
            "options": { "to": ["text"] }
        }))
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("Docling request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!(
            "Docling returned HTTP {}: {}",
            status,
            body.chars().take(500).collect::<String>()
        ));
    }

    let body: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Docling response: {}", e))?;

    // Docling response typically has document.text or output.text — try common paths
    if let Some(text) = body
        .pointer("/document/text")
        .or_else(|| body.pointer("/output/text"))
        .or_else(|| body.pointer("/text"))
        .and_then(|v| v.as_str())
    {
        Ok(text.to_string())
    } else {
        // Fallback: return the full response as a string so we can debug
        Err(format!(
            "Could not extract text from Docling response. Keys: {:?}",
            body.as_object().map(|o| o.keys().collect::<Vec<_>>())
        ))
    }
}

/// Ingest a single PDF — extract text and store in proposal_corpus.
async fn do_ingest(
    state: &AppState,
    file_path: &str,
    project_name_override: Option<&str>,
) -> Result<Value, ApiError> {
    let filename = FsPath::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    let project_number = extract_project_number(&filename);

    let extracted_text = extract_text_from_pdf(&state.http, &state.docling_url, file_path)
        .await
        .map_err(|e| {
            warn!("Docling extraction failed for {}: {}", file_path, e);
            ApiError::service_unavailable(format!("PDF extraction failed: {}", e))
        })?;

    if extracted_text.trim().is_empty() {
        return Err(ApiError::bad_request(
            "Extracted text is empty — PDF may be image-only or corrupted",
        ));
    }

    let project_name = project_name_override.map(|s| s.to_string());

    // Build optional SET fragments — never bind NULL for option<T>
    let mut optional_sets = String::new();
    if project_number.is_some() {
        optional_sets.push_str(", project_number = $project_number");
    }
    if project_name.is_some() {
        optional_sets.push_str(", project_name = $project_name");
    }

    let query = format!(
        "CREATE proposal_corpus SET \
         filename = $filename, \
         extracted_text = $extracted_text, \
         created_at = time::now(){optional_sets};"
    );

    let mut q = state
        .db
        .query(&query)
        .bind(("filename", filename.clone()))
        .bind(("extracted_text", extracted_text));

    if let Some(ref pn) = project_number {
        q = q.bind(("project_number", pn.clone()));
    }
    if let Some(ref name) = project_name {
        q = q.bind(("project_name", name.clone()));
    }

    let mut response = q.await?;
    let docs: Vec<ProposalCorpus> = response.take(0)?;

    match docs.into_iter().next() {
        Some(doc) => {
            info!("Ingested corpus document: {}", filename);
            Ok(corpus_to_json(&doc))
        }
        None => Err(ApiError::internal(
            "Failed to create corpus record — no record returned",
        )),
    }
}

// ── Route handlers ──────────────────────────────────────────────────

/// Ingest a single PDF into the proposal corpus.
#[utoipa::path(
    post,
    path = "/corpus",
    tag = "Corpus",
    request_body = Value,
    responses(
        (status = 201, description = "Document ingested"),
        (status = 400, description = "Validation error"),
        (status = 503, description = "Docling-Serve unavailable"),
    ),
    security(("api_key" = []))
)]
pub async fn ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IngestRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.file_path.trim().is_empty() {
        return Err(ApiError::bad_request("file_path must not be empty"));
    }

    let doc = do_ingest(&state, &body.file_path, body.project_name.as_deref()).await?;

    Ok(Json(json!({ "data": doc })))
}

/// Batch-ingest all PDFs from a directory.
#[utoipa::path(
    post,
    path = "/corpus/ingest-batch",
    tag = "Corpus",
    request_body = Value,
    responses(
        (status = 200, description = "Batch ingestion summary"),
        (status = 400, description = "Invalid directory"),
    ),
    security(("api_key" = []))
)]
pub async fn ingest_batch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IngestBatchRequest>,
) -> Result<Json<Value>, ApiError> {
    if body.directory.trim().is_empty() {
        return Err(ApiError::bad_request("directory must not be empty"));
    }

    let dir = FsPath::new(&body.directory);
    if !dir.is_dir() {
        return Err(ApiError::bad_request(format!(
            "Directory does not exist or is not accessible: {}",
            body.directory
        )));
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| ApiError::bad_request(format!("Cannot read directory: {}", e)))?;

    let mut pdf_paths: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("pdf") {
            pdf_paths.push(path.to_string_lossy().to_string());
        }
    }

    if pdf_paths.is_empty() {
        return Ok(Json(json!({
            "total": 0,
            "succeeded": 0,
            "failed": 0,
            "errors": [],
            "message": "No PDF files found in directory"
        })));
    }

    let total = pdf_paths.len();
    let mut succeeded = 0usize;
    let mut errors: Vec<Value> = Vec::new();

    for path in &pdf_paths {
        match do_ingest(&state, path, None).await {
            Ok(_) => succeeded += 1,
            Err(e) => {
                errors.push(json!({
                    "file": path,
                    "error": e.message
                }));
            }
        }
    }

    let failed = total - succeeded;

    info!(
        "Batch ingestion complete: {}/{} succeeded, {} failed",
        succeeded, total, failed
    );

    Ok(Json(json!({
        "total": total,
        "succeeded": succeeded,
        "failed": failed,
        "errors": errors
    })))
}

/// List all ingested corpus documents.
#[utoipa::path(
    get,
    path = "/corpus",
    tag = "Corpus",
    responses(
        (status = 200, description = "List of corpus documents"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn list_corpus(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let mut response = state
        .db
        .query("SELECT * FROM proposal_corpus ORDER BY created_at DESC")
        .await?;
    let docs: Vec<ProposalCorpus> = response.take(0)?;
    let total = docs.len();

    let data: Vec<Value> = docs.iter().map(corpus_to_summary).collect();

    Ok(Json(json!({ "data": data, "total": total })))
}

/// Get a single corpus document by ID.
#[utoipa::path(
    get,
    path = "/corpus/{id}",
    tag = "Corpus",
    params(("id" = String, Path, description = "Corpus document record key")),
    responses(
        (status = 200, description = "Document found"),
        (status = 404, description = "Document not found"),
    ),
    security(("api_key" = []))
)]
pub async fn get_corpus_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let key = id.strip_prefix("proposal_corpus:").unwrap_or(&id);

    let doc: Option<ProposalCorpus> = state.db.select(("proposal_corpus", key)).await?;

    match doc {
        Some(d) => Ok(Json(json!({ "data": corpus_to_json(&d) }))),
        None => Err(ApiError::not_found("Corpus document", &id)),
    }
}

/// Query parameters for corpus search.
#[derive(Debug, Deserialize)]
pub struct CorpusSearchParams {
    /// Search terms (text CONTAINS).
    pub q: Option<String>,
}

/// Search corpus documents by text content (placeholder — simple CONTAINS).
#[utoipa::path(
    get,
    path = "/corpus/search",
    tag = "Corpus",
    params(
        ("q" = Option<String>, Query, description = "Search terms"),
    ),
    responses(
        (status = 200, description = "Search results"),
        (status = 401, description = "Missing or invalid API key"),
    ),
    security(("api_key" = []))
)]
pub async fn search_corpus(
    State(state): State<Arc<AppState>>,
    params: Query<CorpusSearchParams>,
) -> Result<Json<Value>, ApiError> {
    let query_text = params
        .q
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_string();

    if query_text.is_empty() {
        return Err(ApiError::bad_request("Query parameter 'q' is required"));
    }

    let query = "SELECT * FROM proposal_corpus \
                 WHERE string::contains(string::lowercase(extracted_text), string::lowercase($query)) \
                 ORDER BY created_at DESC \
                 LIMIT 20";

    let mut response = state
        .db
        .query(query)
        .bind(("query", query_text))
        .await?;
    let docs: Vec<ProposalCorpus> = response.take(0)?;
    let total = docs.len();

    let data: Vec<Value> = docs.iter().map(corpus_to_summary).collect();

    Ok(Json(json!({ "data": data, "total": total })))
}

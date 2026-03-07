//! Proposal corpus ingestion and search route handlers.
//!
//! Supports two extraction methods:
//! - **vision** (default): PDF → PNG (Stirling) → Qwen3.5 vision → Qwen3.5 text verification
//! - **docling**: PDF → Docling-Serve text extraction (simpler layouts)

use std::path::Path as FsPath;
use std::sync::Arc;

use axum::{extract::Path, extract::Query, extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use e_fees_core::models::{dbvalue_to_json, record_key_string};

use crate::error::ApiError;
use crate::models::{ExtractionMethod, IngestBatchRequest, IngestRequest, ProposalCorpus};
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

// ── Extraction pipelines ────────────────────────────────────────────

/// Call Docling-Serve to extract text from a PDF file.
async fn extract_via_docling(
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

    if let Some(text) = body
        .pointer("/document/text")
        .or_else(|| body.pointer("/output/text"))
        .or_else(|| body.pointer("/text"))
        .and_then(|v| v.as_str())
    {
        Ok(text.to_string())
    } else {
        Err(format!(
            "Could not extract text from Docling response. Keys: {:?}",
            body.as_object().map(|o| o.keys().collect::<Vec<_>>())
        ))
    }
}

/// Step 1: Convert PDF to PNG images via Stirling PDF.
/// Returns a Vec of PNG image bytes (one per page).
async fn pdf_to_pngs(
    http: &reqwest::Client,
    stirling_url: &str,
    pdf_bytes: &[u8],
    filename: &str,
) -> Result<Vec<Vec<u8>>, String> {
    let url = format!("{}/api/v1/convert/pdf/img", stirling_url);

    let form = reqwest::multipart::Form::new()
        .part(
            "fileInput",
            reqwest::multipart::Part::bytes(pdf_bytes.to_vec())
                .file_name(filename.to_string())
                .mime_str("application/pdf")
                .unwrap(),
        )
        .text("imageFormat", "png")
        .text("singleOrMultiple", "multiple")
        .text("dpi", "200");

    let res = http
        .post(&url)
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .map_err(|e| format!("Stirling request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!(
            "Stirling returned HTTP {}: {}",
            status,
            body.chars().take(500).collect::<String>()
        ));
    }

    // Stirling returns a ZIP file containing PNG images
    let zip_bytes = res
        .bytes()
        .await
        .map_err(|e| format!("Failed to read Stirling response: {}", e))?;

    let cursor = std::io::Cursor::new(zip_bytes.as_ref());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    let mut pages: Vec<(String, Vec<u8>)> = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;
        if file.name().ends_with(".png") {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buf)
                .map_err(|e| format!("Failed to read PNG from ZIP: {}", e))?;
            pages.push((file.name().to_string(), buf));
        }
    }

    // Sort by filename to maintain page order
    pages.sort_by(|a, b| a.0.cmp(&b.0));

    if pages.is_empty() {
        return Err("Stirling returned no PNG pages".to_string());
    }

    Ok(pages.into_iter().map(|(_, bytes)| bytes).collect())
}

/// Step 2: Send a PNG image to Qwen3.5 vision to extract text.
async fn vision_extract_page(
    http: &reqwest::Client,
    ollama_url: &str,
    model: &str,
    png_bytes: &[u8],
    page_num: usize,
) -> Result<String, String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);

    let body = json!({
        "model": model,
        "prompt": format!(
            "Extract ALL text from this fee proposal page (page {}). \
             Preserve the structure: headings, numbered lists, tables, bullet points. \
             Output the text exactly as written — do not summarize or paraphrase. \
             For tables, use markdown table format. \
             For multi-column layouts, process left column first, then right column.",
            page_num
        ),
        "images": [b64],
        "stream": false,
        "think": false,
        "options": {
            "temperature": 0.1
        }
    });

    let res = http
        .post(format!("{}/api/generate", ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|e| format!("Ollama vision request failed for page {}: {}", page_num, e))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!(
            "Ollama vision returned HTTP {} for page {}: {}",
            status, page_num, text
        ));
    }

    let result: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama vision response: {}", e))?;

    result["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Ollama vision response missing 'response' for page {}", page_num))
}

/// Step 3: Verification pass — send extracted text through text-only LLM
/// to clean up OCR artifacts and verify completeness.
async fn verify_extraction(
    http: &reqwest::Client,
    ollama_url: &str,
    model: &str,
    raw_text: &str,
    filename: &str,
) -> Result<String, String> {
    let body = json!({
        "model": model,
        "prompt": format!(
            "Below is text extracted from a fee proposal PDF '{}' using vision OCR. \
             The text is divided into pages marked with [PAGE N] headers.\n\n\
             STRICT RULES:\n\
             1. Keep ALL [PAGE N] markers exactly as they appear.\n\
             2. NEVER reorder, move, or rearrange any content between pages.\n\
             3. Process each page independently — only fix OCR artifacts within that page.\n\
             4. Fix garbled characters, broken words, and obvious OCR typos.\n\
             5. Preserve ALL content, structure, numbering, tables, and formatting.\n\
             6. Do NOT summarize, remove, merge, or add content.\n\
             7. Do NOT reorganize sections across page boundaries.\n\n\
             --- EXTRACTED TEXT ---\n{}\n--- END ---\n\n\
             Output the cleaned text with all [PAGE N] markers preserved in original order:",
            filename, raw_text
        ),
        "system": "You are a document processing assistant. Fix OCR artifacts while preserving \
                    EXACT page order. The [PAGE N] markers define strict boundaries — never move \
                    content across page boundaries or reorder pages. Process each page in isolation.",
        "stream": false,
        "think": false,
        "options": {
            "temperature": 0.1
        }
    });

    let res = http
        .post(format!("{}/api/generate", ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|e| format!("Ollama verification request failed: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(format!("Ollama verification returned HTTP {}: {}", status, text));
    }

    let result: Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama verification response: {}", e))?;

    result["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Ollama verification response missing 'response' field".to_string())
}

/// Multi-pass vision extraction pipeline:
/// PDF → PNG (Stirling) → Qwen3.5 vision per page → Qwen3.5 text verification
async fn extract_via_vision(
    http: &reqwest::Client,
    stirling_url: &str,
    ollama_url: &str,
    model: &str,
    file_path: &str,
) -> Result<String, String> {
    let filename = FsPath::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "document.pdf".to_string());

    // Read the PDF file
    let pdf_bytes = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read PDF file '{}': {}", file_path, e))?;

    info!("Vision pipeline: converting {} to PNG pages via Stirling", filename);

    // Step 1: PDF → PNGs via Stirling
    let pages = pdf_to_pngs(http, stirling_url, &pdf_bytes, &filename).await?;
    info!("Vision pipeline: {} pages extracted from {}", pages.len(), filename);

    // Step 2: Vision extraction per page
    let mut page_texts = Vec::with_capacity(pages.len());
    for (i, png_bytes) in pages.iter().enumerate() {
        let page_num = i + 1;
        info!("Vision pipeline: extracting page {}/{} of {}", page_num, pages.len(), filename);
        let text = vision_extract_page(http, ollama_url, model, png_bytes, page_num).await?;
        page_texts.push(text);
    }

    // Add explicit page markers so the verification pass cannot reorder content
    let combined: String = page_texts
        .iter()
        .enumerate()
        .map(|(i, text)| format!("[PAGE {}]\n{}", i + 1, text))
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");

    // Step 3: Verification pass
    info!("Vision pipeline: running verification pass on {}", filename);
    let verified = verify_extraction(http, ollama_url, model, &combined, &filename).await?;

    Ok(verified)
}

/// Extract text from a PDF using the specified method.
async fn extract_text_from_pdf(
    state: &AppState,
    file_path: &str,
    method: &ExtractionMethod,
) -> Result<String, String> {
    match method {
        ExtractionMethod::Docling => {
            extract_via_docling(&state.http, &state.docling_url, file_path).await
        }
        ExtractionMethod::Vision => {
            extract_via_vision(
                &state.http,
                &state.stirling_url,
                &state.ollama_url,
                &state.ollama_model,
                file_path,
            )
            .await
        }
    }
}

/// Ingest a single PDF — extract text and store in proposal_corpus.
async fn do_ingest(
    state: &AppState,
    file_path: &str,
    project_name_override: Option<&str>,
    method: &ExtractionMethod,
) -> Result<Value, ApiError> {
    let filename = FsPath::new(file_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    // Deduplication: check if this filename has already been ingested
    let mut dup_res = state
        .db
        .query("SELECT count() AS total FROM proposal_corpus WHERE filename = $filename GROUP ALL")
        .bind(("filename", filename.clone()))
        .await?;
    let counts: Vec<Value> = dup_res.take(0)?;
    let already_exists = counts
        .first()
        .and_then(|v| v["total"].as_i64())
        .unwrap_or(0)
        > 0;
    if already_exists {
        return Err(ApiError::conflict(format!(
            "Document '{}' already exists in corpus. Delete it first to re-ingest.",
            filename
        )));
    }

    let project_number = extract_project_number(&filename);

    let extracted_text = extract_text_from_pdf(state, file_path, method)
        .await
        .map_err(|e| {
            warn!("Extraction failed for {}: {}", file_path, e);
            ApiError::service_unavailable(format!("PDF extraction failed: {}", e))
        })?;

    if extracted_text.trim().is_empty() {
        return Err(ApiError::bad_request(
            "Extracted text is empty — PDF may be image-only or corrupted",
        ));
    }

    let method_name = match method {
        ExtractionMethod::Docling => "docling",
        ExtractionMethod::Vision => "vision",
    };

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
         metadata = {{ extraction_method: $method }}, \
         created_at = time::now(){optional_sets};"
    );

    let mut q = state
        .db
        .query(&query)
        .bind(("filename", filename.clone()))
        .bind(("extracted_text", extracted_text))
        .bind(("method", method_name));

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
            info!("Ingested corpus document: {} (method: {})", filename, method_name);
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
        (status = 503, description = "Extraction service unavailable"),
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

    let doc = do_ingest(&state, &body.file_path, body.project_name.as_deref(), &body.method).await?;

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
        match do_ingest(&state, path, None, &body.method).await {
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

/// Extract standard clauses from the corpus using LLM analysis.
///
/// Analyzes all (or filtered) corpus documents to identify recurring sections
/// and creates clause library entries from them.
#[utoipa::path(
    post,
    path = "/corpus/extract-clauses",
    tag = "Corpus",
    request_body = Value,
    responses(
        (status = 200, description = "Clauses extracted and created"),
        (status = 400, description = "No corpus documents found"),
        (status = 503, description = "Ollama unavailable"),
    ),
    security(("api_key" = []))
)]
pub async fn extract_clauses(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // Optional filter by project_number
    let filter = body
        .get("project_number")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (query, needs_bind) = if filter.is_empty() {
        ("SELECT * FROM proposal_corpus ORDER BY created_at DESC LIMIT 10".to_string(), false)
    } else {
        ("SELECT * FROM proposal_corpus WHERE project_number = $pn ORDER BY created_at DESC LIMIT 10".to_string(), true)
    };

    let mut q = state.db.query(&query);
    if needs_bind {
        q = q.bind(("pn", filter.to_string()));
    }

    let mut response = q.await?;
    let docs: Vec<ProposalCorpus> = response.take(0)?;

    if docs.is_empty() {
        return Err(ApiError::bad_request(
            "No corpus documents found to extract clauses from",
        ));
    }

    // Combine extracted texts (truncated per doc to fit context window)
    let corpus_sample: String = docs
        .iter()
        .take(5)
        .map(|d| {
            let text = if d.extracted_text.len() > 4000 {
                format!("{}...", &d.extracted_text[..4000])
            } else {
                d.extracted_text.clone()
            };
            format!("--- {} ---\n{}", d.filename, text)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    // Ask LLM to identify standard clause sections
    let prompt = format!(
        "Analyze these fee proposal documents and identify the STANDARD RECURRING SECTIONS \
         that appear across proposals. For each section, extract:\n\
         1. category — the broad grouping (e.g., \"Services\", \"Commercial\", \"Legal\", \"Administrative\")\n\
         2. title — the section heading as it appears\n\
         3. body — the standard/template text for that section\n\n\
         Output ONLY valid JSON — an array of objects with fields: category, title, body.\n\
         Focus on sections that would be REUSABLE as template clauses.\n\
         Exclude project-specific details (dates, amounts, names).\n\
         Keep body text as the generalised template version.\n\n\
         Documents:\n{}\n\n\
         JSON output:",
        corpus_sample
    );

    let llm_body = json!({
        "model": &state.ollama_model,
        "prompt": prompt,
        "system": "You are a document analysis assistant. Extract reusable clause templates \
                    from fee proposals. Output ONLY valid JSON arrays. No markdown, no explanation.",
        "stream": false,
        "think": false,
        "options": { "temperature": 0.2 }
    });

    let res = state
        .http
        .post(format!("{}/api/generate", state.ollama_url))
        .json(&llm_body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|e| ApiError::service_unavailable(format!("Ollama error: {}", e)))?;

    if !res.status().is_success() {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(ApiError::service_unavailable(format!(
            "Ollama returned {}: {}",
            status, text
        )));
    }

    let result: Value = res
        .json()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to parse Ollama response: {}", e)))?;

    let raw_response = result["response"]
        .as_str()
        .ok_or_else(|| ApiError::internal("Ollama response missing 'response' field"))?;

    // Try to parse the JSON array from the LLM response
    // Strip markdown code fences if present
    let cleaned = raw_response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let clauses_array: Vec<Value> = serde_json::from_str(cleaned).map_err(|e| {
        ApiError::internal(format!(
            "Failed to parse LLM clause output as JSON: {}. Raw: {}",
            e,
            &cleaned[..cleaned.len().min(500)]
        ))
    })?;

    // Insert each clause into the clause library
    let mut created = 0usize;
    let mut skipped = 0usize;

    for (i, clause) in clauses_array.iter().enumerate() {
        let category = clause["category"].as_str().unwrap_or("Uncategorized");
        let title = clause["title"].as_str().unwrap_or("");
        let body_text = clause["body"].as_str().unwrap_or("");

        if title.is_empty() || body_text.is_empty() {
            skipped += 1;
            continue;
        }

        // Check for duplicate title within same category
        let mut dup_res = state
            .db
            .query(
                "SELECT count() AS total FROM clause WHERE category = $cat AND title = $title AND status = 'active' GROUP ALL",
            )
            .bind(("cat", category.to_string()))
            .bind(("title", title.to_string()))
            .await?;
        let dup_counts: Vec<Value> = dup_res.take(0)?;
        let clause_exists = dup_counts
            .first()
            .and_then(|v| v["total"].as_i64())
            .unwrap_or(0)
            > 0;
        if clause_exists {
            skipped += 1;
            continue;
        }

        let sort_order = (i as i64 + 1) * 10;
        state
            .db
            .query(
                "CREATE clause SET \
                 category = $category, \
                 title = $title, \
                 body = $body_text, \
                 sort_order = $sort_order, \
                 is_default = true, \
                 status = 'active', \
                 version = 1, \
                 tags = ['auto-extracted'], \
                 created_at = time::now(), \
                 updated_at = time::now();",
            )
            .bind(("category", category.to_string()))
            .bind(("title", title.to_string()))
            .bind(("body_text", body_text.to_string()))
            .bind(("sort_order", sort_order))
            .await?;

        created += 1;
    }

    info!(
        "Clause extraction: {} created, {} skipped (empty or duplicate) from {} corpus docs",
        created,
        skipped,
        docs.len()
    );

    Ok(Json(json!({
        "created": created,
        "skipped": skipped,
        "total_analysed": docs.len(),
        "llm_model": &state.ollama_model
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

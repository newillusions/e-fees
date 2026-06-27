//! Clause domain models for the scope service.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb_types::Datetime;

/// A clause record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Clause {
    pub id: RecordId,
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub conditions: Option<surrealdb_types::Value>,
    pub sort_order: i64,
    pub tags: Option<Vec<String>>,
    pub is_default: bool,
    pub status: String,
    pub version: i64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Payload for creating a new clause.
#[derive(Debug, Deserialize)]
pub struct NewClause {
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub conditions: Option<serde_json::Value>,
    pub sort_order: i64,
    pub tags: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub is_default: bool,
}

fn default_true() -> bool {
    true
}

/// Payload for updating an existing clause (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateClause {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub sort_order: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_default: Option<bool>,
}

// ── Proposal Corpus models ──────────────────────────────────────────

/// A proposal corpus document as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProposalCorpus {
    pub id: RecordId,
    pub filename: String,
    pub project_number: Option<String>,
    pub project_name: Option<String>,
    pub extracted_text: String,
    pub sections: Option<surrealdb_types::Value>,
    pub metadata: Option<surrealdb_types::Value>,
    pub embedding: Option<Vec<f64>>,
    pub created_at: Datetime,
}

/// Extraction method for PDF ingestion.
#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMethod {
    /// Docling-Serve for text extraction (good for simple layouts).
    Docling,
    /// Multi-pass: Stirling PDF→PNG, Qwen3.5 vision, then text verification.
    #[default]
    Vision,
}

/// Payload for single PDF ingestion.
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub file_path: String,
    /// Optional project name override (otherwise parsed from filename).
    pub project_name: Option<String>,
    /// Extraction method (default: "vision" for multi-pass pipeline).
    #[serde(default)]
    pub method: ExtractionMethod,
}

/// Payload for batch ingestion from a directory.
#[derive(Debug, Deserialize)]
pub struct IngestBatchRequest {
    pub directory: String,
    /// Extraction method (default: "vision" for multi-pass pipeline).
    #[serde(default)]
    pub method: ExtractionMethod,
}

// ── Scope Assembly models ─────────────────────────────────────────────

/// A scope assembly record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ScopeAssembly {
    pub id: RecordId,
    pub fee_id: RecordId,
    pub clauses: surrealdb_types::Value,
    pub generated_text: String,
    pub numbering: Option<surrealdb_types::Value>,
    pub llm_model: Option<String>,
    pub llm_polished: bool,
    #[serde(default)]
    pub current_revision: i64,
    #[serde(default)]
    pub stages_snapshot: Option<Vec<String>>,
    /// Per-proposal clause selection (Stage 1).  When present, generate_scope
    /// uses only the `included=true` entries (with their `override_body`) instead
    /// of pulling every active clause from the library.
    #[serde(default)]
    pub selected_clauses: Option<surrealdb_types::Value>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Stage input from the fee's pricing data, passed by the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageInput {
    pub name: String,
    pub code: String,
    pub is_post_contract: bool,
    pub order: i64,
}

/// Payload for generating a scope assembly.
#[derive(Debug, Deserialize)]
pub struct GenerateScopeRequest {
    pub fee_id: String,
    #[serde(default)]
    pub polish: bool,
    #[serde(default)]
    pub stages: Option<Vec<StageInput>>,
}

/// Payload for manually updating a scope assembly.
#[derive(Debug, Deserialize)]
pub struct UpdateScopeRequest {
    pub generated_text: Option<String>,
    pub clauses: Option<serde_json::Value>,
}

// ── Stage Config models ──────────────────────────────────────────

/// A stage configuration record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct StageConfig {
    pub id: RecordId,
    pub canonical_name: String,
    pub default_label: String,
    pub aliases: Option<Vec<String>>,
    pub sort_order: i64,
    pub intro_text: Option<String>,
    pub status: String,
}

/// Payload for updating a stage config.
#[derive(Debug, Deserialize)]
pub struct UpdateStageConfig {
    pub default_label: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub sort_order: Option<i64>,
    pub intro_text: Option<String>,
}

// ── Deliverable models ───────────────────────────────────────────

/// A deliverable record as stored in SurrealDB.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Deliverable {
    pub id: RecordId,
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub layer: String,
    pub discipline: Option<String>,
    pub condition: Option<surrealdb_types::Value>,
    pub replaces: Option<RecordId>,
    pub sort_order: i64,
    pub source_proposals: Option<Vec<String>>,
    pub usage_history: Option<surrealdb_types::Value>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub version: i64,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Payload for creating a new deliverable.
#[derive(Debug, Deserialize)]
pub struct NewDeliverable {
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub layer: String,
    pub discipline: Option<String>,
    pub condition: Option<serde_json::Value>,
    pub replaces: Option<String>,
    pub sort_order: i64,
    pub source_proposals: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Payload for updating an existing deliverable (all fields optional).
#[derive(Debug, Deserialize)]
pub struct UpdateDeliverable {
    pub title: Option<String>,
    pub short_name: Option<String>,
    pub body: Option<String>,
    pub stage: Option<String>,
    pub layer: Option<String>,
    pub discipline: Option<String>,
    pub condition: Option<serde_json::Value>,
    pub replaces: Option<String>,
    pub sort_order: Option<i64>,
    pub source_proposals: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

/// Request to assemble deliverables for a fee.
#[derive(Debug, Deserialize)]
pub struct AssembleRequest {
    pub fee_id: String,
    /// Disciplines to include (e.g., ["lighting", "av"]).
    pub disciplines: Vec<String>,
    /// Project attributes for conditional matching (e.g., {"tool": "revit"}).
    pub conditions: Option<serde_json::Value>,
    /// Stages to include (canonical names). If empty, includes all active stages.
    pub stages: Option<Vec<String>>,
    /// Stage label overrides (e.g., {"schematic": "50% DD"}).
    pub stage_labels: Option<serde_json::Map<String, serde_json::Value>>,
    /// Whether to run LLM polish on the assembled text.
    #[serde(default)]
    pub polish: bool,
}

/// Request to save the scope builder state.
#[derive(Debug, Deserialize)]
pub struct SaveScopeBuilderRequest {
    pub fee_id: String,
    /// Array of deliverable references with optional wording overrides.
    pub deliverables: Vec<ScopeDeliverableEntry>,
    /// Custom one-off items not from the library.
    pub manual_items: Option<Vec<ManualDeliverableEntry>>,
    /// Stage label overrides for this proposal.
    pub stage_labels: Option<serde_json::Map<String, serde_json::Value>>,
    /// Whether to run LLM polish.
    #[serde(default)]
    pub polish: bool,
}

/// A deliverable entry in a saved scope assembly.
#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeDeliverableEntry {
    pub deliverable_id: String,
    pub stage: String,
    pub sort_order: i64,
    /// If set, this overrides the master wording for this proposal only.
    pub wording_override: Option<String>,
}

/// A manually added deliverable (not from library).
#[derive(Debug, Serialize, Deserialize)]
pub struct ManualDeliverableEntry {
    pub title: String,
    pub short_name: String,
    pub body: String,
    pub stage: String,
    pub sort_order: i64,
}

// ── Clause Selection models (Stage 1) ────────────────────────────────────────

/// A single clause selection entry for a per-proposal clause toggle + override.
///
/// `override_body` is a structural field: once set it is never silently removed.
/// Downstream code (generate_scope) uses `override_body` in preference to the
/// master clause body whenever it is `Some`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseSelectionEntry {
    pub clause_id: String,
    pub included: bool,
    /// Override body text — structural, preserved once set.
    pub override_body: Option<String>,
}

/// Payload for saving clause selections for a fee proposal.
///
/// Writes `selected_clauses` on the `scope_assembly` record (creating it if
/// it does not yet exist).  A subsequent `POST /scope/generate` will use these
/// selections instead of pulling all active clauses.
#[derive(Debug, Deserialize)]
pub struct SaveClauseSelectionRequest {
    pub fee_id: String,
    pub selections: Vec<ClauseSelectionEntry>,
}

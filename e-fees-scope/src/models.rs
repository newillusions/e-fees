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

/// Payload for single PDF ingestion.
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub file_path: String,
    /// Optional project name override (otherwise parsed from filename).
    pub project_name: Option<String>,
}

/// Payload for batch ingestion from a directory.
#[derive(Debug, Deserialize)]
pub struct IngestBatchRequest {
    pub directory: String,
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
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Payload for generating a scope assembly.
#[derive(Debug, Deserialize)]
pub struct GenerateScopeRequest {
    pub fee_id: String,
    #[serde(default)]
    pub polish: bool,
}

/// Payload for manually updating a scope assembly.
#[derive(Debug, Deserialize)]
pub struct UpdateScopeRequest {
    pub generated_text: Option<String>,
    pub clauses: Option<serde_json::Value>,
}

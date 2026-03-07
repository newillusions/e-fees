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

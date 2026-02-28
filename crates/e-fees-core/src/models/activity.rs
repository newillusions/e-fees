//! Activity log types.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb_types::Datetime;

/// Activity log entry for tracking user actions.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ActivityLog {
    pub id: Option<RecordId>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: String,
    pub timestamp: Datetime,
    pub metadata: Option<serde_json::Value>,
}

/// Create structure for new activity log entries.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ActivityLogCreate {
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

//! Common types, conversion helpers, and utility structures.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use surrealdb_types::{Datetime, Number as DbNumber, Value as DbValue};

// ============================================================================
// DBVALUE ↔ JSON CONVERSION
// ============================================================================
//
// SurrealDB v3 binary protocol sends native types (int, float, datetime, etc.)
// that serde_json::Value cannot handle directly. surrealdb_types::Value (DbValue)
// handles binary protocol natively via SurrealValue trait, BUT its Serialize
// implementation produces tagged JSON (e.g. {"Number":{"Int":150000}}) which
// breaks the frontend.
//
// Solution: Use DbValue in struct fields for binary protocol compatibility,
// but apply custom serde (de)serialization that converts to/from plain JSON.

/// Convert a surrealdb_types::Value to a plain serde_json::Value.
pub fn dbvalue_to_json(v: &DbValue) -> serde_json::Value {
    match v {
        DbValue::None | DbValue::Null => serde_json::Value::Null,
        DbValue::Bool(b) => serde_json::Value::Bool(*b),
        DbValue::Number(n) => match n {
            DbNumber::Int(i) => serde_json::Value::Number((*i).into()),
            DbNumber::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            DbNumber::Decimal(d) => {
                // Decimal → try as f64
                let s = d.to_string();
                s.parse::<f64>()
                    .ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(s))
            }
        },
        DbValue::String(s) => serde_json::Value::String(s.to_string()),
        DbValue::Array(arr) => serde_json::Value::Array(arr.iter().map(dbvalue_to_json).collect()),
        DbValue::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj
                .iter()
                .map(|(k, v)| (k.to_string(), dbvalue_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        DbValue::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        DbValue::RecordId(rid) => serde_json::Value::String(record_id_string(rid)),
        // Fallback for other types: use Debug representation
        other => serde_json::Value::String(format!("{:?}", other)),
    }
}

/// Convert a plain serde_json::Value to surrealdb_types::Value.
pub fn json_to_dbvalue(v: &serde_json::Value) -> DbValue {
    match v {
        serde_json::Value::Null => DbValue::None,
        serde_json::Value::Bool(b) => DbValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                DbValue::Number(DbNumber::Int(i))
            } else if let Some(f) = n.as_f64() {
                DbValue::Number(DbNumber::Float(f))
            } else {
                DbValue::None
            }
        }
        serde_json::Value::String(s) => DbValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            DbValue::Array(arr.iter().map(json_to_dbvalue).collect::<Vec<_>>().into())
        }
        serde_json::Value::Object(obj) => {
            let map: std::collections::BTreeMap<String, DbValue> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_dbvalue(v)))
                .collect();
            DbValue::Object(map.into())
        }
    }
}

// ============================================================================
// RECORD ID HELPERS (v3 RecordIdKey has no Display impl)
// ============================================================================

/// Convert a RecordIdKey to its string representation.
pub fn record_key_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        _ => format!("{:?}", key),
    }
}

/// Format a RecordId as "table:key".
pub fn record_id_string(id: &RecordId) -> String {
    format!("{}:{}", id.table, record_key_string(&id.key))
}

// ============================================================================
// COMMON STRUCTURES
// ============================================================================

/// Timestamp structure for created_at and updated_at.
/// Uses SurrealDB's native Datetime type for v3 binary protocol compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TimeStamps {
    #[cfg_attr(feature = "openapi", schema(value_type = String, example = "2026-01-15T10:30:00Z"))]
    pub created_at: Datetime,
    #[cfg_attr(feature = "openapi", schema(value_type = String, example = "2026-02-28T14:00:00Z"))]
    pub updated_at: Datetime,
}

// ============================================================================
// PAGINATION STRUCTURES
// ============================================================================

/// Paginated response structure for lazy loading.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaginatedResponse<T: SurrealValue> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}

impl<T: SurrealValue> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let has_more = page * page_size < total;
        Self {
            items,
            total,
            page,
            page_size,
            has_more,
        }
    }
}

/// Entity counts for Dashboard statistics.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct EntityCounts {
    pub total_projects: usize,
    pub total_companies: usize,
    pub total_contacts: usize,
    pub total_fees: usize,
    pub active_fees: usize,
}

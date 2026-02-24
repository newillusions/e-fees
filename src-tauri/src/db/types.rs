//! Database entity types and structures.

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};
use surrealdb_types::{Datetime, Value as DbValue, Number as DbNumber};

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
                s.parse::<f64>().ok()
                    .and_then(serde_json::Number::from_f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(s))
            }
        },
        DbValue::String(s) => serde_json::Value::String(s.to_string()),
        DbValue::Array(arr) => serde_json::Value::Array(
            arr.iter().map(dbvalue_to_json).collect()
        ),
        DbValue::Object(obj) => {
            let map: serde_json::Map<String, serde_json::Value> = obj.iter()
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
        serde_json::Value::String(s) => DbValue::String(s.clone().into()),
        serde_json::Value::Array(arr) => DbValue::Array(
            arr.iter().map(json_to_dbvalue).collect::<Vec<_>>().into()
        ),
        serde_json::Value::Object(obj) => {
            let map: std::collections::BTreeMap<String, DbValue> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_dbvalue(v)))
                .collect();
            DbValue::Object(map.into())
        }
    }
}

/// Serde module for Option<DbValue> fields that converts to/from plain JSON.
/// Use with `#[serde(default, with = "opt_dbvalue_json")]` on struct fields.
mod opt_dbvalue_json {
    use super::{DbValue, dbvalue_to_json, json_to_dbvalue};
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(val: &Option<DbValue>, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        match val {
            None => serializer.serialize_none(),
            Some(v) => dbvalue_to_json(v).serialize(serializer),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DbValue>, D::Error>
    where D: Deserializer<'de> {
        let opt: Option<serde_json::Value> = Option::deserialize(deserializer)?;
        Ok(opt.map(|v| json_to_dbvalue(&v)))
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
// ENTITY STRUCTURES
// ============================================================================

/// Project entity representing core business projects.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Project {
    pub id: Option<RecordId>,
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
    pub time: TimeStamps,
}

/// Project number structure implementing the YY-CCCNN numbering system.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProjectNumber {
    pub year: i32,
    pub country: i32,
    pub seq: i32,
    pub id: String,
}

/// Timestamp structure for created_at and updated_at.
/// Uses SurrealDB's native Datetime type for v3 binary protocol compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct TimeStamps {
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

/// Project creation struct without auto-managed fields.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct NewProject {
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub area: String,
    pub city: String,
    pub country: String,
    pub folder: String,
    pub number: ProjectNumber,
}

/// Company entity representing client organizations.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Company {
    pub id: Option<RecordId>,
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
    pub time: TimeStamps,
}

/// CompanyCreate represents a new company being created.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CompanyCreate {
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
    pub reg_no: Option<String>,
    pub tax_no: Option<String>,
}

/// Contact entity.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Contact {
    pub id: Option<RecordId>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub position: Option<String>,
    #[serde(default)]
    pub company: Option<RecordId>,
    #[serde(default)]
    pub time: Option<TimeStamps>,
}

/// Contact creation struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String,
}

/// Fee entity representing fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Fee {
    pub id: Option<RecordId>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub number: String,
    #[serde(default)]
    pub rev: i32,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub issue_date: String,
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub package: String,
    pub project_id: RecordId,
    pub company_id: RecordId,
    pub contact_id: RecordId,
    #[serde(default)]
    pub staff_name: String,
    #[serde(default)]
    pub staff_email: String,
    #[serde(default)]
    pub staff_phone: String,
    #[serde(default)]
    pub staff_position: String,
    #[serde(default)]
    pub strap_line: String,
    #[serde(default)]
    pub revisions: Vec<Revision>,
    pub time: TimeStamps,

    // Pricing fields — use DbValue for SurrealDB v3 binary protocol compatibility.
    // serde_json::Value CANNOT handle native SurrealDB types from binary protocol.
    // DbValue handles binary deserialization natively via SurrealValue trait.
    // Custom serde via opt_dbvalue_json converts to/from plain JSON for Tauri IPC.
    #[serde(default, skip_serializing_if = "Option::is_none", with = "opt_dbvalue_json")]
    pub pricing: Option<DbValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "opt_dbvalue_json")]
    pub payment_schedule: Option<DbValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "opt_dbvalue_json")]
    pub import_source: Option<DbValue>,
}

impl Fee {
    /// Convert the native DbValue pricing back to a typed PricingBreakdown.
    /// DbValue → plain JSON (via dbvalue_to_json) → PricingBreakdown (via serde),
    /// which handles int→float coercion automatically.
    pub fn pricing_typed(&self) -> Option<PricingBreakdown> {
        self.pricing.as_ref().and_then(|v| {
            let json = dbvalue_to_json(v);
            serde_json::from_value(json).ok()
        })
    }
}

/// Fee creation struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct FeeCreate {
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String,
    pub issue_date: String,
    pub activity: String,
    pub package: String,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    #[serde(default)]
    pub revisions: Vec<Revision>,

    // Pricing fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<PricingBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_schedule: Option<PaymentSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_source: Option<serde_json::Value>,
}

/// Fee update struct.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct FeeUpdate {
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String,
    #[serde(default)]
    pub issue_date: String,
    pub activity: Option<String>,
    pub package: Option<String>,
    pub project_id: String,
    pub company_id: String,
    pub contact_id: String,
    pub staff_name: Option<String>,
    pub staff_email: Option<String>,
    pub staff_phone: Option<String>,
    pub staff_position: Option<String>,
    pub strap_line: Option<String>,
    #[serde(default)]
    pub revisions: Vec<Revision>,

    // Pricing fields (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<PricingBreakdown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_contract_items: Option<Vec<PostContractItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_schedule: Option<PaymentSchedule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing_revisions: Option<Vec<PricingRevision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_revision_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_release_number: Option<i32>,
}

/// Revision entry for fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Revision {
    pub revision_number: i32,
    pub revision_date: String,
    pub author_email: String,
    pub author_name: String,
    pub notes: String,
}

// ============================================================================
// PRICING STRUCTURES
// ============================================================================

/// Discipline allocation for fee breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Discipline {
    pub id: String,
    pub name: String,
    pub percentage: f64,
    pub order: i32,
}

/// Design stage definition.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub code: String,
    pub percentage: f64,
    pub order: i32,
    pub is_post_contract: bool,
}

/// Pricing configuration.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
pub struct PricingConfig {
    #[serde(default)]
    pub target_fee: f64,
    #[serde(default)]
    pub buffer_percent: f64,
    #[serde(default)]
    pub quoted_fee: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub vat_percent: f64,
    #[serde(default)]
    pub vat_included: bool,
    #[serde(default)]
    pub mobilisation_percent: f64,
}

/// Discipline × Stage pricing cell.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PricingCell {
    pub discipline_id: String,
    pub stage_id: String,
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_amount: Option<f64>,
}

/// Reimbursable cost/expense.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ReimbursableCost {
    pub id: String,
    pub description: String,
    pub stage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discipline_id: Option<String>,
    pub base_cost: f64,
    pub markup_percent: f64,
    pub cost_to_client: f64,
    pub date_incurred: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Post-contract line item (different structure from design stages).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PostContractItem {
    pub id: String,
    pub stage_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    pub rate: f64,
    pub amount: f64,
}

/// Payment schedule entry.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaymentScheduleEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub payment_type: String,  // "mobilisation", "milestone", "final"
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_percentage: Option<f64>,
    pub amount: f64,
    pub percentage_of_total: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub status: String,  // "pending", "invoiced", "paid"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invoice_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paid_date: Option<String>,
}

/// Payment tracking summary.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaymentSchedule {
    pub entries: Vec<PaymentScheduleEntry>,
    pub total_invoiced: f64,
    pub total_paid: f64,
    pub total_outstanding: f64,
}

/// Complete pricing breakdown.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, Default)]
pub struct PricingBreakdown {
    #[serde(default)]
    pub config: PricingConfig,
    #[serde(default)]
    pub disciplines: Vec<Discipline>,
    #[serde(default)]
    pub stages: Vec<Stage>,
    #[serde(default)]
    pub cells: Vec<PricingCell>,
    #[serde(default)]
    pub costs: Vec<ReimbursableCost>,
    #[serde(default)]
    pub design_phase_total: f64,
    #[serde(default)]
    pub post_contract_total: f64,
    #[serde(default)]
    pub costs_total: f64,
    #[serde(default)]
    pub subtotal: f64,
    #[serde(default)]
    pub vat_amount: f64,
    #[serde(default)]
    pub grand_total: f64,
}

/// Pricing update struct - for updating only pricing-related fields on a fee.
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PricingUpdate {
    pub pricing: Option<PricingBreakdown>,
    pub post_contract_items: Option<Vec<PostContractItem>>,
    pub reimbursable_costs: Option<Vec<ReimbursableCost>>,
    pub payment_schedule: Option<PaymentSchedule>,
}

/// Pricing revision tracking (append-only).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PricingRevision {
    pub id: String,
    pub fee_id: String,
    pub revision_number: i32,
    pub created_at: String,
    pub created_by: String,
    pub change_summary: String,
    pub pricing_snapshot: PricingBreakdown,
    pub is_client_release: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_number: Option<i32>,
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
        Self { items, total, page, page_size, has_more }
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

// ============================================================================
// ACTIVITY LOG STRUCTURES
// ============================================================================

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

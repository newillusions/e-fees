//! Database entity types and structures.

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

// ============================================================================
// ENTITY STRUCTURES
// ============================================================================

/// Project entity representing core business projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<Thing>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNumber {
    pub year: i32,
    pub country: i32,
    pub seq: i32,
    pub id: String,
}

/// Timestamp structure for created_at and updated_at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStamps {
    pub created_at: String,
    pub updated_at: String,
}

/// Project creation struct without auto-managed fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Option<Thing>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Option<Thing>,
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
    pub company: Option<Thing>,
    #[serde(default)]
    pub time: Option<TimeStamps>,
}

/// Contact creation struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String,
}

/// Fee entity representing fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fee {
    pub id: Option<Thing>,
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String,
    pub issue_date: String,
    pub activity: String,
    pub package: String,
    pub project_id: Thing,
    pub company_id: Thing,
    pub contact_id: Thing,
    pub staff_name: String,
    pub staff_email: String,
    pub staff_phone: String,
    pub staff_position: String,
    pub strap_line: String,
    pub revisions: Vec<Revision>,
    pub time: TimeStamps,
}

/// Fee creation struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub revisions: Vec<Revision>,
}

/// Fee update struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeUpdate {
    pub name: String,
    pub number: String,
    pub rev: i32,
    pub status: String,
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
    pub revisions: Vec<Revision>,
}

/// Revision entry for fee proposals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub revision_number: i32,
    pub revision_date: String,
    pub author_email: String,
    pub author_name: String,
    pub notes: String,
}

// ============================================================================
// PAGINATION STRUCTURES
// ============================================================================

/// Paginated response structure for lazy loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, page_size: usize) -> Self {
        let has_more = page * page_size < total;
        Self { items, total, page, page_size, has_more }
    }
}

/// Entity counts for Dashboard statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Option<Thing>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub description: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub user: String,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}

/// Create structure for new activity log entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

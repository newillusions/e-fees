//! API-specific response types for OpenAPI documentation.
//!
//! These describe the JSON shapes returned by route handlers
//! (which use `Json<Value>`, not typed responses).

use serde::Serialize;
use utoipa::ToSchema;

/// Paginated list response envelope.
#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T: ToSchema> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

/// Single-item response envelope.
#[derive(Serialize, ToSchema)]
pub struct SingleResponse<T: ToSchema> {
    pub data: T,
}

/// Delete confirmation response.
#[derive(Serialize, ToSchema)]
pub struct DeleteResponse {
    pub deleted: bool,
    pub id: String,
}

/// Error response body.
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Dependency health status.
#[derive(Serialize, ToSchema)]
pub struct DependencyStatus {
    pub status: String,
    pub latency_ms: f64,
}

/// Health check response (Container Standards compliant).
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime: f64,
    pub checked_at: String,
    pub dependencies: std::collections::HashMap<String, DependencyStatus>,
}

/// Dashboard statistics.
#[derive(Serialize, ToSchema)]
pub struct StatsResponse {
    pub total_projects: u64,
    pub total_companies: u64,
    pub total_contacts: u64,
    pub total_fees: u64,
    pub active_fees: u64,
}

// --- Entity response schemas (matching the JSON output from *_to_json functions) ---

/// Project summary (as returned by list/detail endpoints).
#[derive(Serialize, ToSchema)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub name_short: String,
    pub status: String,
    pub country: String,
    pub city: String,
    pub area: String,
    pub folder: String,
    pub number: String,
}

/// Fee summary (as returned by list endpoint).
#[derive(Serialize, ToSchema)]
pub struct FeeSummaryResponse {
    pub id: String,
    pub name: String,
    pub number: String,
    pub rev: i64,
    pub status: String,
    pub project_id: String,
    pub company_id: String,
    pub total_fee: f64,
    pub currency: String,
}

/// Fee detail (as returned by single-item endpoint).
#[derive(Serialize, ToSchema)]
pub struct FeeDetailResponse {
    pub id: String,
    pub name: String,
    pub number: String,
    pub rev: i64,
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
    pub total_fee: f64,
    pub currency: String,
}

/// Company summary (as returned by list/detail endpoints).
#[derive(Serialize, ToSchema)]
pub struct CompanyResponse {
    pub id: String,
    pub name: String,
    pub name_short: String,
    pub abbreviation: String,
    pub city: String,
    pub country: String,
}

/// Contact summary (as returned by list/detail endpoints).
#[derive(Serialize, ToSchema)]
pub struct ContactResponse {
    pub id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub position: Option<String>,
    pub company_id: Option<String>,
}

/// Next available project number preview.
#[derive(Serialize, ToSchema)]
pub struct NextNumberResponse {
    pub number: String,
    pub year: u8,
    pub country_code: u16,
    pub seq: u8,
}

/// Folder creation success response.
#[derive(Serialize, ToSchema)]
pub struct FolderCreatedResponse {
    pub status: String,
    pub project: String,
    pub name: String,
    pub path: String,
}

/// Help/self-documentation response.
#[derive(Serialize, ToSchema)]
pub struct HelpResponse {
    pub service: String,
    pub version: String,
    pub description: String,
    pub config_file: String,
    pub endpoints: Vec<HelpEndpoint>,
}

/// Single endpoint in help response.
#[derive(Serialize, ToSchema)]
pub struct HelpEndpoint {
    pub method: String,
    pub path: String,
    pub description: String,
    pub auth: bool,
}

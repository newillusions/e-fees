use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
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

#[derive(Serialize, ToSchema)]
pub struct ClauseResponse {
    pub id: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub title: String,
    pub body: String,
    pub sort_order: i64,
    pub is_default: bool,
    pub status: String,
    pub version: i64,
}

#[derive(Serialize, ToSchema)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ScopeAssemblyResponse {
    pub fee_id: String,
    pub generated_text: String,
    pub llm_polished: bool,
    pub clause_count: u64,
}

#[derive(Serialize, ToSchema)]
pub struct CorpusDocResponse {
    pub id: String,
    pub filename: String,
    pub project_number: Option<String>,
    pub project_name: Option<String>,
    pub section_count: u64,
}

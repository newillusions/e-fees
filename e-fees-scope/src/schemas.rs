use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub database: String,
    pub ollama: String,
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

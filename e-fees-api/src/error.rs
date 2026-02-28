//! API error types and conversion helpers.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

/// Structured API error with HTTP status, machine-readable code, and human message.
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
}

impl ApiError {
    /// Create a 404 Not Found error with entity context.
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found".into(),
            message: format!("{} '{}' not found", entity, id),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            Json(json!({
                "error": self.code,
                "message": self.message
            })),
        )
            .into_response()
    }
}

impl From<surrealdb::Error> for ApiError {
    fn from(e: surrealdb::Error) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "database_error".into(),
            message: e.to_string(),
        }
    }
}

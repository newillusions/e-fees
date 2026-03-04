use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// Middleware that validates the `X-API-Key` header against configured API keys.
///
/// Checks the header value against a set of valid keys (supports multiple keys
/// for multi-service access). All endpoints require a valid API key.
///
/// # Returns
/// - `Ok(Response)` if the API key is valid
/// - `Err(StatusCode::UNAUTHORIZED)` if the API key is missing or invalid
pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match request.headers().get("X-API-Key") {
        Some(key) if state.api_keys.contains(key.to_str().unwrap_or("")) => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

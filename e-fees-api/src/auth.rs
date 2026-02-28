use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::AppState;

/// Middleware that validates the `X-API-Key` header against the configured API key.
///
/// Reads the API key from shared application state (validated at startup).
/// All endpoints require a valid API key.
///
/// # Returns
/// - `Ok(Response)` if the API key is valid
/// - `Err(StatusCode::UNAUTHORIZED)` if the API key is missing or invalid
pub async fn require_api_key(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Validate X-API-Key header against startup-validated key
    match request.headers().get("X-API-Key") {
        Some(key) if key.to_str().unwrap_or("") == state.api_key => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

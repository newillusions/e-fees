use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Middleware that validates the `X-API-Key` header against the configured API_KEY environment variable.
///
/// Skips authentication for the `/health` endpoint to allow monitoring without credentials.
/// All other endpoints require a valid API key.
///
/// # Returns
/// - `Ok(Response)` if the API key is valid or the endpoint is `/health`
/// - `Err(StatusCode::UNAUTHORIZED)` if the API key is missing or invalid
pub async fn require_api_key(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = std::env::var("API_KEY").unwrap_or_default();

    // Skip authentication for health endpoint
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Validate X-API-Key header
    match request.headers().get("X-API-Key") {
        Some(key) if key.to_str().unwrap_or("") == api_key => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use subtle::ConstantTimeEq;

use crate::AppState;

/// Constant-time membership check: whether `candidate` equals any key in
/// `configured`, without early-exiting on the first byte mismatch. A plain
/// `HashSet::contains`/`==` compare short-circuits, which leaks a timing
/// signal proportional to how many leading bytes of a guess match a real
/// key - irrelevant for a single local caller, but this middleware is
/// reachable over the network (see the CORS/bind posture fixed alongside
/// this), so every candidate is compared against every configured key in
/// full, and the results are OR'd together in constant time.
fn any_key_matches(configured: &std::collections::HashSet<String>, candidate: &str) -> bool {
    let candidate_bytes = candidate.as_bytes();
    configured
        .iter()
        .fold(subtle::Choice::from(0u8), |acc, key| {
            acc | key.as_bytes().ct_eq(candidate_bytes)
        })
        .into()
}

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
        Some(key) if any_key_matches(&state.api_keys, key.to_str().unwrap_or("")) => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use subtle::ConstantTimeEq;

use crate::AppState;

/// Constant-time membership check - see e-fees-api/src/auth.rs::any_key_matches
/// for the full rationale (duplicated here rather than shared, same reasoning
/// as validation.rs: no crate boundary exists between the two services).
fn any_key_matches(configured: &std::collections::HashSet<String>, candidate: &str) -> bool {
    let candidate_bytes = candidate.as_bytes();
    configured
        .iter()
        .fold(subtle::Choice::from(0u8), |acc, key| {
            acc | key.as_bytes().ct_eq(candidate_bytes)
        })
        .into()
}

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

//! Integration tests for e-fees-api.
//!
//! These tests run against a live API server and require:
//! - The API server running at `API_BASE_URL` (default: `http://localhost:3200`)
//! - `API_KEY` environment variable set to a valid API key
//! - `SURREAL_URL` must NOT point at the production database
//!
//! Run with: `cargo test -p e-fees-api --test integration_tests`

use reqwest::Client;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// SAFETY GUARD: Refuse to run if SURREAL_URL points at the production database.
fn verify_not_production() {
    let url = std::env::var("SURREAL_URL").unwrap_or_default();
    assert!(
        !url.contains("10.0.23.11"),
        "REFUSING TO RUN: Tests pointing at PRODUCTION database (10.0.23.11)"
    );
}

/// Read the API key from the environment.
fn api_key() -> String {
    std::env::var("API_KEY").expect("API_KEY must be set for integration tests")
}

/// Base URL for the running API server.
fn base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3200".to_string())
}

// ---------------------------------------------------------------------------
// Production safety
// ---------------------------------------------------------------------------

#[test]
fn test_production_safety_guard() {
    verify_not_production();
}

// ---------------------------------------------------------------------------
// Health endpoint (no auth required)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_no_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_health_response_format() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");

    assert!(body["status"].is_string(), "missing 'status' field");
    assert!(body["service"].is_string(), "missing 'service' field");
    assert!(body["version"].is_string(), "missing 'version' field");
    assert!(body["database"].is_string(), "missing 'database' field");

    assert_eq!(body["service"], "e-fees-api");
}

// ---------------------------------------------------------------------------
// Auth enforcement — all protected routes must reject requests without API key
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_projects_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_fees_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/fees", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_companies_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/companies", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_contacts_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/contacts", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_stats_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/stats", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

// ---------------------------------------------------------------------------
// Authenticated endpoint responses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_projects_with_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/projects", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(
        body["count"].is_number(),
        "response must have numeric 'count'"
    );
    assert!(
        body["count"].as_u64().unwrap_or(0) > 0,
        "expected at least one project"
    );
}

#[tokio::test]
async fn test_fees_with_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/fees", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(
        body["count"].is_number(),
        "response must have numeric 'count'"
    );
}

#[tokio::test]
async fn test_companies_with_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/companies", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
}

#[tokio::test]
async fn test_contacts_with_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/contacts", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
}

#[tokio::test]
async fn test_stats_with_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/stats", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(
        body["total_projects"].is_number(),
        "missing 'total_projects'"
    );
    assert!(
        body["total_companies"].is_number(),
        "missing 'total_companies'"
    );
    assert!(
        body["total_contacts"].is_number(),
        "missing 'total_contacts'"
    );
    assert!(body["total_fees"].is_number(), "missing 'total_fees'");
    assert!(body["active_fees"].is_number(), "missing 'active_fees'");
}

// ---------------------------------------------------------------------------
// 404 responses for nonexistent resources
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_nonexistent_project() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/projects/nonexistent_xyz", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_fee() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/fees/nonexistent_xyz", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_company() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/companies/nonexistent_xyz", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

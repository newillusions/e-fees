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

/// Build a reqwest client with the API key header pre-configured.
fn authed_client() -> Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", api_key().parse().unwrap());
    Client::builder().default_headers(headers).build().unwrap()
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
// Paginated list responses
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_projects_paginated_response() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(body["total"].is_number(), "response must have 'total'");
    assert!(body["page"].is_number(), "response must have 'page'");
    assert!(body["page_size"].is_number(), "response must have 'page_size'");
    assert!(body["total_pages"].is_number(), "response must have 'total_pages'");

    // Default page should be 1
    assert_eq!(body["page"], 1);
    // Default page_size should be 50
    assert_eq!(body["page_size"], 50);
    // Total should be > 0 (we have data in dev DB)
    assert!(body["total"].as_u64().unwrap_or(0) > 0, "expected at least one project");
}

#[tokio::test]
async fn test_fees_paginated_response() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/fees", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(body["total"].is_number(), "response must have 'total'");
    assert!(body["page"].is_number(), "response must have 'page'");
}

#[tokio::test]
async fn test_companies_paginated_response() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/companies", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(body["total"].is_number(), "response must have 'total'");
}

#[tokio::test]
async fn test_contacts_paginated_response() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/contacts", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["data"].is_array(), "response must have 'data' array");
    assert!(body["total"].is_number(), "response must have 'total'");
}

// ---------------------------------------------------------------------------
// Pagination query parameters
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_pagination_custom_page_size() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?page=1&page_size=2", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 2);
    // With page_size=2, data array should have at most 2 items
    assert!(body["data"].as_array().unwrap().len() <= 2);
    // total_pages should be calculated correctly
    let total = body["total"].as_u64().unwrap();
    let expected_pages = if total == 0 { 0 } else { (total + 1) / 2 };
    assert_eq!(body["total_pages"].as_u64().unwrap(), expected_pages);
}

#[tokio::test]
async fn test_pagination_page_size_clamped_to_max() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?page_size=999", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    // Max page_size is 100
    assert_eq!(body["page_size"], 100);
}

// ---------------------------------------------------------------------------
// Stats endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_stats_with_auth() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/stats", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(body["total_projects"].is_number(), "missing 'total_projects'");
    assert!(body["total_companies"].is_number(), "missing 'total_companies'");
    assert!(body["total_contacts"].is_number(), "missing 'total_contacts'");
    assert!(body["total_fees"].is_number(), "missing 'total_fees'");
    assert!(body["active_fees"].is_number(), "missing 'active_fees'");
}

// ---------------------------------------------------------------------------
// 404 responses for nonexistent resources
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_nonexistent_project() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_fee() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/fees/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_nonexistent_company() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/companies/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// CRUD lifecycle: Company (create → read → update → delete)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_company_crud_lifecycle() {
    verify_not_production();

    let client = authed_client();

    // CREATE
    let create_body = serde_json::json!({
        "name": "DELETE ME - Test Company API",
        "name_short": "DELETE ME - TC",
        "abbreviation": "DMTC",
        "city": "Dubai",
        "country": "UAE",
        "reg_no": null,
        "tax_no": null
    });

    let resp = client
        .post(format!("{}/companies", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create company");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let company_id = body["data"]["id"].as_str().unwrap().to_string();
    assert!(company_id.starts_with("company:"), "ID should have table prefix");
    assert_eq!(body["data"]["name"], "DELETE ME - Test Company API");

    // Extract just the key part (after "company:")
    let key = company_id.strip_prefix("company:").unwrap();

    // READ
    let resp = client
        .get(format!("{}/companies/{}", base_url(), key))
        .send()
        .await
        .expect("Failed to get company");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["name"], "DELETE ME - Test Company API");

    // UPDATE
    let update_body = serde_json::json!({
        "city": "Abu Dhabi"
    });

    let resp = client
        .put(format!("{}/companies/{}", base_url(), key))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update company");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["city"], "Abu Dhabi");
    // Name should be unchanged
    assert_eq!(body["data"]["name"], "DELETE ME - Test Company API");

    // DELETE
    let resp = client
        .delete(format!("{}/companies/{}", base_url(), key))
        .send()
        .await
        .expect("Failed to delete company");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["deleted"], true);

    // VERIFY DELETED
    let resp = client
        .get(format!("{}/companies/{}", base_url(), key))
        .send()
        .await
        .expect("Failed to verify deletion");

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// CRUD lifecycle: Contact (create → read → update → delete)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_contact_crud_lifecycle() {
    verify_not_production();

    let client = authed_client();

    // CREATE
    let create_body = serde_json::json!({
        "first_name": "DELETE ME",
        "last_name": "Test Contact",
        "email": "delete-me@example.com",
        "phone": "+971500000000",
        "position": "Test Position",
        "company": "company:nonexistent"
    });

    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create contact");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let contact_id = body["data"]["id"].as_str().unwrap().to_string();
    let key = contact_id.strip_prefix("contacts:").unwrap();

    // UPDATE
    let update_body = serde_json::json!({
        "position": "Updated Position"
    });

    let resp = client
        .put(format!("{}/contacts/{}", base_url(), key))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update contact");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["position"], "Updated Position");

    // DELETE
    let resp = client
        .delete(format!("{}/contacts/{}", base_url(), key))
        .send()
        .await
        .expect("Failed to delete contact");

    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["deleted"], true);
}

// ---------------------------------------------------------------------------
// Validation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_project_invalid_status() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "name": "DELETE ME - Bad Status",
        "name_short": "DM",
        "status": "InvalidStatus",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": 999, "id": "26-971999" }
    });

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn test_create_company_empty_name() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "name": "",
        "name_short": "X",
        "abbreviation": "X",
        "city": "Dubai",
        "country": "UAE"
    });

    let resp = client
        .post(format!("{}/companies", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
}

// ---------------------------------------------------------------------------
// OpenAPI / Swagger UI
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_openapi_spec_accessible() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/api-docs/openapi.json", base_url()))
        .send()
        .await
        .expect("Failed to fetch OpenAPI spec");

    assert_eq!(resp.status(), 200);

    let spec: serde_json::Value = resp.json().await.expect("OpenAPI spec must be valid JSON");
    assert_eq!(spec["info"]["title"], "E-Fees API");
    assert!(spec["paths"].is_object(), "spec must have paths");
    // Verify all entity paths exist
    assert!(spec["paths"]["/projects"].is_object(), "missing /projects path");
    assert!(spec["paths"]["/fees"].is_object(), "missing /fees path");
    assert!(spec["paths"]["/companies"].is_object(), "missing /companies path");
    assert!(spec["paths"]["/contacts"].is_object(), "missing /contacts path");
    assert!(spec["paths"]["/health"].is_object(), "missing /health path");
    assert!(spec["paths"]["/stats"].is_object(), "missing /stats path");
}

#[tokio::test]
async fn test_swagger_ui_accessible() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/docs/", base_url()))
        .send()
        .await
        .expect("Failed to fetch Swagger UI");

    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("swagger"), "Swagger UI HTML expected");
}

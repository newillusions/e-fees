//! Integration tests for e-fees-api.
//!
//! These tests run against a live API server and require:
//! - The API server running at `API_BASE_URL` (default: `http://localhost:3200`)
//! - `API_KEY` environment variable set to a valid API key
//! - `SURREAL_URL` must NOT point at the production database
//!
//! Run with: `cargo test -p e-fees-api --test integration_tests`

use chrono;
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
    assert!(body["version"].is_string(), "missing 'version' field");
    assert!(body["uptime"].is_number(), "missing 'uptime' field");
    assert!(body["checked_at"].is_string(), "missing 'checked_at' field");
    assert!(
        body["dependencies"].is_object(),
        "missing 'dependencies' field"
    );
}

#[tokio::test]
async fn test_health_has_uptime() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        body["uptime"].is_number(),
        "missing 'uptime' field (should be seconds as number)"
    );
    assert!(
        body["uptime"].as_f64().unwrap() >= 0.0,
        "uptime must be non-negative"
    );
}

#[tokio::test]
async fn test_health_has_checked_at() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["checked_at"].is_string(), "missing 'checked_at' field");
    let ts = body["checked_at"].as_str().unwrap();
    assert!(
        chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
        "checked_at must be RFC3339: got {}",
        ts
    );
}

#[tokio::test]
async fn test_health_has_dependencies() {
    verify_not_production();
    let client = Client::new();
    let body: serde_json::Value = client
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        body["dependencies"].is_object(),
        "missing 'dependencies' object"
    );
    assert!(
        body["dependencies"]["surrealdb"].is_object(),
        "missing 'surrealdb' dependency"
    );
    assert!(
        body["dependencies"]["surrealdb"]["status"].is_string(),
        "missing dependency status"
    );
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
    assert!(
        body["page_size"].is_number(),
        "response must have 'page_size'"
    );
    assert!(
        body["total_pages"].is_number(),
        "response must have 'total_pages'"
    );

    // Default page should be 1
    assert_eq!(body["page"], 1);
    // Default page_size should be 50
    assert_eq!(body["page_size"], 50);
    // Total should be > 0 (we have data in dev DB)
    assert!(
        body["total"].as_u64().unwrap_or(0) > 0,
        "expected at least one project"
    );
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
    assert!(
        company_id.starts_with("company:"),
        "ID should have table prefix"
    );
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
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["deleted"],
        true
    );
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
    assert!(
        spec["paths"]["/projects"].is_object(),
        "missing /projects path"
    );
    assert!(spec["paths"]["/fees"].is_object(), "missing /fees path");
    assert!(
        spec["paths"]["/companies"].is_object(),
        "missing /companies path"
    );
    assert!(
        spec["paths"]["/contacts"].is_object(),
        "missing /contacts path"
    );
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

// ---------------------------------------------------------------------------
// CRUD lifecycle: Project (create → read → update → delete)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_project_crud_lifecycle() {
    verify_not_production();

    let client = authed_client();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Use a unique project number to avoid collisions
    let seq = (ts % 900 + 100) as i64; // 100-999
    let project_id_str = format!("26-971{}", seq);
    let expected_key = project_id_str.replace('-', "_");

    // CREATE
    let create_body = serde_json::json!({
        "name": "DELETE ME - Test Project API",
        "name_short": "DELETE ME - TP",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": seq, "id": project_id_str }
    });

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create project");

    assert_eq!(
        resp.status(),
        200,
        "create project failed: {:?}",
        resp.text().await
    );
    let body: serde_json::Value = client
        .get(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["data"]["name"], "DELETE ME - Test Project API");
    assert_eq!(body["data"]["status"], "Lead");
    assert_eq!(body["data"]["city"], "Dubai");

    // UPDATE
    let update_body = serde_json::json!({
        "status": "RFP",
        "city": "Abu Dhabi"
    });

    let resp = client
        .put(format!("{}/projects/{}", base_url(), expected_key))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update project");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["status"], "RFP");
    assert_eq!(body["data"]["city"], "Abu Dhabi");
    // Name should be unchanged
    assert_eq!(body["data"]["name"], "DELETE ME - Test Project API");

    // DELETE
    let resp = client
        .delete(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .expect("Failed to delete project");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["deleted"], true);

    // VERIFY DELETED
    let resp = client
        .get(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .expect("Failed to verify deletion");

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// CRUD lifecycle: Fee (create → read → update → delete)
// Fees require a project, company, and contact, so we create those first.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fee_crud_lifecycle() {
    verify_not_production();

    let client = authed_client();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // 1. Create prerequisite: Company
    let company_body = serde_json::json!({
        "name": "DELETE ME - Fee Test Company",
        "name_short": "DELETE ME - FTC",
        "abbreviation": "DMFTC",
        "city": "Dubai",
        "country": "UAE",
        "reg_no": null,
        "tax_no": null
    });

    let resp = client
        .post(format!("{}/companies", base_url()))
        .json(&company_body)
        .send()
        .await
        .expect("Failed to create prerequisite company");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let company_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let company_key = company_full_id
        .strip_prefix("company:")
        .unwrap()
        .to_string();

    // 2. Create prerequisite: Contact
    let contact_body = serde_json::json!({
        "first_name": "DELETE ME",
        "last_name": "Fee Test Contact",
        "email": "delete-me-fee@example.com",
        "phone": "+971500000001",
        "position": "Test",
        "company": company_key
    });

    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&contact_body)
        .send()
        .await
        .expect("Failed to create prerequisite contact");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let contact_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let contact_key = contact_full_id
        .strip_prefix("contacts:")
        .unwrap()
        .to_string();

    // 3. Create prerequisite: Project
    let seq = (ts % 900 + 100) as i64;
    let project_number_id = format!("26-971{}", seq);
    let project_key = project_number_id.replace('-', "_");

    let project_body = serde_json::json!({
        "name": "DELETE ME - Fee Test Project",
        "name_short": "DELETE ME - FTP",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": seq, "id": project_number_id }
    });

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&project_body)
        .send()
        .await
        .expect("Failed to create prerequisite project");
    assert_eq!(resp.status(), 200);

    // 4. Create Fee
    // Note: issue_date must be YYYYMM format (6-digit numeric string per DB ASSERT)
    let fee_body = serde_json::json!({
        "name": "DELETE ME - Test Fee API",
        "number": "FP-001",
        "rev": 1,
        "status": "Draft",
        "issue_date": "202603",
        "activity": "Lighting Design",
        "package": "Full Scope",
        "project_id": project_number_id,
        "company_id": company_key,
        "contact_id": contact_key,
        "staff_name": "Test Staff",
        "staff_email": "test@emittiv.com",
        "staff_phone": "+971500000002",
        "staff_position": "Associate",
        "strap_line": "Professional Lighting Consultancy",
        "revisions": []
    });

    let resp = client
        .post(format!("{}/fees", base_url()))
        .json(&fee_body)
        .send()
        .await
        .expect("Failed to create fee");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "create fee failed: {}", body);
    let fee_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let fee_key = fee_full_id.strip_prefix("fee:").unwrap().to_string();

    assert_eq!(body["data"]["name"], "DELETE ME - Test Fee API");
    assert_eq!(body["data"]["status"], "Draft");

    // READ
    let resp = client
        .get(format!("{}/fees/{}", base_url(), fee_key))
        .send()
        .await
        .expect("Failed to get fee");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["name"], "DELETE ME - Test Fee API");
    assert_eq!(body["data"]["activity"], "Lighting Design");

    // UPDATE
    let update_body = serde_json::json!({
        "status": "Sent",
        "activity": "Updated Lighting Design"
    });

    let resp = client
        .put(format!("{}/fees/{}", base_url(), fee_key))
        .json(&update_body)
        .send()
        .await
        .expect("Failed to update fee");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["status"], "Sent");

    // DELETE fee
    let resp = client
        .delete(format!("{}/fees/{}", base_url(), fee_key))
        .send()
        .await
        .expect("Failed to delete fee");
    assert_eq!(resp.status(), 200);

    // VERIFY DELETED
    let resp = client
        .get(format!("{}/fees/{}", base_url(), fee_key))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);

    // Cleanup prerequisites (reverse order)
    client
        .delete(format!("{}/projects/{}", base_url(), project_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/contacts/{}", base_url(), contact_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/companies/{}", base_url(), company_key))
        .send()
        .await
        .ok();
}

// ---------------------------------------------------------------------------
// GET /projects/{id}?include=client — embeds latest non-superseded fee's
// company as a nested `client: { id, name }` object on the response.
// Consumer: cad-export (avoids a second-hop fees+companies fetch at preflight).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_project_with_include_client() {
    verify_not_production();

    let client = authed_client();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Seed: company → contact → project → fee (linking project to company)
    let company_body = serde_json::json!({
        "name": "DELETE ME - Include Client Co",
        "name_short": "DELETE ME - ICC",
        "abbreviation": "DMICC",
        "city": "Dubai",
        "country": "UAE",
        "reg_no": null,
        "tax_no": null
    });
    let resp = client
        .post(format!("{}/companies", base_url()))
        .json(&company_body)
        .send()
        .await
        .expect("seed company");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let company_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let company_key = company_full_id.strip_prefix("company:").unwrap().to_string();

    let contact_body = serde_json::json!({
        "first_name": "DELETE ME",
        "last_name": "Include Client",
        "email": "delete-me-include@example.com",
        "phone": "+971500000003",
        "position": "Test",
        "company": company_key
    });
    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&contact_body)
        .send()
        .await
        .expect("seed contact");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let contact_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let contact_key = contact_full_id.strip_prefix("contacts:").unwrap().to_string();

    let seq = (ts % 900 + 100) as i64;
    let project_number_id = format!("26-971{}", seq);
    let project_key = project_number_id.replace('-', "_");

    let project_body = serde_json::json!({
        "name": "DELETE ME - Include Client Project",
        "name_short": "DELETE ME - ICP",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": seq, "id": project_number_id }
    });
    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&project_body)
        .send()
        .await
        .expect("seed project");
    assert_eq!(resp.status(), 200);

    let fee_body = serde_json::json!({
        "name": "DELETE ME - Include Client Fee",
        "number": "FP-IC1",
        "rev": 1,
        "status": "Draft",
        "issue_date": "202603",
        "activity": "Lighting Design",
        "package": "Full Scope",
        "project_id": project_number_id,
        "company_id": company_key,
        "contact_id": contact_key,
        "staff_name": "Test Staff",
        "staff_email": "test@emittiv.com",
        "staff_phone": "+971500000004",
        "staff_position": "Associate",
        "strap_line": "Professional Lighting Consultancy",
        "revisions": []
    });
    let resp = client
        .post(format!("{}/fees", base_url()))
        .json(&fee_body)
        .send()
        .await
        .expect("seed fee");
    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "seed fee failed: {}", body);
    let fee_full_id = body["data"]["id"].as_str().unwrap().to_string();
    let fee_key = fee_full_id.strip_prefix("fee:").unwrap().to_string();

    // 1) GET without ?include → no client field
    let resp = client
        .get(format!("{}/projects/{}", base_url(), project_key))
        .send()
        .await
        .expect("GET project (no include)");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"].get("client").is_none(),
        "without ?include=client the response must not contain a `client` field, got: {}",
        body
    );

    // 2) GET with ?include=client → nested client { id, name } object
    let resp = client
        .get(format!(
            "{}/projects/{}?include=client",
            base_url(),
            project_key
        ))
        .send()
        .await
        .expect("GET project (include=client)");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["data"]["client"]["id"], company_full_id,
        "client.id must match the seeded company record-id, got: {}",
        body
    );
    assert_eq!(
        body["data"]["client"]["name"], "DELETE ME - Include Client Co",
        "client.name must match the seeded company name, got: {}",
        body
    );

    // 3) GET with ?include=client on a project with NO fees → client: null
    let bare_seq = ((ts + 1) % 900 + 100) as i64;
    let bare_number_id = format!("26-971{}", bare_seq);
    let bare_key = bare_number_id.replace('-', "_");
    let bare_body = serde_json::json!({
        "name": "DELETE ME - Bare Project (no fees)",
        "name_short": "DELETE ME - BP",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": bare_seq, "id": bare_number_id }
    });
    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&bare_body)
        .send()
        .await
        .expect("seed bare project");
    assert_eq!(resp.status(), 200);
    let resp = client
        .get(format!(
            "{}/projects/{}?include=client",
            base_url(),
            bare_key
        ))
        .send()
        .await
        .expect("GET bare project (include=client)");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"]["client"].is_null(),
        "bare project with no fees must return client: null, got: {}",
        body
    );

    // Cleanup (reverse order)
    client
        .delete(format!("{}/projects/{}", base_url(), bare_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/fees/{}", base_url(), fee_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/projects/{}", base_url(), project_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/contacts/{}", base_url(), contact_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/companies/{}", base_url(), company_key))
        .send()
        .await
        .ok();
}

// ---------------------------------------------------------------------------
// 404 on nonexistent contact
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_nonexistent_contact() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/contacts/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// 404 on delete/update of nonexistent resources
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_delete_nonexistent_company() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .delete(format!("{}/companies/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_delete_nonexistent_project() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .delete(format!("{}/projects/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_delete_nonexistent_fee() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .delete(format!("{}/fees/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_delete_nonexistent_contact() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .delete(format!("{}/contacts/nonexistent_xyz", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_nonexistent_company() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .put(format!("{}/companies/nonexistent_xyz", base_url()))
        .json(&serde_json::json!({"city": "Dubai"}))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_nonexistent_project() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .put(format!("{}/projects/nonexistent_xyz", base_url()))
        .json(&serde_json::json!({"city": "Dubai"}))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_nonexistent_fee() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .put(format!("{}/fees/nonexistent_xyz", base_url()))
        .json(&serde_json::json!({"status": "Draft"}))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_nonexistent_contact() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .put(format!("{}/contacts/nonexistent_xyz", base_url()))
        .json(&serde_json::json!({"position": "Test"}))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// Additional validation tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_contact_empty_first_name() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "first_name": "",
        "last_name": "Contact",
        "email": "test@example.com",
        "phone": "+971500000000",
        "position": "Test",
        "company": "HOL"
    });

    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("first_name"));
}

#[tokio::test]
async fn test_create_contact_empty_last_name() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "first_name": "Test",
        "last_name": "",
        "email": "test@example.com",
        "phone": "+971500000000",
        "position": "Test",
        "company": "HOL"
    });

    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("last_name"));
}

#[tokio::test]
async fn test_create_fee_empty_name() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "name": "",
        "number": "FP-001",
        "rev": 1,
        "status": "Draft",
        "issue_date": "202603",
        "activity": "",
        "package": "",
        "project_id": "26-97100",
        "company_id": "HOL",
        "contact_id": "test",
        "staff_name": "",
        "staff_email": "",
        "staff_phone": "",
        "staff_position": "",
        "strap_line": "",
        "revisions": []
    });

    let resp = client
        .post(format!("{}/fees", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("name"));
}

#[tokio::test]
async fn test_create_fee_invalid_status() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "name": "DELETE ME - Bad Fee",
        "number": "FP-001",
        "rev": 1,
        "status": "InvalidFeeStatus",
        "issue_date": "202603",
        "activity": "",
        "package": "",
        "project_id": "26-97100",
        "company_id": "HOL",
        "contact_id": "test",
        "staff_name": "",
        "staff_email": "",
        "staff_phone": "",
        "staff_position": "",
        "strap_line": "",
        "revisions": []
    });

    let resp = client
        .post(format!("{}/fees", base_url()))
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
async fn test_update_fee_invalid_status() {
    verify_not_production();

    let client = authed_client();

    // Try to update an existing fee (or nonexistent — validation runs before DB lookup)
    let body = serde_json::json!({
        "status": "BogusStatus"
    });

    let resp = client
        .put(format!("{}/fees/nonexistent_xyz", base_url()))
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
async fn test_update_project_invalid_status() {
    verify_not_production();

    let client = authed_client();

    let body = serde_json::json!({
        "status": "BogusStatus"
    });

    let resp = client
        .put(format!("{}/projects/nonexistent_xyz", base_url()))
        .json(&body)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
    assert!(body["message"].as_str().unwrap().contains("Invalid"));
}

// ---------------------------------------------------------------------------
// PATCH /projects/{id} — pa-core sends PATCH to overwrite folder field after
// auto-derivation. Must accept the same body shape as PUT (partial merge).
// Regression for hub message:moox1tgdtqqoosbr033k (2026-05-06 Bermuda Beach).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_patch_project_accepts_folder_field() {
    verify_not_production();

    let client = authed_client();

    // Create a throwaway project so PATCH has something real to merge into.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let create_body = serde_json::json!({
        "name": format!("DELETE ME - PATCH Test {}", ts),
        "country": "UAE",
        "city": "Dubai",
        "status": "Lead"
    });

    let create_resp = client
        .post(format!("{}/projects", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create project");
    assert_eq!(create_resp.status(), 200);
    let created: serde_json::Value = create_resp.json().await.unwrap();
    let key = created["data"]["number"].as_str().unwrap().replace('-', "_");

    // PATCH overrides the auto-derived folder field — pa-core's exact use case.
    let patch_body = serde_json::json!({
        "folder": format!("{}-Bermuda-Beach", &key.replace('_', "-"))
    });

    let resp = client
        .patch(format!("{}/projects/{}", base_url(), key))
        .json(&patch_body)
        .send()
        .await
        .expect("Failed to PATCH project");

    assert_eq!(
        resp.status(),
        200,
        "PATCH /projects/{{id}} must return 200, not 405 (method not allowed)"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["data"]["folder"]
            .as_str()
            .unwrap()
            .ends_with("-Bermuda-Beach"),
        "folder field should be merged: {:?}",
        body["data"]["folder"]
    );

    // Cleanup
    let _ = client
        .delete(format!("{}/projects/{}", base_url(), key))
        .send()
        .await;
}

#[tokio::test]
async fn test_patch_project_method_allowed() {
    // Regression test: ensure PATCH is registered for /projects/{id}.
    // A 405 here means the PATCH route was not added back (axum returns 405,
    // not 404, when the path matches but the method doesn't).
    verify_not_production();

    let client = authed_client();
    let resp = client
        .patch(format!("{}/projects/nonexistent_xyz", base_url()))
        .json(&serde_json::json!({"folder": "noop"}))
        .send()
        .await
        .expect("Failed to send PATCH");

    assert_ne!(
        resp.status(),
        405,
        "PATCH /projects/{{id}} returned 405 — the route is missing the .patch() handler"
    );
    // Expected: 404 (project doesn't exist), not 405.
    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// OpenAPI spec completeness — verify all CRUD operations are documented
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_openapi_spec_has_all_operations() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .get(format!("{}/api-docs/openapi.json", base_url()))
        .send()
        .await
        .expect("Failed to fetch OpenAPI spec");

    let spec: serde_json::Value = resp.json().await.unwrap();

    // Each entity path should have get+post (list+create)
    for path in ["/projects", "/fees", "/companies", "/contacts"] {
        assert!(
            spec["paths"][path]["get"].is_object(),
            "{} missing GET (list)",
            path
        );
        assert!(
            spec["paths"][path]["post"].is_object(),
            "{} missing POST (create)",
            path
        );
    }

    // Each entity/{id} path should have get+put+delete
    for path in [
        "/projects/{id}",
        "/fees/{id}",
        "/companies/{id}",
        "/contacts/{id}",
    ] {
        assert!(
            spec["paths"][path]["get"].is_object(),
            "{} missing GET (read)",
            path
        );
        assert!(
            spec["paths"][path]["put"].is_object(),
            "{} missing PUT (update)",
            path
        );
        assert!(
            spec["paths"][path]["delete"].is_object(),
            "{} missing DELETE",
            path
        );
    }
}

// ---------------------------------------------------------------------------
// Auth: wrong API key should be rejected
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_wrong_api_key_rejected() {
    verify_not_production();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", "wrong-key-12345".parse().unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

// ---------------------------------------------------------------------------
// Response field validation — verify list items have expected fields
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_project_response_fields() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    if let Some(first) = body["data"].as_array().and_then(|a| a.first()) {
        assert!(first["id"].is_string(), "project missing 'id'");
        assert!(first["name"].is_string(), "project missing 'name'");
        assert!(first["status"].is_string(), "project missing 'status'");
        assert!(first["country"].is_string(), "project missing 'country'");
        assert!(first["number"].is_string(), "project missing 'number'");
    }
}

#[tokio::test]
async fn test_company_response_fields() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/companies?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    if let Some(first) = body["data"].as_array().and_then(|a| a.first()) {
        assert!(first["id"].is_string(), "company missing 'id'");
        assert!(first["name"].is_string(), "company missing 'name'");
        assert!(first["city"].is_string(), "company missing 'city'");
        assert!(first["country"].is_string(), "company missing 'country'");
    }
}

#[tokio::test]
async fn test_fee_response_fields() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/fees?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    if let Some(first) = body["data"].as_array().and_then(|a| a.first()) {
        assert!(first["id"].is_string(), "fee missing 'id'");
        assert!(first["name"].is_string(), "fee missing 'name'");
        assert!(first["status"].is_string(), "fee missing 'status'");
        assert!(first["project_id"].is_string(), "fee missing 'project_id'");
        assert!(first["company_id"].is_string(), "fee missing 'company_id'");
    }
}

#[tokio::test]
async fn test_contact_response_fields() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/contacts?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    if let Some(first) = body["data"].as_array().and_then(|a| a.first()) {
        assert!(first["id"].is_string(), "contact missing 'id'");
        assert!(
            first["first_name"].is_string(),
            "contact missing 'first_name'"
        );
        assert!(
            first["last_name"].is_string(),
            "contact missing 'last_name'"
        );
    }
}

// ===========================================================================
// Phase 1: Multiple API Keys
// ===========================================================================

#[tokio::test]
async fn test_auth_invalid_key_rejected() {
    verify_not_production();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", "definitely-not-a-valid-key".parse().unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_auth_empty_key_rejected() {
    verify_not_production();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("X-API-Key", "".parse().unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_auth_valid_key_accepted() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
}

// ===========================================================================
// Phase 2: Filter Parameters
// ===========================================================================

#[tokio::test]
async fn test_projects_filter_by_status() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?status=Lead", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    // All returned projects must have status "Lead"
    if let Some(arr) = body["data"].as_array() {
        for project in arr {
            assert_eq!(project["status"], "Lead", "filter returned wrong status");
        }
    }
}

#[tokio::test]
async fn test_projects_filter_invalid_status_400() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects?status=Bogus", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("Invalid"));
}

#[tokio::test]
async fn test_projects_no_filter_unchanged() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["data"].is_array());
    assert!(body["total"].is_u64());
    assert!(body["page"].is_u64());
}

#[tokio::test]
async fn test_companies_filter_by_name() {
    verify_not_production();

    let client = authed_client();

    // Create a company to search for
    let create_body = serde_json::json!({
        "name": "DELETE ME - Filter Test Co",
        "name_short": "DELETE ME - FTC",
        "abbreviation": "DMFTC",
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
    let created: serde_json::Value = resp.json().await.unwrap();
    let company_id = created["data"]["id"].as_str().unwrap().to_string();
    let key = company_id.strip_prefix("company:").unwrap();

    // Search with case-insensitive substring
    let resp = client
        .get(format!("{}/companies?name=filter+test", base_url()))
        .send()
        .await
        .expect("Failed to filter companies");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let matches = body["data"].as_array().unwrap();
    assert!(
        matches
            .iter()
            .any(|c| c["name"].as_str().unwrap().contains("Filter Test")),
        "filter should find the created company"
    );

    // Cleanup
    client
        .delete(format!("{}/companies/{}", base_url(), key))
        .send()
        .await
        .expect("Failed to cleanup");
}

#[tokio::test]
async fn test_contacts_filter_by_company() {
    verify_not_production();

    let client = authed_client();

    // Create a company, then a contact linked to it
    let company_body = serde_json::json!({
        "name": "DELETE ME - Contact Filter Co",
        "name_short": "DELETE ME - CFC",
        "abbreviation": "DMCFC",
        "city": "Dubai",
        "country": "UAE",
        "reg_no": null,
        "tax_no": null
    });

    let resp = client
        .post(format!("{}/companies", base_url()))
        .json(&company_body)
        .send()
        .await
        .expect("Failed to create company");
    assert_eq!(resp.status(), 200);
    let company_created: serde_json::Value = resp.json().await.unwrap();
    let company_full_id = company_created["data"]["id"].as_str().unwrap().to_string();
    let company_key = company_full_id
        .strip_prefix("company:")
        .unwrap()
        .to_string();

    // Create contact linked to that company
    let contact_body = serde_json::json!({
        "first_name": "DELETE ME",
        "last_name": "Contact Filter Test",
        "email": "delete-me-filter@example.com",
        "phone": "+971500000001",
        "position": "Test",
        "company": &company_key
    });

    let resp = client
        .post(format!("{}/contacts", base_url()))
        .json(&contact_body)
        .send()
        .await
        .expect("Failed to create contact");
    assert_eq!(resp.status(), 200);
    let contact_created: serde_json::Value = resp.json().await.unwrap();
    let contact_full_id = contact_created["data"]["id"].as_str().unwrap().to_string();
    let contact_key = contact_full_id
        .strip_prefix("contacts:")
        .unwrap_or(&contact_full_id)
        .to_string();

    // Filter contacts by company key
    let resp = client
        .get(format!("{}/contacts?company={}", base_url(), company_key))
        .send()
        .await
        .expect("Failed to filter contacts");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let matches = body["data"].as_array().unwrap();
    assert!(
        matches.iter().any(|c| c["company_id"]
            .as_str()
            .unwrap_or("")
            .contains(&company_key)),
        "filter should return contacts for that company"
    );

    // Cleanup
    client
        .delete(format!("{}/contacts/{}", base_url(), contact_key))
        .send()
        .await
        .ok();
    client
        .delete(format!("{}/companies/{}", base_url(), company_key))
        .send()
        .await
        .ok();
}

#[tokio::test]
async fn test_filter_with_pagination() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!(
            "{}/projects?status=Lead&page=1&page_size=5",
            base_url()
        ))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["data"].is_array());
    assert_eq!(body["page"], 1);
    assert_eq!(body["page_size"], 5);
}

#[tokio::test]
async fn test_empty_filter_result() {
    verify_not_production();

    let client = authed_client();
    // "Superseded" is a valid status but likely has 0 results
    let resp = client
        .get(format!(
            "{}/projects?status=Superseded&page=999",
            base_url()
        ))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

// ===========================================================================
// Phase 3: Project Number Auto-Assignment
// ===========================================================================

#[tokio::test]
async fn test_next_number_uae() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/next-number?country=UAE", base_url()))
        .send()
        .await
        .expect("Failed to get next number");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    let number = body["number"].as_str().unwrap();
    // Format: YY-CCCNN  e.g. "26-97105"
    assert!(number.len() >= 7, "number too short: {}", number);
    assert!(number.contains("-"), "number missing dash: {}", number);
    assert!(
        number.contains("971"),
        "UAE number must contain 971: {}",
        number
    );

    assert!(body["year"].is_u64());
    assert_eq!(body["country_code"], 971);
    assert!(body["seq"].is_u64());
}

#[tokio::test]
async fn test_next_number_invalid_country_400() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!(
            "{}/projects/next-number?country=Narnia",
            base_url()
        ))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_create_project_auto_number() {
    verify_not_production();

    let client = authed_client();

    // Create project WITHOUT number — should be auto-assigned
    let create_body = serde_json::json!({
        "name": "DELETE ME - Auto Number Test",
        "name_short": "DELETE ME - ANT",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": ""
    });

    // First, get next-number to know what ID will be created
    let next_resp = client
        .get(format!("{}/projects/next-number?country=UAE", base_url()))
        .send()
        .await
        .expect("Failed to get next number");
    let next_body: serde_json::Value = next_resp.json().await.unwrap();
    let expected_number = next_body["number"].as_str().unwrap().to_string();
    let expected_key = expected_number.replace('-', "_");

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create project");

    assert_eq!(
        resp.status(),
        200,
        "auto-number create failed: {:?}",
        resp.text().await
    );

    // Verify the project was created with the expected auto-number
    let resp = client
        .get(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .expect("Failed to get created project");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let number = body["data"]["number"].as_str().unwrap();
    assert!(
        number.contains("971"),
        "auto-number should use UAE dial code 971"
    );
    assert!(
        number.contains("-"),
        "auto-number should have YY-CCCNN format"
    );
    assert_eq!(number, expected_number);

    // Cleanup
    client
        .delete(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .ok();
}

#[tokio::test]
async fn test_create_project_explicit_number_backward_compat() {
    verify_not_production();

    let client = authed_client();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let seq = (ts % 900 + 100) as i64;
    let project_id_str = format!("26-971{}", seq);
    let expected_key = project_id_str.replace('-', "_");

    let create_body = serde_json::json!({
        "name": "DELETE ME - Explicit Number Test",
        "name_short": "DELETE ME - ENT",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": seq, "id": project_id_str }
    });

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create project");

    assert_eq!(
        resp.status(),
        200,
        "explicit number create failed: {:?}",
        resp.text().await
    );

    // Verify the project exists at the expected key
    let resp = client
        .get(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .expect("Failed to get project");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["data"]["number"], project_id_str);

    // Cleanup
    client
        .delete(format!("{}/projects/{}", base_url(), expected_key))
        .send()
        .await
        .ok();
}

#[tokio::test]
async fn test_next_number_with_explicit_year() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .get(format!(
            "{}/projects/next-number?country=UAE&year=25",
            base_url()
        ))
        .send()
        .await
        .expect("Failed to get next number");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["year"], 25);
    let number = body["number"].as_str().unwrap();
    assert!(
        number.starts_with("25-"),
        "number should start with year 25: {}",
        number
    );
}

// ---------------------------------------------------------------------------
// Folder creation endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_folder_requires_auth() {
    verify_not_production();

    let client = Client::new();
    let resp = client
        .post(format!("{}/projects/26_97101/folder", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_folder_nonexistent_project() {
    verify_not_production();

    let client = authed_client();
    let resp = client
        .post(format!("{}/projects/99_00099/folder", base_url()))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "not_found");
}

#[tokio::test]
async fn test_create_folder_returns_expected_fields() {
    verify_not_production();

    let client = authed_client();

    // Use first existing project (avoids SurrealDB v3 write issues in test)
    let list_resp = client
        .get(format!("{}/projects?page_size=1", base_url()))
        .send()
        .await
        .expect("Failed to list projects");
    assert_eq!(list_resp.status(), 200);
    let list: serde_json::Value = list_resp.json().await.unwrap();
    let projects = list["data"].as_array().expect("data should be array");
    assert!(
        !projects.is_empty(),
        "Need at least one project in DB for this test"
    );

    let project_id = projects[0]["id"].as_str().unwrap();
    let key = project_id.strip_prefix("projects:").unwrap();

    // Try to create folder — may fail with 503 if SSH not configured, that's OK
    let resp = client
        .post(format!("{}/projects/{}/folder", base_url(), key))
        .send()
        .await
        .expect("Failed to send folder request");

    let status = resp.status().as_u16();
    let body: serde_json::Value = resp.json().await.unwrap();

    if status == 200 {
        // Full success — SSH worked
        assert_eq!(body["status"], "created");
        assert!(
            body["project"].as_str().is_some(),
            "should have project number"
        );
        assert!(body["path"].as_str().is_some(), "should have path");
    } else if status == 503 {
        // SSH not available in test env — acceptable
        assert_eq!(body["error"], "folder_creation_failed");
    } else {
        panic!("Unexpected status {}: {:?}", status, body);
    }
}

// ---------------------------------------------------------------------------
// Task 2: /api/health alias and /openapi.json endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_api_health_alias() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/api/health", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["status"].is_string(),
        "/api/health must return same schema as /health"
    );
    assert!(
        body["uptime"].is_number(),
        "/api/health must include uptime"
    );
}

#[tokio::test]
async fn test_openapi_json_endpoint() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/openapi.json", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["openapi"].is_string(), "must have 'openapi' field");
    assert!(body["paths"].is_object(), "must have 'paths' field");
}

// ---------------------------------------------------------------------------
// Task 3: /help endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_help_endpoint() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["service"].is_string(), "must have 'service'");
    assert!(body["version"].is_string(), "must have 'version'");
    assert!(body["description"].is_string(), "must have 'description'");
    assert!(body["endpoints"].is_array(), "must have 'endpoints' array");
    assert!(
        !body["endpoints"].as_array().unwrap().is_empty(),
        "endpoints must not be empty"
    );
}

#[tokio::test]
async fn test_help_no_auth_required() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 401, "/help should not require auth");
}

// ===========================================================================
// GET /projects/typeahead — lightweight type-ahead search for cad-export
// ===========================================================================

#[tokio::test]
async fn test_typeahead_requires_auth() {
    verify_not_production();
    let client = Client::new();
    let resp = client
        .get(format!("{}/projects/typeahead", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_typeahead_returns_array() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/typeahead", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(
        body.is_array(),
        "typeahead must return a bare JSON array (not a paginated wrapper), got: {}",
        body
    );
}

#[tokio::test]
async fn test_typeahead_item_shape() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/typeahead?limit=1", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    if let Some(first) = body.as_array().and_then(|a| a.first()) {
        assert!(first["number"].is_string(), "item must have 'number' string field");
        assert!(first["name"].is_string(), "item must have 'name' string field");
        // client field must be present (null or {id, name} object)
        assert!(
            first.get("client").is_some(),
            "item must have 'client' field (null or {{id, name}})"
        );
    }
}

#[tokio::test]
async fn test_typeahead_default_limit_20() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/typeahead", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    let count = body.as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        count <= 20,
        "default limit should be 20, got {} items",
        count
    );
}

#[tokio::test]
async fn test_typeahead_limit_clamped_to_100() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/typeahead?limit=9999", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    let count = body.as_array().map(|a| a.len()).unwrap_or(0);
    assert!(
        count <= 100,
        "limit must be clamped to 100, got {} items",
        count
    );
}

#[tokio::test]
async fn test_typeahead_no_match_returns_empty_array() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!(
            "{}/projects/typeahead?q=zzz_no_match_xyz_abc_99999",
            base_url()
        ))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert!(
        body.as_array().unwrap().is_empty(),
        "no-match query must return empty array, got: {}",
        body
    );
}

#[tokio::test]
async fn test_typeahead_q_filters_by_name() {
    // Seed a project with a distinctive name, search for it, verify it appears.
    verify_not_production();
    let client = authed_client();

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let seq = (ts % 800 + 101) as i64;
    let project_number_id = format!("26-971{}", seq);
    let project_key = project_number_id.replace('-', "_");
    let unique_name = format!("DELETE ME - Typeahead Filter {}", ts);

    let create_body = serde_json::json!({
        "name": unique_name,
        "name_short": "DELETE ME - TAF",
        "status": "Lead",
        "area": "0",
        "city": "Dubai",
        "country": "UAE",
        "folder": "",
        "number": { "year": 26, "country": 971, "seq": seq, "id": project_number_id }
    });

    let resp = client
        .post(format!("{}/projects", base_url()))
        .json(&create_body)
        .send()
        .await
        .expect("Failed to create project");
    assert_eq!(
        resp.status(),
        200,
        "seed project failed: {:?}",
        resp.text().await
    );

    // Search using a substring of the unique name
    let resp = client
        .get(format!("{}/projects/typeahead?q=typeahead+filter", base_url()))
        .send()
        .await
        .expect("Failed to send typeahead request");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let results = body.as_array().unwrap();
    let found = results
        .iter()
        .any(|r| r["name"].as_str().unwrap_or("").to_lowercase().contains("typeahead filter"));
    assert!(
        found,
        "typeahead should find project by name substring (case-insensitive), got: {}",
        body
    );

    // Cleanup
    client
        .delete(format!("{}/projects/{}", base_url(), project_key))
        .send()
        .await
        .ok();
}

#[tokio::test]
async fn test_typeahead_q_filters_by_number() {
    verify_not_production();
    let client = authed_client();
    // Search by project number fragment — "26-" should return projects starting with that year
    let resp = client
        .get(format!("{}/projects/typeahead?q=26-971&limit=5", base_url()))
        .send()
        .await
        .expect("Failed to send typeahead request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    // All returned items should have numbers containing "26-971"
    if let Some(arr) = body.as_array() {
        for item in arr {
            let num = item["number"].as_str().unwrap_or("");
            assert!(
                num.contains("26-971") || num.is_empty(),
                "number '{}' does not match q=26-971",
                num
            );
        }
    }
}

#[tokio::test]
async fn test_typeahead_client_field_is_null_or_object() {
    verify_not_production();
    let client = authed_client();
    let resp = client
        .get(format!("{}/projects/typeahead?limit=5", base_url()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    for item in body.as_array().unwrap_or(&vec![]) {
        let c = &item["client"];
        assert!(
            c.is_null() || c.is_object(),
            "client field must be null or {{id, name}} object, got: {}",
            c
        );
        if c.is_object() {
            assert!(c["id"].is_string(), "client.id must be a string");
            assert!(c["name"].is_string(), "client.name must be a string");
        }
    }
}

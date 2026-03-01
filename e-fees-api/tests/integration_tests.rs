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

    assert_eq!(resp.status(), 200, "create project failed: {:?}", resp.text().await);
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
    let company_key = company_full_id.strip_prefix("company:").unwrap().to_string();

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
    let contact_key = contact_full_id.strip_prefix("contacts:").unwrap().to_string();

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
        assert!(first["first_name"].is_string(), "contact missing 'first_name'");
        assert!(first["last_name"].is_string(), "contact missing 'last_name'");
    }
}

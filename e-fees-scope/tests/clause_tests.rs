//! Integration tests for the clause CRUD endpoints.
//!
//! Requires the e-fees-scope service to be running.
//! Set `API_BASE_URL` and `API_KEY` environment variables.
//!
//! Run with: cargo test -p e-fees-scope --test clause_tests -- --test-threads=1

use reqwest::Client;
use serde_json::{json, Value};

fn base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3300".to_string())
}

fn api_key() -> String {
    std::env::var("API_KEY").expect("API_KEY env var must be set")
}

fn client() -> Client {
    Client::new()
}

/// Helper: create a clause and return the full response body.
async fn create_test_clause(client: &Client, extra: Option<Value>) -> Value {
    let mut body = json!({
        "category": "DELETE ME - Test Category",
        "title": "DELETE ME - Test Clause Title",
        "body": "DELETE ME - This is a test clause body for integration testing.",
        "sort_order": 100,
        "is_default": false,
    });

    if let Some(extra) = extra {
        if let (Some(base_obj), Some(extra_obj)) = (body.as_object_mut(), extra.as_object()) {
            for (k, v) in extra_obj {
                base_obj.insert(k.clone(), v.clone());
            }
        }
    }

    let resp = client
        .post(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .json(&body)
        .send()
        .await
        .expect("Failed to send create request");

    assert!(
        resp.status().is_success(),
        "Create clause failed with status {}",
        resp.status()
    );

    resp.json::<Value>()
        .await
        .expect("Failed to parse create response")
}

/// Helper: hard-delete a test clause from SurrealDB (cleanup).
/// Uses the API's DELETE endpoint (soft-delete/archive), which is sufficient for test isolation.
async fn cleanup_clause(client: &Client, id: &str) {
    // First archive it via the API
    let _ = client
        .delete(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await;
}

// ============================================================================
// TESTS
// ============================================================================

#[tokio::test]
async fn test_health() {
    let resp = client()
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .expect("Health check failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.expect("Failed to parse health response");
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
async fn test_health_has_dependencies() {
    let body: Value = client()
        .get(format!("{}/health", base_url()))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        body["dependencies"]["surrealdb"].is_object(),
        "missing 'surrealdb' dependency"
    );
    assert!(
        body["dependencies"]["surrealdb"]["status"].is_string(),
        "missing surrealdb status"
    );
    assert!(
        body["dependencies"]["ollama"].is_object(),
        "missing 'ollama' dependency"
    );
    assert!(
        body["dependencies"]["ollama"]["status"].is_string(),
        "missing ollama status"
    );
}

#[tokio::test]
async fn test_api_health_alias() {
    let resp = client()
        .get(format!("{}/api/health", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["status"].is_string());
    assert!(body["uptime"].is_number());
}

#[tokio::test]
async fn test_openapi_json_endpoint() {
    let resp = client()
        .get(format!("{}/openapi.json", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["openapi"].is_string(), "must have 'openapi' field");
    assert!(body["paths"].is_object(), "must have 'paths' field");
}

#[tokio::test]
async fn test_help_endpoint() {
    let resp = client()
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
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
    let resp = client()
        .get(format!("{}/help", base_url()))
        .send()
        .await
        .unwrap();
    assert_ne!(resp.status(), 401, "/help should not require auth");
}

#[tokio::test]
async fn test_clauses_require_auth() {
    let resp = client()
        .get(format!("{}/clauses", base_url()))
        // No API key header
        .send()
        .await
        .expect("Request failed");

    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_clause() {
    let c = client();
    let result = create_test_clause(&c, None).await;

    let data = &result["data"];
    assert!(data["id"].is_string(), "id should be a string");
    assert_eq!(data["category"], "DELETE ME - Test Category");
    assert_eq!(data["title"], "DELETE ME - Test Clause Title");
    assert_eq!(data["status"], "active");
    assert_eq!(data["version"], 1);
    assert_eq!(data["is_default"], false);
    assert_eq!(data["sort_order"], 100);

    // Cleanup
    let id = data["id"].as_str().unwrap();
    cleanup_clause(&c, id).await;
}

#[tokio::test]
async fn test_create_and_get_clause() {
    let c = client();

    let extra = json!({
        "subcategory": "DELETE ME - Subcategory",
        "tags": ["test", "integration", "delete-me"],
        "conditions": { "min_area": 500, "project_type": "hospitality" },
    });
    let result = create_test_clause(&c, Some(extra)).await;

    let id = result["data"]["id"].as_str().unwrap().to_string();

    // GET the clause
    let resp = c
        .get(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET clause failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["id"], id);
    assert_eq!(data["subcategory"], "DELETE ME - Subcategory");
    assert_eq!(data["tags"], json!(["test", "integration", "delete-me"]));
    assert_eq!(data["conditions"]["min_area"], 500);
    assert_eq!(data["conditions"]["project_type"], "hospitality");
    assert_eq!(data["version"], 1);

    // Cleanup
    cleanup_clause(&c, &id).await;
}

#[tokio::test]
async fn test_update_clause_increments_version() {
    let c = client();

    let result = create_test_clause(&c, None).await;
    let id = result["data"]["id"].as_str().unwrap().to_string();

    // Update body and title
    let update_body = json!({
        "title": "DELETE ME - Updated Title",
        "body": "DELETE ME - Updated clause body text.",
    });

    let resp = c
        .put(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .json(&update_body)
        .send()
        .await
        .expect("PUT clause failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["title"], "DELETE ME - Updated Title");
    assert_eq!(data["body"], "DELETE ME - Updated clause body text.");
    assert_eq!(data["version"], 2, "Version should be incremented to 2");

    // Cleanup
    cleanup_clause(&c, &id).await;
}

#[tokio::test]
async fn test_delete_clause_archives() {
    let c = client();

    let result = create_test_clause(&c, None).await;
    let id = result["data"]["id"].as_str().unwrap().to_string();

    // DELETE (soft-delete)
    let resp = c
        .delete(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("DELETE clause failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["data"]["status"], "archived",
        "Deleted clause should have status 'archived'"
    );

    // Verify via GET that it's archived
    let resp2 = c
        .get(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET archived clause failed");

    assert_eq!(resp2.status(), 200);
    let body2: Value = resp2.json().await.unwrap();
    assert_eq!(body2["data"]["status"], "archived");
}

#[tokio::test]
async fn test_list_clauses_with_category_filter() {
    let c = client();

    // Create two clauses in different categories
    let c1 = create_test_clause(&c, Some(json!({ "category": "DELETE ME - Cat Alpha" }))).await;
    let c2 = create_test_clause(&c, Some(json!({ "category": "DELETE ME - Cat Beta" }))).await;

    let id1 = c1["data"]["id"].as_str().unwrap().to_string();
    let id2 = c2["data"]["id"].as_str().unwrap().to_string();

    // List with category filter
    let encoded_cat = urlencoding::encode("DELETE ME - Cat Alpha");
    let resp = c
        .get(format!("{}/clauses?category={}", base_url(), encoded_cat))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("List clauses with filter failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().expect("data should be an array");

    // Should contain at least the Alpha clause, and NOT contain the Beta clause
    let has_alpha = data.iter().any(|c| c["id"] == id1);
    let has_beta = data.iter().any(|c| c["id"] == id2);
    assert!(has_alpha, "Should contain the Alpha category clause");
    assert!(!has_beta, "Should NOT contain the Beta category clause");

    // Cleanup
    cleanup_clause(&c, &id1).await;
    cleanup_clause(&c, &id2).await;
}

/// Regression: the unfiltered list must deserialize EVERY active clause, including
/// legacy/seeded rows. A row whose `created_at`/`updated_at` was stored as an ISO
/// string instead of a SurrealDB datetime made `Vec<Clause>` deserialization fail and
/// the endpoint return 500. The category-filtered test missed it because it only lists
/// its own freshly-created (datetime) clause. Fixed 2026-06-14 by migrating the clause
/// timestamps to datetime; this test is read-only and creates no test data.
#[tokio::test]
async fn test_list_clauses_unfiltered_deserializes_all_active() {
    let c = client();

    let resp = c
        .get(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("Unfiltered list clauses request failed");

    assert_eq!(
        resp.status(),
        200,
        "GET /clauses must return 200; a 500 means an active clause failed to \
         deserialize (e.g. a string timestamp where a datetime is expected)"
    );

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().expect("data should be an array");
    assert!(
        !data.is_empty(),
        "the active clause library should not be empty"
    );
    for clause in data {
        assert!(
            clause.get("created_at").is_some(),
            "each listed clause must expose created_at: {clause:?}"
        );
    }
}

#[tokio::test]
async fn test_list_categories() {
    let c = client();

    let resp = c
        .get(format!("{}/clauses/categories", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("List categories failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert!(
        body["data"].is_array(),
        "categories response should have a data array"
    );
}

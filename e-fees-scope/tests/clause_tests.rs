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

// ============================================================================
// CLAUSE LIBRARY BACKLOG TESTS (2026-06-19)
// Validates 4 divergent fixes + 7 gap clauses from the 2026-06-14 library audit.
// All tests are read-only against the live clause library (no test-data cleanup needed).
// ============================================================================

/// Helper: fetch all active clauses and return the one matching the given title, if any.
async fn find_clause_by_title(client: &Client, title: &str) -> Option<Value> {
    let resp = client
        .get(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("List clauses request failed");

    assert_eq!(resp.status(), 200, "GET /clauses returned non-200");

    let body: Value = resp.json().await.expect("Failed to parse clause list");
    let clauses = body["data"].as_array().expect("data must be array");

    clauses
        .iter()
        .find(|c| c["title"].as_str() == Some(title))
        .cloned()
}

// ── FIX 1: Defined Role ──────────────────────────────────────────────────────

/// Defined Role must contain the Regulations / standards paragraph (appears in 61/69 historical FPs).
/// Audit finding: the library clause omits this material client obligation entirely.
#[tokio::test]
async fn test_defined_role_has_regulations_paragraph() {
    let c = client();
    let clause = find_clause_by_title(&c, "Defined Role")
        .await
        .expect("Defined Role clause must exist in the active library");

    let body = clause["body"].as_str().expect("body must be a string");

    assert!(
        body.contains("Regulations / standards / landlord guidelines"),
        "Defined Role body must contain the regulations/standards paragraph. \
         This appears in 61/69 historical FPs and establishes the client's obligation \
         to disclose applicable standards. Current body: {body:.200}"
    );

    assert!(
        body.contains("inform emittiv at time of appointment"),
        "Defined Role regulations paragraph must include the canonical 'inform emittiv at \
         time of appointment' sentence. Current body: {body:.200}"
    );

    assert!(
        body.contains("Part L or CIBSE"),
        "Defined Role regulations paragraph must reference 'Part L or CIBSE' as the baseline \
         standard. Current body: {body:.200}"
    );
}

// ── FIX 2: Fees / Payment Terms ──────────────────────────────────────────────

/// Fees / Payment Terms must contain the canonical variation notice (absent from library, present in catalog).
#[tokio::test]
async fn test_fees_payment_terms_has_variation_notice() {
    let c = client();
    let clause = find_clause_by_title(&c, "Fees / Payment Terms")
        .await
        .expect("Fees / Payment Terms clause must exist in the active library");

    let body = clause["body"].as_str().expect("body must be a string");

    assert!(
        body.contains("Before exceeding the above fee"),
        "Fees / Payment Terms must contain the canonical fee variation notice starting with \
         'Before exceeding the above fee...'. This appears in the dominant historical canonical \
         and is absent from the current library clause. Current body: {body:.200}"
    );

    assert!(
        body.contains("notify the Client in writing"),
        "Fee variation notice must include 'notify the Client in writing'. \
         Current body: {body:.200}"
    );
}

/// Fees / Payment Terms must use 14 calendar days (not 30) as the dominant historical default.
#[tokio::test]
async fn test_fees_payment_terms_uses_14_day_window() {
    let c = client();
    let clause = find_clause_by_title(&c, "Fees / Payment Terms")
        .await
        .expect("Fees / Payment Terms clause must exist in the active library");

    let body = clause["body"].as_str().expect("body must be a string");

    assert!(
        body.contains("14 calendar days") || body.contains("fourteen (14)") || body.contains("14-day"),
        "Fees / Payment Terms must use 14 calendar days (dominant historical default, 25+ proposals). \
         The library incorrectly uses 30 days. Current body: {body:.200}"
    );
}

// ── FIX 3: Proposal Validity ──────────────────────────────────────────────────

/// Proposal Validity must use the historical canonical wording with "sixty [60] days".
#[tokio::test]
async fn test_proposal_validity_uses_canonical_sixty_60_wording() {
    let c = client();
    let clause = find_clause_by_title(&c, "Proposal Validity")
        .await
        .expect("Proposal Validity clause must exist in the active library");

    let body = clause["body"].as_str().expect("body must be a string");

    assert!(
        body.contains("sixty [60] days"),
        "Proposal Validity must use the canonical 'sixty [60] days' wording (62/69 historical proposals). \
         The library currently uses '[XX] days' placeholder. Current body: {body:.300}"
    );

    assert!(
        body.contains("shall remain valid"),
        "Proposal Validity must use 'shall remain valid' (canonical wording). \
         Current body: {body:.300}"
    );
}

// ── FIX 4: Design Phase Notes ────────────────────────────────────────────────

/// Design Phase Notes must contain the illustrative renders clause (Variant A2, recommended as new default).
#[tokio::test]
async fn test_design_phase_notes_has_illustrative_renders_clause() {
    let c = client();
    let clause = find_clause_by_title(&c, "Design Phase Notes")
        .await
        .expect("Design Phase Notes clause must exist in the active library");

    let body = clause["body"].as_str().expect("body must be a string");

    assert!(
        body.contains("illustrative in nature"),
        "Design Phase Notes must contain the 'illustrative renders' bullet (Variant A2, \
         12 recent 2024-2026 proposals, recommended as new canonical default). \
         The current library has only the older 8-point Variant A1. Current body: {body:.300}"
    );

    assert!(
        body.contains("realistic renders"),
        "Design Phase Notes renders clause must mention 'realistic renders' prepared by the Architect. \
         Current body: {body:.300}"
    );
}

// ── GAP 1: Scope & Services - Appointment Confirmation ───────────────────────

/// The Scope & Services appointment confirmation clause must exist.
/// This is the most-used clause family in the corpus with no prior library equivalent.
#[tokio::test]
async fn test_scope_services_appointment_confirmation_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Scope & Services - Appointment Confirmation")
        .await;

    assert!(
        clause.is_some(),
        "Scope & Services - Appointment Confirmation clause must exist. \
         This appears in nearly every proposal and is the biggest catalog gap (GAP 1)."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("emittiv confirms"),
        "Appointment confirmation body must contain 'emittiv confirms'. Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Services"),
        "Appointment confirmation must be in the Services category"
    );
}

// ── GAP 2: Scope & Services - Areas Included/Excluded ────────────────────────

/// The Scope & Services areas block must exist.
#[tokio::test]
async fn test_scope_services_areas_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Scope & Services - Areas")
        .await;

    assert!(
        clause.is_some(),
        "Scope & Services - Areas clause must exist. \
         The areas included/excluded block appears in nearly every proposal."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("Lighting") || body.contains("lighting"),
        "Areas clause body must list Lighting as a service area. Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Services"),
        "Areas clause must be in the Services category"
    );
}

// ── GAP 3: Concept Design stage clause ───────────────────────────────────────

/// Concept Design stage clause must exist (most-used individual stage deliverable, 25+ proposals).
#[tokio::test]
async fn test_concept_design_stage_clause_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Stage X - Concept Design")
        .await;

    assert!(
        clause.is_some(),
        "Stage X - Concept Design clause must exist. \
         This is the most-used individual stage deliverable (25+ proposals) with no library equivalent."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("Concept Design Report"),
        "Concept Design clause must mention 'Concept Design Report'. Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Services"),
        "Concept Design must be in the Services category"
    );
}

// ── GAP 4: Schematic Design stage clause ─────────────────────────────────────

/// Schematic Design stage clause must exist (second most-used stage, 20+ proposals).
#[tokio::test]
async fn test_schematic_design_stage_clause_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Stage X - Schematic Design")
        .await;

    assert!(
        clause.is_some(),
        "Stage X - Schematic Design clause must exist. \
         This is the second most-used stage deliverable (20+ proposals) with no library equivalent."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("LOD 200") || body.contains("Schematic Design"),
        "Schematic Design clause must reference LOD 200 or Schematic Design phase. Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Services"),
        "Schematic Design must be in the Services category"
    );
}

// ── GAP 5: Exclusions clause ──────────────────────────────────────────────────

/// Exclusions clause must exist (heavily reused in proposals, absent from library).
#[tokio::test]
async fn test_exclusions_clause_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Exclusions")
        .await;

    assert!(
        clause.is_some(),
        "Exclusions clause must exist. \
         A discipline-organised exclusions clause is heavily reused across proposals."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("Emergency Lighting") || body.contains("PAVA"),
        "Exclusions clause must list key exclusions (Emergency Lighting or PAVA). Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Legal"),
        "Exclusions must be in the Legal category"
    );
}

// ── GAP 6: Company Profile ────────────────────────────────────────────────────

/// Company Profile clause must exist (highest-reuse boilerplate, 7+ proposals verbatim).
#[tokio::test]
async fn test_company_profile_clause_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "About emittiv")
        .await;

    assert!(
        clause.is_some(),
        "About emittiv clause must exist. \
         This is the highest-reuse non-templatable boilerplate in the corpus (7+ proposals verbatim)."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("emittiv"),
        "Company profile body must contain 'emittiv'. Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Administrative"),
        "Company profile must be in the Administrative category"
    );
}

// ── GAP 7: Post Contract Fees ─────────────────────────────────────────────────

/// Post Contract Fees clause must exist (no standalone clause for post-contract fee scheduling).
#[tokio::test]
async fn test_post_contract_fees_clause_exists() {
    let c = client();
    let clause = find_clause_by_title(&c, "Post Contract Fees")
        .await;

    assert!(
        clause.is_some(),
        "Post Contract Fees clause must exist. \
         No standalone post-contract fee schedule clause exists in the library."
    );

    let clause = clause.unwrap();
    let body = clause["body"].as_str().expect("body must be a string");
    assert!(
        body.contains("estimate") || body.contains("post-contract") || body.contains("Post Contract"),
        "Post Contract Fees body must mention the estimate disclaimer or post-contract context. \
         Body: {body:.300}"
    );
    assert_eq!(
        clause["category"].as_str(),
        Some("Commercial"),
        "Post Contract Fees must be in the Commercial category"
    );
}

// ── Regression: all active clauses still deserialize after new additions ───────

/// After adding new clauses, the unfiltered list must still deserialize all active clauses without 500.
/// This is a regression test to catch timestamp/schema issues in new records.
#[tokio::test]
async fn test_library_complete_after_backlog_all_deserialize() {
    let c = client();
    let resp = c
        .get(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /clauses failed");

    assert_eq!(
        resp.status(),
        200,
        "GET /clauses must return 200 after backlog additions"
    );

    let body: Value = resp.json().await.expect("Failed to parse clause list");
    let clauses = body["data"].as_array().expect("data must be array");

    // After backlog: 4 fixes (existing) + 7 new = still 20 + 7 = 27 active clauses minimum
    // (Assumptions clause paw4ejkcmhtwpm0l2miq may or may not be present in dev)
    assert!(
        clauses.len() >= 27,
        "Active clause library must have at least 27 clauses after backlog additions. \
         Found: {}",
        clauses.len()
    );

    for clause in clauses {
        assert!(
            clause.get("created_at").is_some(),
            "Each clause must expose created_at after backlog additions: {clause:?}"
        );
        assert!(
            clause.get("id").is_some() && !clause["id"].as_str().unwrap_or("").is_empty(),
            "Each clause must have a non-empty id: {clause:?}"
        );
    }
}

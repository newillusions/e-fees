//! Integration tests for deliverable CRUD and assembly endpoints.
//!
//! Requires the e-fees-scope service to be running.
//! Set `API_BASE_URL` and `API_KEY` environment variables.
//!
//! Run with: cargo test -p e-fees-scope --test deliverable_tests -- --test-threads=1

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

// ============================================================================
// Helpers
// ============================================================================

/// Create a generic deliverable and return the response body.
async fn create_generic_deliverable(c: &Client) -> Value {
    let body = json!({
        "title": "DELETE ME - Test Generic Deliverable",
        "short_name": "Test Gen",
        "body": "Test body",
        "stage": "concept",
        "layer": "generic",
        "sort_order": 900,
    });

    let resp = c
        .post(format!("{}/deliverables", base_url()))
        .header("X-API-Key", api_key())
        .json(&body)
        .send()
        .await
        .expect("Failed to create generic deliverable");

    assert!(
        resp.status().is_success(),
        "Create generic deliverable failed: {}",
        resp.status()
    );

    resp.json::<Value>()
        .await
        .expect("Failed to parse response")
}

/// Create a discipline deliverable that replaces a generic one.
async fn create_discipline_deliverable(c: &Client, replaces_id: &str) -> Value {
    let body = json!({
        "title": "DELETE ME - Test Lighting Deliverable",
        "short_name": "Test Lgt",
        "body": "Lighting-specific body",
        "stage": "concept",
        "layer": "discipline",
        "discipline": "lighting",
        "sort_order": 900,
        "replaces": replaces_id,
    });

    let resp = c
        .post(format!("{}/deliverables", base_url()))
        .header("X-API-Key", api_key())
        .json(&body)
        .send()
        .await
        .expect("Failed to create discipline deliverable");

    assert!(
        resp.status().is_success(),
        "Create discipline deliverable failed: {}",
        resp.status()
    );

    resp.json::<Value>()
        .await
        .expect("Failed to parse response")
}

/// Soft-delete (archive) a deliverable by ID.
async fn cleanup_deliverable(c: &Client, id: &str) {
    let _ = c
        .delete(format!("{}/deliverables/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await;
}

/// Search for any lingering "DELETE ME" deliverables and archive them.
async fn cleanup_all_test_deliverables(c: &Client) {
    let resp = c
        .get(format!(
            "{}/deliverables?search={}",
            base_url(),
            urlencoding::encode("DELETE ME")
        ))
        .header("X-API-Key", api_key())
        .send()
        .await;

    if let Ok(resp) = resp {
        if let Ok(body) = resp.json::<Value>().await {
            if let Some(arr) = body["data"].as_array() {
                for item in arr {
                    if let Some(id) = item["id"].as_str() {
                        cleanup_deliverable(c, id).await;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Stage config tests
// ============================================================================

#[tokio::test]
async fn test_stages_list_returns_ordered_stages() {
    let resp = client()
        .get(format!("{}/stages", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /stages failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().expect("data should be an array");

    assert_eq!(data.len(), 8, "Should have 8 stages");

    // Each stage must have the required fields
    for stage in data {
        assert!(
            stage["canonical_name"].is_string(),
            "canonical_name missing"
        );
        assert!(stage["default_label"].is_string(), "default_label missing");
        assert!(stage["sort_order"].is_number(), "sort_order missing");
        // aliases is optional so we just check it doesn't panic
    }

    // First should be preliminaries, last should be handover
    assert_eq!(
        data[0]["canonical_name"], "preliminaries",
        "First stage should be preliminaries"
    );
    assert_eq!(
        data[7]["canonical_name"], "handover",
        "Last stage should be handover"
    );

    // Verify sort_order is ascending
    for i in 1..data.len() {
        let prev = data[i - 1]["sort_order"].as_i64().unwrap();
        let curr = data[i]["sort_order"].as_i64().unwrap();
        assert!(curr >= prev, "Stages should be ordered by sort_order");
    }
}

// ============================================================================
// Deliverable CRUD tests
// ============================================================================

#[tokio::test]
async fn test_deliverable_create() {
    let c = client();

    // Cleanup any leftovers first
    cleanup_all_test_deliverables(&c).await;

    let result = create_generic_deliverable(&c).await;
    let data = &result["data"];

    assert!(data["id"].is_string(), "id should be a string");
    assert_eq!(data["title"], "DELETE ME - Test Generic Deliverable");
    assert_eq!(data["short_name"], "Test Gen");
    assert_eq!(data["body"], "Test body");
    assert_eq!(data["stage"], "concept");
    assert_eq!(data["layer"], "generic");
    assert_eq!(data["status"], "active");
    assert_eq!(data["version"], 1);
    assert_eq!(data["sort_order"], 900);

    // Cleanup
    cleanup_deliverable(&c, data["id"].as_str().unwrap()).await;
}

#[tokio::test]
async fn test_deliverable_list_with_stage_filter() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    let created = create_generic_deliverable(&c).await;
    let id = created["data"]["id"].as_str().unwrap().to_string();

    // List with stage=concept filter
    let resp = c
        .get(format!("{}/deliverables?stage=concept", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /deliverables?stage=concept failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().expect("data should be an array");

    let found = data.iter().any(|d| d["id"].as_str() == Some(&id));
    assert!(
        found,
        "Test deliverable should appear in stage=concept list"
    );

    // Cleanup
    cleanup_deliverable(&c, &id).await;
}

#[tokio::test]
async fn test_deliverable_get_by_id() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    let created = create_generic_deliverable(&c).await;
    let id = created["data"]["id"].as_str().unwrap().to_string();

    // GET by ID
    let resp = c
        .get(format!("{}/deliverables/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /deliverables/{id} failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["id"], id);
    assert_eq!(data["title"], "DELETE ME - Test Generic Deliverable");
    assert_eq!(data["body"], "Test body");
    assert_eq!(data["version"], 1);

    // Cleanup
    cleanup_deliverable(&c, &id).await;
}

#[tokio::test]
async fn test_deliverable_update_bumps_version() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    let created = create_generic_deliverable(&c).await;
    let id = created["data"]["id"].as_str().unwrap().to_string();

    // Update body
    let update_body = json!({
        "body": "Updated test body",
    });

    let resp = c
        .put(format!("{}/deliverables/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .json(&update_body)
        .send()
        .await
        .expect("PUT /deliverables/{id} failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = &body["data"];

    assert_eq!(data["body"], "Updated test body");
    assert_eq!(data["version"], 2, "Version should be bumped to 2");

    // Cleanup
    cleanup_deliverable(&c, &id).await;
}

#[tokio::test]
async fn test_deliverable_search() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    let created = create_generic_deliverable(&c).await;
    let id = created["data"]["id"].as_str().unwrap().to_string();

    // Fuzzy search
    let resp = c
        .get(format!(
            "{}/deliverables?search={}",
            base_url(),
            urlencoding::encode("DELETE ME")
        ))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /deliverables?search=DELETE+ME failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let data = body["data"].as_array().expect("data should be an array");

    let found = data.iter().any(|d| d["id"].as_str() == Some(&id));
    assert!(found, "Test deliverable should appear in search results");

    // Cleanup
    cleanup_deliverable(&c, &id).await;
}

#[tokio::test]
async fn test_deliverable_discipline_with_replaces() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // Create generic first
    let generic = create_generic_deliverable(&c).await;
    let generic_id = generic["data"]["id"].as_str().unwrap().to_string();

    // Create discipline deliverable that replaces the generic one
    let discipline = create_discipline_deliverable(&c, &generic_id).await;
    let discipline_id = discipline["data"]["id"].as_str().unwrap().to_string();

    let data = &discipline["data"];
    assert_eq!(data["layer"], "discipline");
    assert_eq!(data["discipline"], "lighting");
    assert_eq!(data["replaces"], generic_id);

    // Cleanup
    cleanup_deliverable(&c, &discipline_id).await;
    cleanup_deliverable(&c, &generic_id).await;
}

#[tokio::test]
async fn test_deliverable_delete_archives() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    let created = create_generic_deliverable(&c).await;
    let id = created["data"]["id"].as_str().unwrap().to_string();

    // DELETE (soft-delete)
    let resp = c
        .delete(format!("{}/deliverables/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("DELETE /deliverables/{id} failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["data"]["status"], "archived",
        "Deleted deliverable should have status 'archived'"
    );

    // Verify via GET
    let resp2 = c
        .get(format!("{}/deliverables/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET archived deliverable failed");

    assert_eq!(resp2.status(), 200);
    let body2: Value = resp2.json().await.unwrap();
    assert_eq!(body2["data"]["status"], "archived");
}

// ============================================================================
// Assembly engine tests
// ============================================================================

#[tokio::test]
async fn test_assemble_with_discipline_dedup() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // Create a generic deliverable
    let generic = create_generic_deliverable(&c).await;
    let generic_id = generic["data"]["id"].as_str().unwrap().to_string();

    // Create a lighting discipline deliverable that replaces the generic
    let discipline = create_discipline_deliverable(&c, &generic_id).await;
    let discipline_id = discipline["data"]["id"].as_str().unwrap().to_string();

    // Assemble with lighting discipline — generic should be replaced
    let assemble_body = json!({
        "fee_id": "test",
        "disciplines": ["lighting"],
        "stages": ["concept"],
    });

    let resp = c
        .post(format!("{}/scope/assemble", base_url()))
        .header("X-API-Key", api_key())
        .json(&assemble_body)
        .send()
        .await
        .expect("POST /scope/assemble failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let stages = body["stages"]
        .as_array()
        .expect("stages should be an array");

    // Find the concept stage
    let concept = stages
        .iter()
        .find(|s| s["canonical_name"] == "concept")
        .expect("concept stage should exist in response");

    let deliverables = concept["deliverables"]
        .as_array()
        .expect("deliverables should be an array");

    // The generic should NOT appear (it's been replaced by the lighting one)
    let has_generic = deliverables
        .iter()
        .any(|d| d["id"].as_str() == Some(&generic_id));
    assert!(
        !has_generic,
        "Generic deliverable should be replaced (deduped) by discipline override"
    );

    // The discipline one SHOULD appear
    let has_discipline = deliverables
        .iter()
        .any(|d| d["id"].as_str() == Some(&discipline_id));
    assert!(
        has_discipline,
        "Lighting discipline deliverable should appear in assembly"
    );

    // Cleanup
    cleanup_deliverable(&c, &discipline_id).await;
    cleanup_deliverable(&c, &generic_id).await;
}

#[tokio::test]
async fn test_assemble_without_matching_discipline_keeps_generic() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // Create a generic deliverable
    let generic = create_generic_deliverable(&c).await;
    let generic_id = generic["data"]["id"].as_str().unwrap().to_string();

    // Create a lighting discipline deliverable that replaces the generic
    let discipline = create_discipline_deliverable(&c, &generic_id).await;
    let discipline_id = discipline["data"]["id"].as_str().unwrap().to_string();

    // Assemble with AV discipline — lighting replacement shouldn't apply
    let assemble_body = json!({
        "fee_id": "test",
        "disciplines": ["av"],
        "stages": ["concept"],
    });

    let resp = c
        .post(format!("{}/scope/assemble", base_url()))
        .header("X-API-Key", api_key())
        .json(&assemble_body)
        .send()
        .await
        .expect("POST /scope/assemble failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let stages = body["stages"]
        .as_array()
        .expect("stages should be an array");

    let concept = stages
        .iter()
        .find(|s| s["canonical_name"] == "concept")
        .expect("concept stage should exist in response");

    let deliverables = concept["deliverables"]
        .as_array()
        .expect("deliverables should be an array");

    // The generic SHOULD appear (lighting replacement not relevant for AV)
    let has_generic = deliverables
        .iter()
        .any(|d| d["id"].as_str() == Some(&generic_id));
    assert!(
        has_generic,
        "Generic deliverable should appear when discipline doesn't match"
    );

    // The lighting discipline should NOT appear (not in requested disciplines)
    let has_discipline = deliverables
        .iter()
        .any(|d| d["id"].as_str() == Some(&discipline_id));
    assert!(
        !has_discipline,
        "Lighting deliverable should NOT appear when only AV is requested"
    );

    // Cleanup
    cleanup_deliverable(&c, &discipline_id).await;
    cleanup_deliverable(&c, &generic_id).await;
}

// ============================================================================
// Scope assembly save & load tests
// ============================================================================

/// Clean up any scope_assembly records for a given test fee key.
/// Uses the save endpoint's DELETE-before-CREATE pattern with an empty
/// deliverables list (which will error), so instead we do a raw approach:
/// save with valid data then accept the record stays — OR — we rely on
/// test isolation via unique fee keys.
///
/// Since there is no DELETE /scope/{id} endpoint, we clean up by saving
/// an assembly with a known deliverable then ignoring the leftover.
/// The test fee keys are unique enough to avoid collision.
async fn cleanup_scope_assembly(c: &Client, fee_key: &str) {
    // Best-effort: calling save with a known-good deliverable just to trigger
    // DELETE of the old record. If this fails, the leftover is harmless.
    // Actually, we just accept that test scope_assembly records with
    // "DELETE_ME" fee keys won't interfere with production data.
    let _ = c
        .post(format!("{}/scope/save", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "fee_id": fee_key,
            "deliverables": [],
        }))
        .send()
        .await;
}

#[tokio::test]
async fn test_assemble_returns_stages() {
    let c = client();

    // Use a known fee_id — doesn't need to exist for assembly
    let assemble_body = json!({
        "fee_id": "DELETE_ME_assemble_test",
        "disciplines": ["lighting"],
    });

    let resp = c
        .post(format!("{}/scope/assemble", base_url()))
        .header("X-API-Key", api_key())
        .json(&assemble_body)
        .send()
        .await
        .expect("POST /scope/assemble failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    let stages = body["stages"]
        .as_array()
        .expect("stages should be an array");

    // Should have at least one stage
    assert!(
        !stages.is_empty(),
        "Assembly should return at least one stage"
    );

    // Each stage must have canonical_name, label, and deliverables
    for stage in stages {
        assert!(
            stage["canonical_name"].is_string(),
            "Each stage must have canonical_name"
        );
        assert!(stage["label"].is_string(), "Each stage must have label");
        assert!(
            stage["deliverables"].is_array(),
            "Each stage must have deliverables array"
        );
    }
}

#[tokio::test]
async fn test_assemble_requires_disciplines() {
    let c = client();

    // Missing disciplines field entirely
    let body_no_disciplines = json!({
        "fee_id": "DELETE_ME_assemble_nodisciplines",
    });

    let resp = c
        .post(format!("{}/scope/assemble", base_url()))
        .header("X-API-Key", api_key())
        .json(&body_no_disciplines)
        .send()
        .await
        .expect("POST /scope/assemble failed");

    // Should fail with 400 or 422 (deserialization error for missing required field)
    assert!(
        resp.status().is_client_error(),
        "Missing disciplines should return 4xx, got {}",
        resp.status()
    );

    // Empty disciplines array
    let body_empty = json!({
        "fee_id": "DELETE_ME_assemble_emptydisciplines",
        "disciplines": [],
    });

    let resp2 = c
        .post(format!("{}/scope/assemble", base_url()))
        .header("X-API-Key", api_key())
        .json(&body_empty)
        .send()
        .await
        .expect("POST /scope/assemble with empty disciplines failed");

    assert_eq!(
        resp2.status(),
        400,
        "Empty disciplines should return 400, got {}",
        resp2.status()
    );
}

#[tokio::test]
async fn test_save_and_load_scope() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // 1. Create a test deliverable to reference
    let created = create_generic_deliverable(&c).await;
    let del_id = created["data"]["id"].as_str().unwrap().to_string();

    let test_fee_key = "DELETE_ME_save_load_test";

    // 2. Save a scope with that deliverable
    let save_body = json!({
        "fee_id": test_fee_key,
        "deliverables": [{
            "deliverable_id": &del_id,
            "stage": "concept",
            "sort_order": 1,
            "wording_override": null,
        }],
        "stage_labels": {
            "concept": "Concept Design Phase"
        },
    });

    let save_resp = c
        .post(format!("{}/scope/save", base_url()))
        .header("X-API-Key", api_key())
        .json(&save_body)
        .send()
        .await
        .expect("POST /scope/save failed");

    assert_eq!(
        save_resp.status(),
        200,
        "Save should succeed, got {}",
        save_resp.status()
    );

    let save_result: Value = save_resp.json().await.unwrap();
    assert_eq!(save_result["status"], "saved");
    assert_eq!(save_result["fee_id"], test_fee_key);
    assert_eq!(save_result["deliverables_count"], 1);

    // 3. Load it back
    let load_resp = c
        .get(format!(
            "{}/scope/{}/deliverables",
            base_url(),
            test_fee_key
        ))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /scope/{fee_id}/deliverables failed");

    assert_eq!(load_resp.status(), 200);

    let load_body: Value = load_resp.json().await.unwrap();
    assert_eq!(load_body["fee_id"], test_fee_key);

    let deliverables_used = load_body["deliverables_used"]
        .as_array()
        .expect("deliverables_used should be an array");

    assert_eq!(
        deliverables_used.len(),
        1,
        "Should have exactly 1 deliverable"
    );

    let saved_del = &deliverables_used[0];
    assert_eq!(saved_del["deliverable_id"], del_id);
    assert_eq!(saved_del["stage"], "concept");
    assert_eq!(saved_del["sort_order"], 1);
    // wording_snapshot should be the master body (no override)
    assert_eq!(saved_del["wording_snapshot"], "Test body");
    assert_eq!(saved_del["short_name"], "Test Gen");

    // Verify stage_labels
    assert_eq!(
        load_body["stage_labels"]["concept"], "Concept Design Phase",
        "Stage labels should be saved and returned"
    );

    // 4. Cleanup
    cleanup_scope_assembly(&c, test_fee_key).await;
    cleanup_deliverable(&c, &del_id).await;
}

#[tokio::test]
async fn test_save_scope_overwrites() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // Create two test deliverables
    let created1 = create_generic_deliverable(&c).await;
    let del_id_1 = created1["data"]["id"].as_str().unwrap().to_string();

    // Create a second deliverable with a distinct title
    let body2 = json!({
        "title": "DELETE ME - Test Second Deliverable",
        "short_name": "Test 2nd",
        "body": "Second body",
        "stage": "concept",
        "layer": "generic",
        "sort_order": 901,
    });
    let resp2 = c
        .post(format!("{}/deliverables", base_url()))
        .header("X-API-Key", api_key())
        .json(&body2)
        .send()
        .await
        .expect("Failed to create second deliverable");
    assert!(resp2.status().is_success());
    let created2: Value = resp2.json().await.unwrap();
    let del_id_2 = created2["data"]["id"].as_str().unwrap().to_string();

    let test_fee_key = "DELETE_ME_overwrite_test";

    // First save: deliverable 1
    let save1 = json!({
        "fee_id": test_fee_key,
        "deliverables": [{
            "deliverable_id": &del_id_1,
            "stage": "concept",
            "sort_order": 1,
            "wording_override": null,
        }],
    });

    let resp = c
        .post(format!("{}/scope/save", base_url()))
        .header("X-API-Key", api_key())
        .json(&save1)
        .send()
        .await
        .expect("First save failed");
    assert_eq!(resp.status(), 200);

    // Second save: deliverable 2 (should replace deliverable 1)
    let save2 = json!({
        "fee_id": test_fee_key,
        "deliverables": [{
            "deliverable_id": &del_id_2,
            "stage": "concept",
            "sort_order": 1,
            "wording_override": "DELETE ME - Custom wording override",
        }],
    });

    let resp = c
        .post(format!("{}/scope/save", base_url()))
        .header("X-API-Key", api_key())
        .json(&save2)
        .send()
        .await
        .expect("Second save failed");
    assert_eq!(resp.status(), 200);

    // Load and verify only the second deliverable is present
    let load_resp = c
        .get(format!(
            "{}/scope/{}/deliverables",
            base_url(),
            test_fee_key
        ))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /scope/{fee_id}/deliverables failed");

    assert_eq!(load_resp.status(), 200);

    let load_body: Value = load_resp.json().await.unwrap();
    let deliverables_used = load_body["deliverables_used"]
        .as_array()
        .expect("deliverables_used should be an array");

    assert_eq!(
        deliverables_used.len(),
        1,
        "Should have exactly 1 deliverable after overwrite"
    );

    assert_eq!(
        deliverables_used[0]["deliverable_id"], del_id_2,
        "Should have the second deliverable, not the first"
    );
    assert_eq!(
        deliverables_used[0]["wording_snapshot"], "DELETE ME - Custom wording override",
        "Should use the wording override, not master body"
    );

    // Cleanup
    cleanup_scope_assembly(&c, test_fee_key).await;
    cleanup_deliverable(&c, &del_id_1).await;
    cleanup_deliverable(&c, &del_id_2).await;
}

#[tokio::test]
async fn test_load_scope_nonexistent_returns_empty() {
    let c = client();

    // Load for a fee_id that has no saved assembly
    let resp = c
        .get(format!(
            "{}/scope/{}/deliverables",
            base_url(),
            "DELETE_ME_nonexistent_fee"
        ))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /scope/{fee_id}/deliverables failed");

    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["fee_id"], "DELETE_ME_nonexistent_fee");

    let deliverables_used = body["deliverables_used"]
        .as_array()
        .expect("deliverables_used should be an array");
    assert!(
        deliverables_used.is_empty(),
        "Non-existent fee should return empty deliverables_used"
    );
}

// ============================================================================
// Final cleanup
// ============================================================================

#[tokio::test]
async fn test_zz_final_cleanup() {
    let c = client();
    cleanup_all_test_deliverables(&c).await;

    // Also clean up test scope assemblies
    for fee_key in ["DELETE_ME_save_load_test", "DELETE_ME_overwrite_test"] {
        cleanup_scope_assembly(&c, fee_key).await;
    }
}

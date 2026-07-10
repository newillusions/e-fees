//! Integration tests for the clause selection endpoints (Stage 1).
//!
//! Tests POST /scope/clause-selection and GET /scope/{fee_id}/clause-selection.
//!
//! Requires the e-fees-scope service to be running.
//! Set `API_BASE_URL` and `API_KEY` environment variables.
//!
//! Uses fee key "delete-me-sel-test-001" — a fictional ID that will not
//! collide with real proposals; the created scope_assembly record is
//! idempotent (tests are safe to re-run).
//!
//! Run with: cargo test -p e-fees-scope --test clause_selection_tests -- --test-threads=1

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

/// Fictional fee key used across all selection tests (safe to reuse / re-run).
const TEST_FEE_KEY: &str = "delete-me-sel-test-001";

// ── Helper: create / archive a scratch clause (Stage 2 conditions test) ──────

/// Create a clause and return the full response body. Merges `extra` fields
/// (e.g. `is_default`, `conditions`) over the DELETE-ME-prefixed defaults.
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

/// Archive a scratch clause via the API (sufficient for test isolation).
async fn cleanup_clause(client: &Client, id: &str) {
    let _ = client
        .delete(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await;
}

// ── Helper: get first active clause id from the library ──────────────────────

async fn first_active_clause_id(c: &Client) -> Option<String> {
    let resp = c
        .get(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    body["data"]
        .as_array()?
        .first()
        .and_then(|c| c["id"].as_str())
        .map(str::to_string)
}

// ── Helper: fetch clause selection ───────────────────────────────────────────

async fn fetch_selection(c: &Client, fee_key: &str) -> Value {
    let resp = c
        .get(format!("{}/scope/{}/clause-selection", base_url(), fee_key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET clause-selection failed");
    assert_eq!(
        resp.status(),
        200,
        "GET clause-selection returned non-200 for fee_key={}",
        fee_key
    );
    resp.json::<Value>()
        .await
        .expect("Failed to parse clause-selection response")
}

/// Fetch clause selection with a `conditions` query param (Stage 2: conditional
/// clause defaults are gated by project conditions, same subset-match shape as
/// the deliverable engine's `condition_matches` in routes/assembly.rs).
async fn fetch_selection_with_conditions(c: &Client, fee_key: &str, conditions: &Value) -> Value {
    let resp = c
        .get(format!("{}/scope/{}/clause-selection", base_url(), fee_key))
        .header("X-API-Key", api_key())
        .query(&[("conditions", conditions.to_string())])
        .send()
        .await
        .expect("GET clause-selection (with conditions) failed");
    assert_eq!(
        resp.status(),
        200,
        "GET clause-selection with conditions returned non-200 for fee_key={}",
        fee_key
    );
    resp.json::<Value>()
        .await
        .expect("Failed to parse clause-selection response")
}

// ── Helper: save a clause selection ──────────────────────────────────────────

async fn save_selection(c: &Client, body: Value) -> Value {
    let resp = c
        .post(format!("{}/scope/clause-selection", base_url()))
        .header("X-API-Key", api_key())
        .json(&body)
        .send()
        .await
        .expect("POST clause-selection failed");
    assert!(
        resp.status().is_success(),
        "POST clause-selection failed with status {}",
        resp.status()
    );
    resp.json::<Value>()
        .await
        .expect("Failed to parse save response")
}

// ============================================================================
// TESTS
// ============================================================================

/// Calling GET without any prior POST pre-fills from each clause's `is_default`
/// flag, gated by `conditions` (Stage 2). With no `conditions` query param,
/// conditional clauses (conditions.is_some()) can never match, so they default
/// to excluded regardless of is_default; unconditional clauses default to their
/// is_default value.
#[tokio::test]
async fn test_get_clause_selection_default_prefills_from_is_default_and_conditions() {
    let c = client();

    // Fetch the master library to know each clause's is_default + conditions.
    let list_resp = c
        .get(format!("{}/clauses", base_url()))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET /clauses failed");
    assert!(list_resp.status().is_success());
    let list_body: Value = list_resp.json().await.expect("parse /clauses");
    let clauses = list_body["data"].as_array().expect("data must be array");
    assert!(!clauses.is_empty(), "need at least one active clause");

    let mut expected: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    for cj in clauses {
        let cid = cj["id"]
            .as_str()
            .unwrap()
            .trim_start_matches("clause:")
            .to_string();
        let is_default = cj["is_default"].as_bool().unwrap_or(false);
        let has_conditions = cj.get("conditions").map_or(false, |v| !v.is_null());
        // No conditions passed on this GET -> conditional clauses never match.
        expected.insert(cid, is_default && !has_conditions);
    }

    // Use a fresh fictional fee that has never had a selection saved
    let no_sel_key = "delete-me-no-sel-000";
    let body = fetch_selection(&c, no_sel_key).await;

    assert_eq!(
        body["fee_id"].as_str(),
        Some(no_sel_key),
        "fee_id must be echoed back"
    );
    assert_eq!(
        body["has_custom_selection"].as_bool(),
        Some(false),
        "has_custom_selection must be false when no selection was saved"
    );

    let selections = body["selections"]
        .as_array()
        .expect("selections must be an array");
    assert_eq!(
        selections.len(),
        clauses.len(),
        "selection count must match the active library count"
    );

    for sel in selections {
        let cid = sel["clause_id"].as_str().unwrap_or("").to_string();
        let expected_included = expected.get(&cid).copied().unwrap_or(false);
        assert_eq!(
            sel["included"].as_bool(),
            Some(expected_included),
            "clause '{}' default inclusion must follow is_default (+ conditions gate)",
            cid
        );
        assert!(sel["clause_id"].is_string(), "clause_id must be a string");
        assert!(sel["title"].is_string(), "title must be present");
        assert!(sel["category"].is_string(), "category must be present");
        assert!(sel["body"].is_string(), "body must be present");
    }
}

/// A default=true clause with a `conditions` object is excluded by default
/// (no project conditions known), included when the passed `conditions` query
/// param satisfies every key in the clause's condition object, and excluded
/// again when the passed conditions don't match. Mirrors the deliverable
/// engine's subset-match semantics (routes/assembly.rs::condition_matches).
#[tokio::test]
async fn test_get_clause_selection_conditional_default_gated_by_conditions() {
    let c = client();

    let created = create_test_clause(
        &c,
        Some(json!({
            "is_default": true,
            "conditions": { "min_area": 500, "project_type": "hospitality" }
        })),
    )
    .await;
    let clause_full_id = created["data"]["id"].as_str().unwrap().to_string();
    let clause_key = clause_full_id.trim_start_matches("clause:").to_string();

    let fee_key = "delete-me-cond-sel-000";

    // No conditions passed -> excluded despite is_default=true.
    let no_cond = fetch_selection(&c, fee_key).await;
    let sel_no_cond = no_cond["selections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(clause_key.as_str()))
        .expect("clause must appear in selection (no conditions)");
    assert_eq!(
        sel_no_cond["included"].as_bool(),
        Some(false),
        "conditional default clause must be excluded when no conditions are known"
    );

    // Matching conditions (subset match; extra request keys ignored) -> included.
    let matching = json!({ "min_area": 500, "project_type": "hospitality", "extra": "ignored" });
    let match_resp = fetch_selection_with_conditions(&c, fee_key, &matching).await;
    let sel_match = match_resp["selections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(clause_key.as_str()))
        .expect("clause must appear in selection (matching conditions)");
    assert_eq!(
        sel_match["included"].as_bool(),
        Some(true),
        "conditional default clause must be included when conditions match"
    );

    // Non-matching conditions -> excluded.
    let non_matching = json!({ "min_area": 200, "project_type": "residential" });
    let nomatch_resp = fetch_selection_with_conditions(&c, fee_key, &non_matching).await;
    let sel_nomatch = nomatch_resp["selections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(clause_key.as_str()))
        .expect("clause must appear in selection (non-matching conditions)");
    assert_eq!(
        sel_nomatch["included"].as_bool(),
        Some(false),
        "conditional default clause must be excluded when conditions don't match"
    );

    cleanup_clause(&c, &clause_key).await;
}

/// Malformed `conditions` query JSON is rejected with 400, not silently ignored.
#[tokio::test]
async fn test_get_clause_selection_rejects_invalid_conditions_json() {
    let resp = client()
        .get(format!(
            "{}/scope/{}/clause-selection",
            base_url(),
            TEST_FEE_KEY
        ))
        .header("X-API-Key", api_key())
        .query(&[("conditions", "not-json")])
        .send()
        .await
        .expect("GET with invalid conditions failed");

    assert_eq!(
        resp.status(),
        400,
        "invalid conditions JSON must be rejected with 400"
    );
}

/// POST saves a new selection and GET returns it with has_custom_selection=true.
#[tokio::test]
async fn test_save_and_get_clause_selection() {
    let c = client();

    // Grab the first two active clause ids
    let first_id = first_active_clause_id(&c)
        .await
        .expect("need at least one active clause in the library");

    let save_body = json!({
        "fee_id": TEST_FEE_KEY,
        "selections": [
            { "clause_id": first_id, "included": true, "override_body": null },
        ]
    });
    let save_resp = save_selection(&c, save_body).await;

    assert_eq!(save_resp["status"].as_str(), Some("saved"));
    assert_eq!(
        save_resp["fee_id"].as_str(),
        Some(TEST_FEE_KEY),
        "fee_id must be echoed"
    );
    assert_eq!(
        save_resp["selections_count"].as_i64(),
        Some(1),
        "selections_count must be 1"
    );
    assert_eq!(
        save_resp["included_count"].as_i64(),
        Some(1),
        "included_count must be 1"
    );

    // Verify GET reflects the saved state
    let get_resp = fetch_selection(&c, TEST_FEE_KEY).await;
    assert_eq!(
        get_resp["has_custom_selection"].as_bool(),
        Some(true),
        "has_custom_selection must be true after saving"
    );

    let selections = get_resp["selections"]
        .as_array()
        .expect("selections must be array");
    let saved = selections
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(first_id.trim_start_matches("clause:")))
        .expect("saved clause_id must appear in GET response");

    assert_eq!(
        saved["included"].as_bool(),
        Some(true),
        "saved clause must be included=true"
    );
}

/// POST with included=false makes that clause excluded in GET.
#[tokio::test]
async fn test_save_excluded_clause() {
    let c = client();

    let first_id = first_active_clause_id(&c)
        .await
        .expect("need at least one active clause");

    let save_body = json!({
        "fee_id": TEST_FEE_KEY,
        "selections": [
            { "clause_id": first_id, "included": false, "override_body": null },
        ]
    });
    save_selection(&c, save_body).await;

    let get_resp = fetch_selection(&c, TEST_FEE_KEY).await;
    let selections = get_resp["selections"].as_array().unwrap();
    let entry = selections
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(first_id.trim_start_matches("clause:")))
        .expect("clause must appear in GET even when excluded");

    assert_eq!(
        entry["included"].as_bool(),
        Some(false),
        "excluded clause must have included=false"
    );
}

/// override_body is structural: once set it is preserved in subsequent GETs.
#[tokio::test]
async fn test_override_body_is_structural_and_preserved() {
    let c = client();

    let first_id = first_active_clause_id(&c)
        .await
        .expect("need at least one active clause");
    let custom_text = "DELETE ME - Custom override body for testing.";

    let save_body = json!({
        "fee_id": TEST_FEE_KEY,
        "selections": [
            {
                "clause_id": first_id,
                "included": true,
                "override_body": custom_text
            },
        ]
    });
    save_selection(&c, save_body).await;

    let get_resp = fetch_selection(&c, TEST_FEE_KEY).await;
    let selections = get_resp["selections"].as_array().unwrap();
    let entry = selections
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(first_id.trim_start_matches("clause:")))
        .expect("clause must be in GET response");

    assert_eq!(
        entry["override_body"].as_str(),
        Some(custom_text),
        "override_body must be preserved exactly as stored"
    );
    assert_eq!(
        entry["included"].as_bool(),
        Some(true),
        "clause must remain included"
    );
}

/// Updating the selection replaces the previous one (idempotent).
#[tokio::test]
async fn test_save_overwrites_previous_selection() {
    let c = client();

    let first_id = first_active_clause_id(&c)
        .await
        .expect("need at least one active clause");

    // First save: included=true
    save_selection(
        &c,
        json!({
            "fee_id": TEST_FEE_KEY,
            "selections": [{ "clause_id": first_id, "included": true, "override_body": null }]
        }),
    )
    .await;

    // Second save: same clause now excluded
    save_selection(
        &c,
        json!({
            "fee_id": TEST_FEE_KEY,
            "selections": [{ "clause_id": first_id, "included": false, "override_body": null }]
        }),
    )
    .await;

    let get_resp = fetch_selection(&c, TEST_FEE_KEY).await;
    let selections = get_resp["selections"].as_array().unwrap();
    let entry = selections
        .iter()
        .find(|s| s["clause_id"].as_str() == Some(first_id.trim_start_matches("clause:")))
        .expect("clause must be in GET response");

    assert_eq!(
        entry["included"].as_bool(),
        Some(false),
        "second save must overwrite first — clause should now be excluded"
    );
}

/// Active clauses not in the saved selection default to included=false.
/// (They were added to the library after the selection was saved.)
#[tokio::test]
async fn test_new_library_clauses_default_to_excluded_in_saved_selection() {
    let c = client();

    // Save a selection that deliberately omits some clauses
    let first_id = first_active_clause_id(&c)
        .await
        .expect("need at least one active clause");

    save_selection(
        &c,
        json!({
            "fee_id": TEST_FEE_KEY,
            "selections": [{ "clause_id": first_id, "included": true, "override_body": null }]
        }),
    )
    .await;

    let get_resp = fetch_selection(&c, TEST_FEE_KEY).await;
    let selections = get_resp["selections"].as_array().unwrap();

    // Any clause NOT in the saved list should default to excluded
    for sel in selections {
        let cid = sel["clause_id"].as_str().unwrap_or("");
        let normalised_first = first_id.trim_start_matches("clause:");
        if cid != normalised_first {
            assert_eq!(
                sel["included"].as_bool(),
                Some(false),
                "clause '{}' not in saved selection should default to included=false",
                cid
            );
        }
    }
}

/// POST /scope/clause-selection requires authentication.
#[tokio::test]
async fn test_save_clause_selection_requires_auth() {
    let resp = client()
        .post(format!("{}/scope/clause-selection", base_url()))
        .json(&json!({
            "fee_id": TEST_FEE_KEY,
            "selections": [{ "clause_id": "abc", "included": true, "override_body": null }]
        }))
        .send()
        .await
        .expect("POST without auth failed");

    assert_eq!(resp.status(), 401, "should require authentication");
}

/// GET /scope/{fee_id}/clause-selection requires authentication.
#[tokio::test]
async fn test_get_clause_selection_requires_auth() {
    let resp = client()
        .get(format!(
            "{}/scope/{}/clause-selection",
            base_url(),
            TEST_FEE_KEY
        ))
        .send()
        .await
        .expect("GET without auth failed");

    assert_eq!(resp.status(), 401, "should require authentication");
}

/// POST rejects an empty selections array.
#[tokio::test]
async fn test_save_empty_selections_rejected() {
    let resp = client()
        .post(format!("{}/scope/clause-selection", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "fee_id": TEST_FEE_KEY,
            "selections": []
        }))
        .send()
        .await
        .expect("POST with empty selections failed");

    assert_eq!(
        resp.status(),
        400,
        "empty selections must be rejected with 400"
    );
}

/// POST rejects an empty fee_id.
#[tokio::test]
async fn test_save_empty_fee_id_rejected() {
    let first_id = first_active_clause_id(&client())
        .await
        .unwrap_or_else(|| "clause:abc".to_string());

    let resp = client()
        .post(format!("{}/scope/clause-selection", base_url()))
        .header("X-API-Key", api_key())
        .json(&json!({
            "fee_id": "",
            "selections": [{ "clause_id": first_id, "included": true, "override_body": null }]
        }))
        .send()
        .await
        .expect("POST with empty fee_id failed");

    assert_eq!(resp.status(), 400, "empty fee_id must be rejected with 400");
}

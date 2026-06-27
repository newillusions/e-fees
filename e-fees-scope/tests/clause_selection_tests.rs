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

/// Calling GET without any prior POST returns all active clauses with included=true.
/// This is the default "all included" state before a user customises their selection.
#[tokio::test]
async fn test_get_clause_selection_default_all_included() {
    let c = client();

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

    let selections = body["selections"].as_array().expect("selections must be an array");
    assert!(
        !selections.is_empty(),
        "should return at least one active clause"
    );
    for sel in selections {
        assert_eq!(
            sel["included"].as_bool(),
            Some(true),
            "all clauses should default to included=true when no selection saved"
        );
        assert!(sel["clause_id"].is_string(), "clause_id must be a string");
        assert!(sel["title"].is_string(), "title must be present");
        assert!(sel["category"].is_string(), "category must be present");
        assert!(sel["body"].is_string(), "body must be present");
    }
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

    let selections = get_resp["selections"].as_array().expect("selections must be array");
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
        .get(format!("{}/scope/{}/clause-selection", base_url(), TEST_FEE_KEY))
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

    assert_eq!(
        resp.status(),
        400,
        "empty fee_id must be rejected with 400"
    );
}

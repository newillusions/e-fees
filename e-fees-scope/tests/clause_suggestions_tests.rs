//! Integration tests for clause suggestions (Stage 3).
//!
//! Tests GET /scope/{fee_id}/clause-suggestions.
//!
//! Requires the e-fees-scope service to be running.
//! Set `API_BASE_URL` and `API_KEY` environment variables.
//!
//! NOTE on scope: this endpoint ranks unselected clauses by mined
//! clause_corpus_stat.usage_count and excludes clauses already included in
//! the fee's saved selection. It does NOT filter by discipline - the clause
//! library has no discipline-taxonomy field (clause.category is a
//! content-type grouping: Administrative/Commercial/Legal/Services;
//! clause.subcategory groups by document structure: Appointment/Areas/Design
//! Stages - neither corresponds to fee.pricing.disciplines[].id like "ld"/
//! "av"). See the PR body for detail on this design-doc deviation.
//!
//! Run with: cargo test -p e-fees-scope --test clause_suggestions_tests -- --test-threads=1

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

/// Fictional fee key used across all suggestion tests (safe to reuse / re-run).
const TEST_FEE_KEY: &str = "delete-me-suggest-test-001";

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

async fn cleanup_clause(client: &Client, id: &str) {
    let _ = client
        .delete(format!("{}/clauses/{}", base_url(), id))
        .header("X-API-Key", api_key())
        .send()
        .await;
}

async fn fetch_suggestions(c: &Client, fee_key: &str) -> Value {
    let resp = c
        .get(format!("{}/scope/{}/clause-suggestions", base_url(), fee_key))
        .header("X-API-Key", api_key())
        .send()
        .await
        .expect("GET clause-suggestions failed");
    assert_eq!(
        resp.status(),
        200,
        "GET clause-suggestions returned non-200 for fee_key={}",
        fee_key
    );
    resp.json::<Value>()
        .await
        .expect("Failed to parse clause-suggestions response")
}

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

/// A fee with no saved selection and (most likely) no mining data yet still
/// gets a 200 with an empty (or well-formed) suggestions array - never an
/// error just because clause_corpus_stat is unpopulated.
#[tokio::test]
async fn test_get_suggestions_returns_200_with_array_shape() {
    let c = client();
    let body = fetch_suggestions(&c, "delete-me-suggest-fresh-000").await;

    assert_eq!(body["fee_id"].as_str(), Some("delete-me-suggest-fresh-000"));
    assert!(
        body["suggestions"].is_array(),
        "suggestions must always be an array, even when unmined"
    );
}

/// GET /scope/{fee_id}/clause-suggestions requires authentication.
#[tokio::test]
async fn test_get_suggestions_requires_auth() {
    let resp = client()
        .get(format!(
            "{}/scope/{}/clause-suggestions",
            base_url(),
            TEST_FEE_KEY
        ))
        .send()
        .await
        .expect("GET without auth failed");

    assert_eq!(resp.status(), 401, "should require authentication");
}

/// A clause the fee has already included (included=true in the saved
/// selection) must never appear in its own suggestions list, even if it
/// somehow has mining data (defensive - the primary suppression is not
/// having stats for a scratch clause at all).
#[tokio::test]
async fn test_already_included_clause_is_excluded_from_suggestions() {
    let c = client();

    let created = create_test_clause(&c, Some(json!({ "is_default": false }))).await;
    let clause_full_id = created["data"]["id"].as_str().unwrap().to_string();
    let clause_key = clause_full_id.trim_start_matches("clause:").to_string();

    let fee_key = "delete-me-suggest-included-000";

    save_selection(
        &c,
        json!({
            "fee_id": fee_key,
            "selections": [{ "clause_id": clause_key, "included": true, "override_body": null }]
        }),
    )
    .await;

    let body = fetch_suggestions(&c, fee_key).await;
    let suggestions = body["suggestions"].as_array().unwrap();
    let found = suggestions
        .iter()
        .any(|s| s["clause_id"].as_str() == Some(clause_key.as_str()));
    assert!(
        !found,
        "an already-included clause must never appear in its own fee's suggestions"
    );

    cleanup_clause(&c, &clause_key).await;
}

/// Suggestions are ranked by usage_count descending when stats exist.
/// This test seeds two clauses with directly-written stat rows (via the
/// clause create + a raw stat insert is out of scope for an HTTP-only test
/// harness) - so instead it asserts the *contract*: whatever rows come back
/// are sorted descending by usage_count. Safe against an empty/unmined DB
/// (a zero- or one-element list is trivially sorted).
#[tokio::test]
async fn test_suggestions_are_ranked_by_usage_count_descending() {
    let c = client();
    let body = fetch_suggestions(&c, "delete-me-suggest-rank-000").await;
    let suggestions = body["suggestions"].as_array().unwrap();

    let counts: Vec<i64> = suggestions
        .iter()
        .map(|s| s["usage_count"].as_i64().unwrap_or(0))
        .collect();
    let mut sorted = counts.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(
        counts, sorted,
        "suggestions must be sorted by usage_count descending"
    );
}

/// Each suggestion entry carries the fields the frontend badge needs.
#[tokio::test]
async fn test_suggestion_entry_shape() {
    let c = client();
    let body = fetch_suggestions(&c, "delete-me-suggest-shape-000").await;
    let suggestions = body["suggestions"].as_array().unwrap();

    for s in suggestions {
        assert!(s["clause_id"].is_string(), "clause_id must be a string");
        assert!(s["title"].is_string(), "title must be present");
        assert!(s["category"].is_string(), "category must be present");
        assert!(s["usage_count"].is_i64(), "usage_count must be an integer");
        assert!(
            s["sample_project_numbers"].is_array(),
            "sample_project_numbers must be an array"
        );
        assert!(
            s["classified_at"].is_string(),
            "classified_at must be present"
        );
    }
}

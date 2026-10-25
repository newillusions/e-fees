//! `PUT /fees/{id}` body validation and update-plan building.
//!
//! The handler MERGEs the body into the `fee` record, so anything unchecked
//! lands in the table. The allowlist below mirrors the live `fee` table
//! (`INFO FOR TABLE fee`, prod and dev, 2026-10-25) plus the schemaless
//! fields the code and data-entry runs already write (`internal_note`,
//! `pricing`, `post_contract_items`, ...). Field types follow the table
//! definitions, not the strict e-fees-core structs, because real callers send
//! partial objects (e.g. a `revisions[]` entry without `author_email`).

use serde_json::{Map, Value};

use crate::error::ApiError;
use crate::validation::{validate_id, validate_status, FEE_STATUSES};

/// A validated PUT body, ready to turn into a query.
#[derive(Debug)]
pub struct FeeUpdatePlan {
    /// Fields MERGEd as-is (link fields and the computed `rev` removed).
    pub data: Map<String, Value>,
    /// New `company` record key, when relinking.
    pub company_id: Option<String>,
    /// New `contacts` record key, when relinking.
    pub contact_id: Option<String>,
}

fn bad(msg: impl Into<String>) -> ApiError {
    ApiError::bad_request(msg.into())
}

/// Accept a bare record key, or the same key with its table prefix.
fn link_key(field: &str, table: &str, value: &Value) -> Result<String, ApiError> {
    let raw = value
        .as_str()
        .ok_or_else(|| bad(format!("'{field}' must be a string record id")))?;
    let key = raw.strip_prefix(&format!("{table}:")).unwrap_or(raw);
    validate_id(key)?;
    Ok(key.to_string())
}

pub fn prepare_fee_update(body: Value) -> Result<FeeUpdatePlan, ApiError> {
    let Value::Object(obj) = body else {
        return Err(bad("Request body must be a JSON object"));
    };

    let mut data = Map::new();
    let mut company_id = None;
    let mut contact_id = None;

    for (key, value) in obj {
        if value.is_null() {
            return Err(bad(format!(
                "'{key}' must not be null (omit the field to leave it unchanged)"
            )));
        }
        match key.as_str() {
            // Free-text strings (table: TYPE string; internal_note is schemaless).
            "name" | "number" | "activity" | "package" | "staff_name" | "staff_email"
            | "staff_phone" | "staff_position" | "strap_line" | "internal_note" => {
                if !value.is_string() {
                    return Err(bad(format!("'{key}' must be a string")));
                }
                if key == "name" && value.as_str().is_some_and(|s| s.is_empty()) {
                    return Err(bad("'name' must not be empty"));
                }
                data.insert(key, value);
            }
            "issue_date" => {
                let ok = value
                    .as_str()
                    .is_some_and(|s| s.len() == 6 && s.chars().all(|c| c.is_ascii_digit()));
                if !ok {
                    return Err(bad("'issue_date' must be a 6-digit string (YYMMDD)"));
                }
                data.insert(key, value);
            }
            "status" => {
                let s = value
                    .as_str()
                    .ok_or_else(|| bad("'status' must be a string"))?;
                validate_status(s, FEE_STATUSES, "fee")?;
                data.insert(key, value);
            }
            "company_id" => company_id = Some(link_key("company_id", "company", &value)?),
            "contact_id" => contact_id = Some(link_key("contact_id", "contacts", &value)?),
            "revisions" | "post_contract_items" | "reimbursable_costs" | "pricing_revisions" => {
                match value.as_array() {
                    Some(items) if items.iter().all(Value::is_object) => {
                        data.insert(key, value);
                    }
                    _ => return Err(bad(format!("'{key}' must be an array of objects"))),
                }
            }
            "pricing" | "payment_schedule" => {
                if !value.is_object() {
                    return Err(bad(format!("'{key}' must be an object")));
                }
                data.insert(key, value);
            }
            "current_revision_number" | "current_release_number" => {
                if !value.is_i64() {
                    return Err(bad(format!("'{key}' must be an integer")));
                }
                data.insert(key, value);
            }
            "import_source" => {
                data.insert(key, value);
            }
            // `rev` is a VALUE field the DB recomputes from revisions[]; callers
            // that send it alongside a revisions[] update are harmless, so it
            // is accepted if an integer and dropped from the merge.
            "rev" => {
                if !value.is_i64() {
                    return Err(bad("'rev' must be an integer"));
                }
            }
            "id" | "time" | "project_id" => {
                return Err(bad(format!("'{key}' cannot be changed via PUT /fees")));
            }
            other => return Err(bad(format!("Unknown fee field '{other}'"))),
        }
    }

    Ok(FeeUpdatePlan {
        data,
        company_id,
        contact_id,
    })
}

/// Markers THROWn by the guards; mapped back to HTTP errors by
/// [`classify_update_errors`].
const FEE_MISSING: &str = "efees:fee_not_found";
const COMPANY_MISSING: &str = "efees:company_not_found";
const CONTACT_MISSING: &str = "efees:contact_not_found";

/// Build the update query for `fee:{key}` (key already validated): one
/// transaction that checks the fee and any relink targets exist, MERGEs, relinks
/// and stamps `updated_at`, so a failed guard or statement rolls everything
/// back. Returns the query and the result slot of the final `SELECT`
/// (slot 0 is BEGIN; every guard and statement takes one slot).
pub fn build_fee_update_query(key: &str, plan: &FeeUpdatePlan) -> (String, usize) {
    let mut stmts: Vec<String> = Vec::new();

    stmts.push(format!(
        "IF array::len(SELECT VALUE id FROM fee:{key}) = 0 {{ THROW '{FEE_MISSING}'; }};"
    ));
    if plan.company_id.is_some() {
        stmts.push(format!(
            "IF array::len(SELECT VALUE id FROM type::record('company', $company_id)) = 0 {{ THROW '{COMPANY_MISSING}'; }};"
        ));
    }
    if plan.contact_id.is_some() {
        stmts.push(format!(
            "IF array::len(SELECT VALUE id FROM type::record('contacts', $contact_id)) = 0 {{ THROW '{CONTACT_MISSING}'; }};"
        ));
    }
    stmts.push(format!("UPDATE fee:{key} MERGE $data RETURN NONE;"));
    if plan.company_id.is_some() {
        stmts.push(format!(
            "UPDATE fee:{key} SET company_id = type::record('company', $company_id) RETURN NONE;"
        ));
    }
    if plan.contact_id.is_some() {
        stmts.push(format!(
            "UPDATE fee:{key} SET contact_id = type::record('contacts', $contact_id) RETURN NONE;"
        ));
    }
    stmts.push(format!(
        "UPDATE fee:{key} SET time.updated_at = time::now() RETURN NONE;"
    ));
    // Slot 0 is BEGIN, so the SELECT (pushed next) lands at 1 + stmts.len().
    let select_slot = 1 + stmts.len();
    stmts.push(format!("SELECT * FROM fee:{key};"));

    (
        format!("BEGIN TRANSACTION; {} COMMIT TRANSACTION;", stmts.join(" ")),
        select_slot,
    )
}

/// Turn the per-statement error messages of a failed transaction into one
/// ApiError. A failed statement makes every later statement (and COMMIT)
/// report a cancelled/not-executed echo; those are skipped in favour of the
/// statement that actually failed.
pub fn classify_update_errors(key: &str, messages: &[String]) -> ApiError {
    let has = |m: &str| messages.iter().any(|e| e.contains(m));
    if has(FEE_MISSING) {
        return ApiError::not_found("Fee", key);
    }
    if has(COMPANY_MISSING) {
        return bad("company_id does not refer to an existing company");
    }
    if has(CONTACT_MISSING) {
        return bad("contact_id does not refer to an existing contact");
    }
    let root = messages
        .iter()
        .find(|e| !e.contains("cancelled transaction") && !e.contains("Cannot COMMIT"))
        .or(messages.first());
    ApiError {
        status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        code: "database_error".into(),
        message: root.cloned().unwrap_or_default(),
    }
}

/// Run a validated update against the database. `Ok(None)` only if the fee
/// vanished between the guard and the SELECT (treated as 404 by the caller).
pub async fn apply_fee_update(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    key: &str,
    plan: FeeUpdatePlan,
) -> Result<Option<e_fees_core::models::Fee>, ApiError> {
    let (query, select_slot) = build_fee_update_query(key, &plan);
    let mut qb = db.query(&query).bind(("data", Value::Object(plan.data)));
    if let Some(company_id) = plan.company_id {
        qb = qb.bind(("company_id", company_id));
    }
    if let Some(contact_id) = plan.contact_id {
        qb = qb.bind(("contact_id", contact_id));
    }
    let mut response = qb.await?;
    let errors = response.take_errors();
    if !errors.is_empty() {
        let mut ordered: Vec<(usize, String)> = errors
            .into_iter()
            .map(|(i, e)| (i, e.to_string()))
            .collect();
        ordered.sort_by_key(|(i, _)| *i);
        let messages: Vec<String> = ordered.into_iter().map(|(_, m)| m).collect();
        return Err(classify_update_errors(key, &messages));
    }
    Ok(response.take(select_slot)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ok(v: Value) -> FeeUpdatePlan {
        prepare_fee_update(v).expect("body should be accepted")
    }

    fn rejected(v: Value) -> bool {
        prepare_fee_update(v).is_err()
    }

    #[test]
    fn accepts_partial_known_fields() {
        let plan = ok(json!({
            "status": "Sent",
            "activity": "Updated Lighting Design",
            "current_revision_number": 2,
            "post_contract_items": [{
                "id": "p1", "stage_id": "s1", "description": "d",
                "quantity": 2, "unit": "hr", "rate": 100, "amount": 200
            }]
        }));
        assert_eq!(plan.data.len(), 4);
    }

    #[test]
    fn accepts_internal_note() {
        let plan = ok(json!({ "internal_note": "OWNER FLOOR - INTERNAL ONLY" }));
        assert_eq!(plan.data["internal_note"], "OWNER FLOOR - INTERNAL ONLY");
        assert!(!rejected(json!({ "internal_note": "x" })));
        assert!(rejected(json!({ "internal_note": 5 })));
    }

    // Payload shapes from ~/.claude/state/returns/efees-rit-record.md
    // (fee:26_97112_1 issue) and efees-lulu-award.md / ROUND 4 (fee
    // :26_97113_0 in-place update, then award). The returns record the field
    // values, not the literal request bodies; these are rebuilt from them.
    #[test]
    fn rit_issue_payload_passes() {
        let plan = ok(json!({
            "number": "26-97112-FP-01",
            "rev": 1,
            "status": "Sent",
            "issue_date": "261023",
            "name": "Fee Proposal",
            "activity": "Specialist Lighting Design and Consultancy",
            "package": "Stage Lighting and Stage Lighting Control Systems",
            "staff_name": "Martin Robert",
            "staff_email": "martin@emittiv.com",
            "staff_phone": "+971 5858 555 69",
            "staff_position": "Lighting Director",
            "strap_line": "sensory design studio",
            "pricing": { "config": { "currency": "AED", "quoted_fee": 140000 } },
            "revisions": [{
                "revision_number": 1,
                "revision_date": "2026-10-23",
                "notes": "Issued to WADG Consultants (ref 26-3032) by email"
            }],
            "internal_note": "OWNER FLOOR - INTERNAL ONLY ..."
        }));
        assert!(!plan.data.contains_key("rev"), "rev is DB-computed");
        assert_eq!(plan.data["pricing"]["config"]["quoted_fee"], 140000);
        assert!(plan.company_id.is_none() && plan.contact_id.is_none());
    }

    #[test]
    fn lulu_sphere_in_place_update_payload_passes() {
        ok(json!({
            "number": "26-97113-FP-01",
            "rev": 1,
            "status": "Sent",
            "package": "Lighting and Lighting Control",
            "pricing": { "config": { "currency": "AED", "quoted_fee": 35000 } },
            "revisions": [{
                "revision_number": 1,
                "revision_date": "2026-10-23",
                "notes": "Issued by email as 26-97113-FP-01 r01: lump sum AED 35,000."
            }],
            "internal_note": "Negotiation floor AED 30,000 (owner)."
        }));
    }

    #[test]
    fn lulu_award_payload_passes() {
        let plan = ok(json!({
            "status": "Accepted",
            "internal_note": "Negotiation floor ... | AWARDED 2026-10-24: PO-00396"
        }));
        assert_eq!(plan.data["status"], "Accepted");
    }

    #[test]
    fn relinking_company_and_contact_uses_string_keys() {
        let plan = ok(json!({
            "company_id": "t9j22b2b6jt6iggc16f2",
            "contact_id": "contacts:337uzkskfneokcpuzpjz"
        }));
        assert_eq!(plan.company_id.as_deref(), Some("t9j22b2b6jt6iggc16f2"));
        assert_eq!(plan.contact_id.as_deref(), Some("337uzkskfneokcpuzpjz"));
        assert!(!plan.data.contains_key("company_id"));
        assert!(!plan.data.contains_key("contact_id"));

        let (q, idx) = build_fee_update_query("26_97113_0", &plan);
        assert!(
            q.contains("company_id = type::record('company', $company_id)"),
            "{q}"
        );
        assert!(
            q.contains("contact_id = type::record('contacts', $contact_id)"),
            "{q}"
        );
        // BEGIN, 3 guards (fee, company, contact), MERGE, 2 relinks, updated_at, SELECT.
        assert_eq!(idx, 8);
        assert!(q.contains("SELECT * FROM fee:26_97113_0; COMMIT TRANSACTION;"));
    }

    #[test]
    fn link_keys_cannot_inject() {
        assert!(rejected(json!({ "company_id": "x'); DELETE fee;--" })));
        assert!(rejected(json!({ "contact_id": 5 })));
        assert!(rejected(json!({ "company_id": "" })));
    }

    #[test]
    fn query_without_links_has_only_merge_and_timestamp() {
        let plan = ok(json!({ "status": "Sent" }));
        let (q, idx) = build_fee_update_query("k1", &plan);
        assert!(!q.contains("type::record"));
        // BEGIN, fee guard, MERGE, updated_at, SELECT.
        assert_eq!(idx, 4);
    }

    #[test]
    fn rejects_non_object_body() {
        assert!(rejected(json!([1, 2])));
        assert!(rejected(json!("x")));
    }

    #[test]
    fn rejects_identity_fields() {
        for key in ["id", "time", "project_id"] {
            assert!(rejected(json!({ key: "x" })), "{key}");
        }
        assert!(rejected(json!({ "rev": "2" })));
    }

    #[test]
    fn rejects_unknown_fields() {
        assert!(rejected(json!({ "bogus": 1 })));
    }

    #[test]
    fn rejects_null_values() {
        assert!(rejected(json!({ "pricing": null })));
        assert!(rejected(json!({ "internal_note": null })));
        assert!(rejected(json!({ "company_id": null })));
    }

    #[test]
    fn rejects_wrong_types_and_values() {
        assert!(rejected(json!({ "name": 5 })));
        assert!(rejected(json!({ "name": "" })));
        assert!(rejected(json!({ "status": 5 })));
        assert!(rejected(json!({ "status": "Bogus" })));
        assert!(rejected(json!({ "issue_date": "26-10-23" })));
        assert!(rejected(json!({ "current_revision_number": "2" })));
        assert!(rejected(json!({ "pricing": "oops" })));
        assert!(rejected(json!({ "revisions": { "a": 1 } })));
        assert!(rejected(json!({ "revisions": [1, 2] })));
    }

    #[test]
    fn update_is_one_transaction_with_guards_before_writes() {
        let plan = ok(json!({
            "internal_note": "n",
            "company_id": "c1",
            "contact_id": "k1"
        }));
        let (q, _) = build_fee_update_query("fee1", &plan);
        assert!(q.starts_with("BEGIN TRANSACTION;"), "{q}");
        assert!(q.trim_end().ends_with("COMMIT TRANSACTION;"), "{q}");
        let merge = q.find("MERGE $data").unwrap();
        for guard in [FEE_MISSING, COMPANY_MISSING, CONTACT_MISSING] {
            assert!(
                q.find(guard).expect(guard) < merge,
                "{guard} must precede the writes: {q}"
            );
        }
        let relink = q.find("SET company_id").unwrap();
        assert!(merge < relink);
    }

    fn msgs(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn missing_company_or_contact_is_400_and_missing_fee_is_404() {
        let echo = "The query was not executed due to a cancelled transaction";
        let commit = "Cannot COMMIT: the transaction was aborted due to a prior error";

        let e = classify_update_errors(
            "f1",
            &msgs(&[
                echo,
                "An error occurred: efees:company_not_found",
                echo,
                commit,
            ]),
        );
        assert_eq!(e.status, axum::http::StatusCode::BAD_REQUEST);
        assert!(e.message.contains("company"));

        let e = classify_update_errors(
            "f1",
            &msgs(&["An error occurred: efees:contact_not_found", echo, commit]),
        );
        assert_eq!(e.status, axum::http::StatusCode::BAD_REQUEST);
        assert!(e.message.contains("contact"));

        let e = classify_update_errors(
            "f1",
            &msgs(&["An error occurred: efees:fee_not_found", echo]),
        );
        assert_eq!(e.status, axum::http::StatusCode::NOT_FOUND);
    }

    #[test]
    fn other_failures_report_the_failing_statement_not_the_rollback_echo() {
        let echo = "The query was not executed due to a cancelled transaction";
        let e = classify_update_errors(
            "f1",
            &msgs(&[
                echo,
                "Found 'x' for field `status`, but expected a string",
                echo,
            ]),
        );
        assert_eq!(e.status, axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        assert!(e.message.contains("field `status`"), "{}", e.message);
    }

    // --- live dev-DB tests (ignored): run with
    // creds-run.sh EFEES_SURREALDB_USER EFEES_SURREALDB_PASS -- cargo test -p e-fees-api --bins -- --ignored fee_update_live --test-threads=1
    async fn dev_db() -> surrealdb::Surreal<surrealdb::engine::remote::ws::Client> {
        use surrealdb::engine::remote::ws::Ws;
        use surrealdb::opt::auth::Root;
        let db = surrealdb::Surreal::new::<Ws>("10.0.23.12:8000")
            .await
            .unwrap();
        db.signin(Root {
            username: std::env::var("EFEES_SURREALDB_USER").expect("EFEES_SURREALDB_USER"),
            password: std::env::var("EFEES_SURREALDB_PASS").expect("EFEES_SURREALDB_PASS"),
        })
        .await
        .unwrap();
        db.use_ns("emittiv").use_db("projects").await.unwrap();
        db
    }

    async fn seed_fee(db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>, key: &str) {
        db.query(format!(
            "CREATE fee:{key} SET name = 'DELETE ME live test', number = 'x', status = 'Draft', \
             issue_date = '261025', activity = 'a', package = 'p', staff_name = '', staff_email = '', \
             staff_phone = '', staff_position = '', strap_line = '', revisions = [], \
             project_id = type::record('projects', 'zz_live_{key}'), \
             company_id = type::record('company', 'zz_live_{key}'), \
             contact_id = type::record('contacts', 'zz_live_{key}'), \
             time = {{ created_at: time::now(), updated_at: time::now() }};"
        ))
        .await
        .unwrap()
        .check()
        .unwrap();
    }

    async fn fee_row(
        db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
        key: &str,
    ) -> Option<Value> {
        let mut r = db
            .query(format!(
                "SELECT internal_note, <string>company_id AS company_id FROM fee:{key};"
            ))
            .await
            .unwrap();
        let rows: Vec<Value> = r.take(0).unwrap();
        rows.into_iter().next()
    }

    #[tokio::test]
    #[ignore]
    async fn fee_update_live_relink_validates_and_rolls_back() {
        let db = dev_db().await;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let key = format!("zz_live_{stamp}");
        seed_fee(&db, &key).await;

        // (1) relink to a company that does not exist: 400, and the MERGE
        // that came first in the body is rolled back too.
        let plan = ok(json!({
            "internal_note": "should be rolled back",
            "company_id": format!("no_such_company_{stamp}")
        }));
        let err = apply_fee_update(&db, &key, plan).await.unwrap_err();
        let row = fee_row(&db, &key).await.unwrap();

        // (2) relink to an existing company: applied together with the merge.
        let plan2 = ok(json!({ "internal_note": "applied", "company_id": "AFN" }));
        let ok2 = apply_fee_update(&db, &key, plan2).await;
        let row2 = fee_row(&db, &key).await.unwrap();

        // (3) unknown fee: 404 and no record created.
        let missing = format!("zz_live_missing_{stamp}");
        let err3 = apply_fee_update(&db, &missing, ok(json!({ "internal_note": "x" })))
            .await
            .unwrap_err();
        let created = fee_row(&db, &missing).await;

        let _ = db
            .query(format!("DELETE fee:{key}; DELETE fee:{missing};"))
            .await;

        assert_eq!(
            err.status,
            axum::http::StatusCode::BAD_REQUEST,
            "{}",
            err.message
        );
        assert!(
            row["internal_note"].is_null(),
            "merge must roll back: {row}"
        );
        assert!(
            row["company_id"].as_str().unwrap().contains("zz_live_"),
            "{row}"
        );

        assert!(ok2.unwrap().is_some());
        assert_eq!(row2["internal_note"], "applied");
        assert!(
            row2["company_id"].as_str().unwrap().contains("AFN"),
            "{row2}"
        );

        assert_eq!(err3.status, axum::http::StatusCode::NOT_FOUND);
        assert!(
            created.is_none(),
            "unknown fee must not be created: {created:?}"
        );
    }

    // --- HTTP level: drives the real handler through an axum router. The 400
    // paths are rejected before any database access, so the Surreal handle is
    // an unconnected placeholder.
    async fn put_fee(path: &str, body: Value) -> (axum::http::StatusCode, Value) {
        use axum::body::Body;
        use axum::http::{header, Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let state = std::sync::Arc::new(crate::AppState {
            db: surrealdb::Surreal::init(),
            api_keys: Default::default(),
            folder_config: None,
            started_at: std::time::Instant::now(),
        });
        let app = axum::Router::new()
            .route(
                "/fees/{id}",
                axum::routing::put(crate::routes::fees::update_fee),
            )
            .with_state(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    #[tokio::test]
    async fn http_put_fee_with_immutable_field_is_400() {
        let (status, body) = put_fee("/fees/26_97113_0", json!({ "project_id": "x" })).await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "bad_request");
        assert!(body["message"].as_str().unwrap().contains("project_id"));
    }

    #[tokio::test]
    async fn http_put_fee_with_null_or_unknown_field_is_400() {
        let (status, _) = put_fee("/fees/26_97113_0", json!({ "internal_note": null })).await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
        let (status, _) = put_fee("/fees/26_97113_0", json!({ "bogus": 1 })).await;
        assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    }
}

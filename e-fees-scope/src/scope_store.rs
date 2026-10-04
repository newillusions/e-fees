//! Write helpers for `scope_assembly` / `scope_revision`.
//!
//! Every write to a fee's `scope_assembly` goes through one of the routes in
//! `routes::scope`, `routes::assembly` or `routes::clause_selection`. Those
//! routes own *different* fields of the same record, so none of them may
//! DELETE + CREATE it: that silently drops whatever the other routes wrote
//! (the 2026-10-04 single-writer audit found `generate` and
//! `save_scope_builder` doing exactly that). The helpers here update the
//! record in place with a single `UPSERT ... WHERE fee_id = ...` statement
//! (create if absent, merge-by-field if present), so each caller touches only
//! the fields it owns.
//!
//! Field ownership (what each write path is allowed to change):
//!
//! | field                                            | generate | save builder | clause selection |
//! |--------------------------------------------------|----------|--------------|------------------|
//! | clauses, generated_text, numbering, llm_polished, |   set    |  preserved   |    preserved     |
//! |   llm_model, stages_snapshot                     |          |              |                  |
//! | deliverables_used                                | preserved|     set      |    preserved     |
//! | stage_labels, manual_items                       | preserved| set if sent  |    preserved     |
//! | selected_clauses                                 | preserved|  preserved   |       set        |
//! | current_revision                                 | preserved (only `create_revision` moves it) |
//!
//! Generic over the SurrealDB connection so the same code runs against the
//! WebSocket client in the service and an embedded in-memory engine in tests.

use serde_json::Value;
use surrealdb::{Connection, Surreal};

use crate::error::ApiError;
use crate::models::ScopeAssembly;

/// How many times `create_revision` re-reads the highest stored revision and
/// tries again when writers race for the same number (the UNIQUE index on
/// `(fee_id, revision)` rejects the losers). Each loser waits a short jittered
/// interval first so a burst of writers spreads out instead of colliding again.
const REVISION_RETRIES: usize = 8;

/// A lost race for a revision number: either the UNIQUE index rejected the
/// duplicate, or the engine aborted the transaction on a write conflict (its
/// message says the transaction "can be retried").
fn is_revision_race(message: &str) -> bool {
    message.contains("already contains")
        || message.contains("idx_scope_rev_fee")
        || message.contains("can be retried")
}

/// Jittered backoff before retry number `attempt` (1-based): 5-25ms base plus
/// up to 25ms of jitter, growing with the attempt.
fn revision_backoff(attempt: usize) -> std::time::Duration {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0) as u64;
    std::time::Duration::from_millis(5 * attempt.min(5) as u64 + nanos % 25)
}

/// Fields owned by the generate path.
pub struct GeneratedScope {
    pub clauses: Value,
    pub generated_text: String,
    pub numbering: Value,
    pub llm_polished: bool,
    pub stages_snapshot: Vec<String>,
    /// `None` clears `llm_model` (unpolished text has no model).
    pub llm_model: Option<String>,
}

/// Fields owned by the scope-builder save path.
pub struct BuilderState {
    pub deliverables_used: Value,
    /// `None` means "not sent": the stored value is kept. `Some` replaces it
    /// (an empty object/array is an explicit clear).
    pub stage_labels: Option<Value>,
    pub manual_items: Option<Value>,
}

/// Highest revision number for `fee_key`, from the stored counter and from
/// the stored `scope_revision` rows, so a reset or stale counter can never
/// hand out a number that is already in use.
async fn highest_revision<C: Connection>(
    db: &Surreal<C>,
    fee_key: &str,
    counter: i64,
) -> Result<i64, ApiError> {
    let mut res = db
        .query(
            "SELECT VALUE revision FROM scope_revision \
             WHERE fee_id = type::record('fee', $fee_key) \
             ORDER BY revision DESC LIMIT 1",
        )
        .bind(("fee_key", fee_key.to_string()))
        .await?;
    let top: Vec<i64> = res.take(0)?;
    Ok(top.into_iter().next().unwrap_or(0).max(counter))
}

/// Snapshot the current assembly as the next `scope_revision` and advance
/// `scope_assembly.current_revision`. Returns the revision number written, or
/// 0 when the fee has no assembly yet (nothing to snapshot).
///
/// The revision number is monotonic: `max(counter, highest stored) + 1`. The
/// revision row and the counter move in one transaction.
pub async fn create_revision<C: Connection>(
    db: &Surreal<C>,
    fee_key: &str,
    trigger: &str,
) -> Result<i64, ApiError> {
    for attempt in 1..=REVISION_RETRIES {
        let mut res = db
            .query("SELECT * FROM scope_assembly WHERE fee_id = type::record('fee', $fee_key)")
            .bind(("fee_key", fee_key.to_string()))
            .await?;
        let assemblies: Vec<ScopeAssembly> = res.take(0)?;

        let Some(assembly) = assemblies.into_iter().next() else {
            return Ok(0); // No existing assembly to revision
        };

        let next_rev = highest_revision(db, fee_key, assembly.current_revision).await? + 1;

        // Store clauses as JSON string (audit snapshot - not queried by field)
        let clauses_json =
            serde_json::to_string(&e_fees_core::models::dbvalue_to_json(&assembly.clauses))
                .unwrap_or_else(|_| "[]".to_string());

        let write = db
            .query(
                "BEGIN TRANSACTION; \
                 CREATE scope_revision SET \
                   fee_id = type::record('fee', $fee_key), \
                   revision = $revision, \
                   clauses = $clauses_str, \
                   generated_text = $generated_text, \
                   stages_at_time = $stages, \
                   trigger = $trigger; \
                 UPDATE scope_assembly SET current_revision = $revision \
                   WHERE fee_id = type::record('fee', $fee_key); \
                 COMMIT TRANSACTION;",
            )
            .bind(("fee_key", fee_key.to_string()))
            .bind(("revision", next_rev))
            .bind(("clauses_str", clauses_json))
            .bind(("generated_text", assembly.generated_text.clone()))
            .bind((
                "stages",
                assembly.stages_snapshot.clone().unwrap_or_default(),
            ))
            .bind(("trigger", trigger.to_string()))
            .await;

        let outcome = match write {
            Ok(response) => response.check().map(|_| ()),
            Err(e) => Err(e),
        };

        match outcome {
            Ok(()) => return Ok(next_rev),
            Err(e) if attempt < REVISION_RETRIES && is_revision_race(&e.to_string()) => {
                tracing::warn!(
                    "scope revision {} for fee_key={} was taken concurrently, retrying: {}",
                    next_rev,
                    fee_key,
                    e
                );
                tokio::time::sleep(revision_backoff(attempt)).await;
            }
            Err(e) => {
                tracing::warn!("Failed to create scope revision: {}", e);
                return Err(e.into());
            }
        }
    }
    unreachable!("the final attempt always returns")
}

/// Write a freshly generated scope into the fee's assembly, in place.
///
/// Sets only the generate-owned fields; `selected_clauses`,
/// `deliverables_used`, `stage_labels`, `manual_items` and `current_revision`
/// are left exactly as stored. Creates the record when none exists.
pub async fn save_generated<C: Connection>(
    db: &Surreal<C>,
    fee_key: &str,
    g: GeneratedScope,
) -> Result<Option<ScopeAssembly>, ApiError> {
    let llm_model_set = if g.llm_model.is_some() {
        "$llm_model"
    } else {
        "NONE"
    };
    let query = format!(
        "UPSERT scope_assembly SET \
           fee_id = type::record('fee', $fee_key), \
           clauses = $clauses, \
           generated_text = $generated_text, \
           numbering = $numbering, \
           llm_polished = $llm_polished, \
           stages_snapshot = $stages_snapshot, \
           llm_model = {llm_model_set}, \
           updated_at = time::now() \
         WHERE fee_id = type::record('fee', $fee_key)"
    );

    let mut q = db
        .query(&query)
        .bind(("fee_key", fee_key.to_string()))
        .bind(("clauses", g.clauses))
        .bind(("generated_text", g.generated_text))
        .bind(("numbering", g.numbering))
        .bind(("llm_polished", g.llm_polished))
        .bind(("stages_snapshot", g.stages_snapshot));
    if let Some(model) = g.llm_model {
        q = q.bind(("llm_model", model));
    }

    let mut response = q.await?.check()?;
    let rows: Vec<ScopeAssembly> = response.take(0)?;
    Ok(rows.into_iter().next())
}

/// Save the scope builder's deliverables into the fee's assembly, in place.
///
/// Always sets `deliverables_used`. `stage_labels` / `manual_items` are
/// replaced only when the caller sent them. Everything else on the record
/// (generated text, clauses, clause selection, revision counter) is kept; a
/// brand-new record starts with empty `clauses` / `generated_text`.
pub async fn save_builder<C: Connection>(
    db: &Surreal<C>,
    fee_key: &str,
    b: BuilderState,
) -> Result<(), ApiError> {
    let mut optional_sets = String::new();
    if b.stage_labels.is_some() {
        optional_sets.push_str(", stage_labels = $stage_labels");
    }
    if b.manual_items.is_some() {
        optional_sets.push_str(", manual_items = $manual_items");
    }

    // `clauses ?? []` / `generated_text ?? ''` keep the stored value on an
    // existing record and satisfy the non-optional schema fields on a new one.
    let query = format!(
        "UPSERT scope_assembly SET \
           fee_id = type::record('fee', $fee_key), \
           deliverables_used = $deliverables_used, \
           clauses = clauses ?? [], \
           generated_text = generated_text ?? '', \
           updated_at = time::now(){optional_sets} \
         WHERE fee_id = type::record('fee', $fee_key)"
    );

    let mut q = db
        .query(&query)
        .bind(("fee_key", fee_key.to_string()))
        .bind(("deliverables_used", b.deliverables_used));
    if let Some(labels) = b.stage_labels {
        q = q.bind(("stage_labels", labels));
    }
    if let Some(manual) = b.manual_items {
        q = q.bind(("manual_items", manual));
    }

    q.await?.check().map_err(|e| {
        tracing::error!("save_builder: UPSERT failed for fee_key={}: {}", fee_key, e);
        ApiError::internal(format!("Failed to save scope assembly: {e}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use surrealdb::engine::local::{Db, Mem};

    const SCHEMA: &str = include_str!("../schema.surql");
    const MIGRATION_V009: &str =
        include_str!("../../scripts/migration/v009_scope_revision_unique.surql");
    const MIGRATION_V009_ROLLBACK: &str =
        include_str!("../../scripts/migration/v009_scope_revision_unique_rollback.surql");

    /// schema.surql with the explicit `scope_assembly` wildcard-element fields
    /// (`x.*`) moved to directly after the table definition. The embedded engine creates an implicit
    /// non-FLEXIBLE `x.*` when it sees `x`, which makes the later explicit FLEXIBLE
    /// definition fail with "already exists"; defining the wildcard first avoids it.
    fn schema_wildcards_first() -> String {
        let is_wildcard = |l: &str| {
            let l = l.trim_start();
            l.starts_with("DEFINE FIELD") && l.contains(".*") && l.contains("ON scope_assembly")
        };
        let wildcards: Vec<&str> = SCHEMA.lines().filter(|l| is_wildcard(l)).collect();
        let mut out = Vec::new();
        for line in SCHEMA.lines().filter(|l| !is_wildcard(l)) {
            out.push(line);
            if line.trim() == "DEFINE TABLE scope_assembly SCHEMAFULL;" {
                out.extend(&wildcards);
            }
        }
        out.join("\n")
    }

    /// Fresh in-memory database with the scope schema applied.
    async fn test_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("t").use_db("t").await.unwrap();
        let mut res = db.query(schema_wildcards_first()).await.unwrap();
        for (idx, e) in res.take_errors() {
            // The nested `clauses.*.clauses.*` hits the same implicit-field overlap.
            assert!(
                e.to_string().contains("already exists"),
                "schema statement {idx} failed: {e}"
            );
        }
        db.query("DEFINE TABLE app_state SCHEMALESS;")
            .await
            .unwrap()
            .check()
            .unwrap();
        db
    }

    async fn run(db: &Surreal<Db>, sql: &str) {
        let mut res = db.query(sql).await.unwrap();
        let errors = res.take_errors();
        assert!(
            errors.is_empty(),
            "statement errors (index -> error): {errors:?}"
        );
    }

    async fn assembly(db: &Surreal<Db>, fee: &str) -> Value {
        let mut res = db
            .query("SELECT * OMIT id FROM scope_assembly WHERE fee_id = type::record('fee', $k)")
            .bind(("k", fee.to_string()))
            .await
            .unwrap();
        let rows: Vec<Value> = res.take(0).unwrap();
        assert_eq!(rows.len(), 1, "exactly one scope_assembly for {fee}");
        rows.into_iter().next().unwrap()
    }

    async fn revisions(db: &Surreal<Db>, fee: &str) -> Vec<i64> {
        let mut res = db
            .query(
                "SELECT VALUE revision FROM scope_revision \
                 WHERE fee_id = type::record('fee', $k) ORDER BY revision ASC",
            )
            .bind(("k", fee.to_string()))
            .await
            .unwrap();
        res.take(0).unwrap()
    }

    /// An assembly carrying every user-curated field, at revision counter 2.
    async fn seed_curated(db: &Surreal<Db>, fee: &str) {
        run(
            db,
            &format!(
                "CREATE scope_assembly SET \
                   fee_id = type::record('fee', '{fee}'), \
                   clauses = [{{title: 'old clause'}}], \
                   generated_text = 'old text', \
                   selected_clauses = [{{clause_id: 'c1', included: true, override_body: 'custom'}}], \
                   deliverables_used = [{{deliverable_id: 'd1', stage: 'A'}}], \
                   stage_labels = {{A: 'Concept'}}, \
                   manual_items = [{{title: 'one-off'}}], \
                   current_revision = 2; \
                 CREATE scope_revision SET fee_id = type::record('fee', '{fee}'), revision = 1, \
                   clauses = '[]', generated_text = 'r1', trigger = 't'; \
                 CREATE scope_revision SET fee_id = type::record('fee', '{fee}'), revision = 2, \
                   clauses = '[]', generated_text = 'r2', trigger = 't';"
            ),
        )
        .await;
    }

    fn generated(text: &str, model: Option<&str>) -> GeneratedScope {
        GeneratedScope {
            clauses: json!([{ "title": "new clause" }]),
            generated_text: text.to_string(),
            numbering: json!({ "c1": "1.1" }),
            llm_polished: model.is_some(),
            stages_snapshot: vec!["Concept".to_string()],
            llm_model: model.map(str::to_string),
        }
    }

    fn assert_curated_fields_intact(row: &Value) {
        assert_eq!(
            row["selected_clauses"],
            json!([{ "clause_id": "c1", "included": true, "override_body": "custom" }])
        );
        assert_eq!(
            row["deliverables_used"],
            json!([{ "deliverable_id": "d1", "stage": "A" }])
        );
        assert_eq!(row["stage_labels"], json!({ "A": "Concept" }));
        assert_eq!(row["manual_items"], json!([{ "title": "one-off" }]));
    }

    // (a) generate preserves the four user-curated fields and the counter.
    #[tokio::test]
    async fn generate_preserves_user_curated_fields() {
        let db = test_db().await;
        seed_curated(&db, "f1").await;

        let rev = create_revision(&db, "f1", "regeneration").await.unwrap();
        assert_eq!(rev, 3);
        let saved = save_generated(&db, "f1", generated("new text", Some("m1")))
            .await
            .unwrap()
            .expect("assembly returned");
        assert_eq!(saved.generated_text, "new text");

        let row = assembly(&db, "f1").await;
        assert_curated_fields_intact(&row);
        assert_eq!(row["generated_text"], "new text");
        assert_eq!(row["llm_model"], "m1");
        assert_eq!(
            row["current_revision"], 3,
            "generate must not move the counter itself"
        );
        assert_eq!(revisions(&db, "f1").await, vec![1, 2, 3]);

        // Unpolished regeneration clears the model rather than keeping a stale one.
        save_generated(&db, "f1", generated("plain", None))
            .await
            .unwrap();
        let row = assembly(&db, "f1").await;
        assert!(row.get("llm_model").is_none_or(Value::is_null));
        assert_curated_fields_intact(&row);
    }

    #[tokio::test]
    async fn generate_creates_the_assembly_when_none_exists() {
        let db = test_db().await;
        let saved = save_generated(&db, "fresh", generated("t", None))
            .await
            .unwrap()
            .expect("created");
        assert_eq!(saved.current_revision, 0);
        assert_eq!(create_revision(&db, "fresh", "x").await.unwrap(), 1);
        assert_eq!(create_revision(&db, "nobody", "x").await.unwrap(), 0);
    }

    // (b) save_scope_builder keeps the curated fields and never resets the counter.
    #[tokio::test]
    async fn save_builder_preserves_fields_and_keeps_revision_counter() {
        let db = test_db().await;
        seed_curated(&db, "f2").await;

        save_builder(
            &db,
            "f2",
            BuilderState {
                deliverables_used: json!([{ "deliverable_id": "d2", "stage": "B" }]),
                stage_labels: None,
                manual_items: None,
            },
        )
        .await
        .unwrap();

        let row = assembly(&db, "f2").await;
        // replaced: the builder owns deliverables_used
        assert_eq!(
            row["deliverables_used"],
            json!([{ "deliverable_id": "d2", "stage": "B" }])
        );
        // preserved: not sent, or owned by other routes
        assert_eq!(row["stage_labels"], json!({ "A": "Concept" }));
        assert_eq!(row["manual_items"], json!([{ "title": "one-off" }]));
        assert_eq!(
            row["selected_clauses"],
            json!([{ "clause_id": "c1", "included": true, "override_body": "custom" }])
        );
        assert_eq!(row["generated_text"], "old text");
        assert_eq!(row["clauses"], json!([{ "title": "old clause" }]));
        assert_eq!(
            row["current_revision"], 2,
            "save must not reset the counter"
        );

        // The next revision continues the sequence instead of re-issuing 1.
        assert_eq!(create_revision(&db, "f2", "manual_edit").await.unwrap(), 3);
        assert_eq!(revisions(&db, "f2").await, vec![1, 2, 3]);
        assert_eq!(assembly(&db, "f2").await["current_revision"], 3);
    }

    #[tokio::test]
    async fn save_builder_replaces_stage_labels_and_manual_items_only_when_sent() {
        let db = test_db().await;
        seed_curated(&db, "f3").await;

        save_builder(
            &db,
            "f3",
            BuilderState {
                deliverables_used: json!([]),
                stage_labels: Some(json!({ "A": "Renamed" })),
                manual_items: Some(json!([])),
            },
        )
        .await
        .unwrap();

        let row = assembly(&db, "f3").await;
        assert_eq!(row["stage_labels"], json!({ "A": "Renamed" }));
        assert_eq!(
            row["manual_items"],
            json!([]),
            "an explicit empty list clears"
        );
    }

    #[tokio::test]
    async fn save_builder_creates_a_valid_assembly_when_none_exists() {
        let db = test_db().await;
        save_builder(
            &db,
            "new-fee",
            BuilderState {
                deliverables_used: json!([{ "deliverable_id": "d1" }]),
                stage_labels: None,
                manual_items: None,
            },
        )
        .await
        .unwrap();
        let row = assembly(&db, "new-fee").await;
        assert_eq!(row["clauses"], json!([]));
        assert_eq!(row["generated_text"], "");
        assert_eq!(row["current_revision"], 0);
    }

    // Legacy data: the counter was reset to 0 while revisions 1..2 exist. The
    // next number must come from the stored rows, not the counter.
    #[tokio::test]
    async fn create_revision_never_reissues_a_stored_number() {
        let db = test_db().await;
        seed_curated(&db, "f4").await;
        run(&db, "UPDATE scope_assembly SET current_revision = 0;").await;

        assert_eq!(create_revision(&db, "f4", "x").await.unwrap(), 3);
        assert_eq!(create_revision(&db, "f4", "x").await.unwrap(), 4);
        assert_eq!(revisions(&db, "f4").await, vec![1, 2, 3, 4]);
    }

    // (c) the schema (and therefore a migrated database) rejects a duplicate number.
    #[tokio::test]
    async fn duplicate_revision_insert_is_rejected() {
        let db = test_db().await;
        seed_curated(&db, "f5").await;
        let dup = db
            .query(
                "CREATE scope_revision SET fee_id = type::record('fee', 'f5'), revision = 2, \
                 clauses = '[]', generated_text = 'dup', trigger = 't';",
            )
            .await
            .unwrap()
            .check();
        let err = dup.expect_err("duplicate (fee_id, revision) must be rejected");
        assert!(err.to_string().contains("already contains"), "{err}");
        // The same number on a different fee is fine.
        run(
            &db,
            "CREATE scope_revision SET fee_id = type::record('fee', 'other'), revision = 2, \
             clauses = '[]', generated_text = 'ok', trigger = 't';",
        )
        .await;
    }

    // Writers racing for the same number: every one must end up with its own
    // contiguous revision and the counter must equal the highest.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_create_revision_yields_distinct_contiguous_numbers() {
        let db = test_db().await;
        seed_curated(&db, "race").await;

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let db = db.clone();
                tokio::spawn(async move { create_revision(&db, "race", "x").await })
            })
            .collect();
        let mut got = Vec::new();
        for h in handles {
            got.push(h.await.unwrap().expect("retry absorbs the race"));
        }
        got.sort_unstable();
        assert_eq!(got, vec![3, 4, 5, 6]);
        assert_eq!(revisions(&db, "race").await, vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(assembly(&db, "race").await["current_revision"], 6);
    }

    #[test]
    fn revision_race_errors_are_recognised() {
        assert!(is_revision_race(
            "Database index `idx_scope_rev_fee` already contains [fee:a, 1], with record `scope_revision:x`"
        ));
        assert!(is_revision_race(
            "Query not executed: Transaction conflict: Write conflict, retry the transaction. This transaction can be retried"
        ));
        assert!(!is_revision_race(
            "The table 'scope_revision' does not exist"
        ));
    }

    #[test]
    fn revision_backoff_is_short_and_bounded() {
        for attempt in 1..=REVISION_RETRIES {
            let d = revision_backoff(attempt);
            assert!(d.as_millis() < 60, "{d:?}");
        }
    }

    /// Statement errors of a script, as (index -> message), without panicking.
    async fn run_collecting_errors(db: &Surreal<Db>, sql: &str) -> Vec<String> {
        let mut res = db.query(sql).await.unwrap();
        let mut errs: Vec<_> = res.take_errors().into_iter().collect();
        errs.sort_by_key(|(i, _)| *i);
        errs.into_iter().map(|(_, e)| e.to_string()).collect()
    }

    async fn stamp(db: &Surreal<Db>) -> Vec<String> {
        let mut res = db
            .query("SELECT VALUE value FROM app_state WHERE key = 'schema_version'")
            .await
            .unwrap();
        res.take(0).unwrap()
    }

    async fn rev_index_def(db: &Surreal<Db>) -> String {
        let mut res = db.query("INFO FOR TABLE scope_revision").await.unwrap();
        let info: Value = res.take::<Option<Value>>(0).unwrap().unwrap();
        info["indexes"]["idx_scope_rev_fee"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    // apply-migration.sh proves the rollback in a fresh namespace holding only
    // app_state (stamped 8): forward, rollback, forward must all run clean.
    #[tokio::test]
    async fn migration_runs_clean_in_an_empty_namespace_forward_rollback_forward() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("t").use_db("t").await.unwrap();
        run(
            &db,
            "CREATE app_state SET key = 'schema_version', value = '8', updated_at = time::now();",
        )
        .await;

        run(&db, MIGRATION_V009).await;
        assert_eq!(stamp(&db).await, vec!["9".to_string()]);
        assert!(rev_index_def(&db).await.contains("UNIQUE"));

        run(&db, MIGRATION_V009_ROLLBACK).await;
        assert_eq!(stamp(&db).await, vec!["8".to_string()]);
        assert!(!rev_index_def(&db).await.contains("UNIQUE"));

        run(&db, MIGRATION_V009).await;
        assert_eq!(stamp(&db).await, vec!["9".to_string()]);
        assert!(rev_index_def(&db).await.contains("UNIQUE"));
    }

    // THROW does not halt the script, so the stamp must not depend on it: if a
    // duplicate appears after the renumbering (a concurrent write), the index
    // is not created and the version must stay 8.
    #[tokio::test]
    async fn surviving_duplicate_leaves_the_index_and_the_stamp_untouched() {
        let db = test_db().await;
        downgrade_to_pre_v009(&db).await;
        seed_duplicates(&db).await;
        run(
            &db,
            "CREATE app_state SET key = 'schema_version', value = '8', updated_at = time::now();",
        )
        .await;

        // Inject a duplicate right after the renumbering pass.
        let injected = MIGRATION_V009.replacen(
            "LET $still_duplicated",
            "CREATE scope_revision SET fee_id = type::record('fee', 'b'), revision = 1, \
             clauses = '[]', generated_text = 'late', trigger = 't';\nLET $still_duplicated",
            1,
        );
        assert_ne!(injected, MIGRATION_V009, "injection point exists");

        let errors = run_collecting_errors(&db, &injected).await;
        assert!(
            errors
                .iter()
                .any(|e| e.contains("schema_version NOT stamped")),
            "{errors:?}"
        );
        assert_eq!(stamp(&db).await, vec!["8".to_string()]);
        assert!(!rev_index_def(&db).await.contains("UNIQUE"));
    }

    /// The pre-v009 shape: non-unique index, no audit field.
    async fn downgrade_to_pre_v009(db: &Surreal<Db>) {
        run(
            db,
            "REMOVE INDEX idx_scope_rev_fee ON scope_revision; \
             DEFINE INDEX idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision;",
        )
        .await;
    }

    async fn seed_duplicates(db: &Surreal<Db>) {
        // fee a: counter was reset, so 1,2 were issued again (and a third 1).
        // fee b: clean. fee c: revisions but no assembly.
        let rev = |id: &str, fee: &str, n: i64, day: u32| {
            format!(
                "CREATE scope_revision:{id} SET fee_id = type::record('fee', '{fee}'), \
                 revision = {n}, clauses = '[]', generated_text = '{id}', trigger = 't', \
                 saved_at = <datetime>'2026-01-0{day}T00:00:00Z';"
            )
        };
        let sql = [
            "CREATE scope_assembly SET fee_id = type::record('fee', 'a'), clauses = [], generated_text = '', current_revision = 2;".to_string(),
            "CREATE scope_assembly SET fee_id = type::record('fee', 'b'), clauses = [], generated_text = '', current_revision = 1;".to_string(),
            rev("a1", "a", 1, 1),
            rev("a2", "a", 2, 2),
            rev("a3", "a", 1, 3),
            rev("a4", "a", 2, 4),
            rev("a5", "a", 1, 5),
            rev("b1", "b", 1, 1),
            rev("c1", "c", 1, 1),
        ]
        .join("\n");
        run(db, &sql).await;
    }

    // (d) migration copes with pre-existing duplicates, deterministically.
    #[tokio::test]
    async fn migration_renumbers_existing_duplicates_then_enforces_uniqueness() {
        let db = test_db().await;
        downgrade_to_pre_v009(&db).await;
        seed_duplicates(&db).await;
        assert_eq!(revisions(&db, "a").await, vec![1, 1, 1, 2, 2]);

        run(&db, MIGRATION_V009).await;

        // Earliest row per number keeps it; later ones move to highest+1 in
        // (revision, saved_at, id) order: a3(1)->3, a5(1)->4, a4(2)->5.
        let mut res = db
            .query("SELECT meta::id(id) AS id, revision, renumbered_from FROM scope_revision WHERE fee_id = type::record('fee', 'a') ORDER BY revision")
            .await
            .unwrap();
        let rows: Vec<Value> = res.take(0).unwrap();
        let got: Vec<(String, i64, Value)> = rows
            .iter()
            .map(|r| {
                (
                    r["id"].as_str().unwrap().to_string(),
                    r["revision"].as_i64().unwrap(),
                    r["renumbered_from"].clone(),
                )
            })
            .collect();
        assert_eq!(
            got,
            vec![
                ("a1".to_string(), 1, Value::Null),
                ("a2".to_string(), 2, Value::Null),
                ("a3".to_string(), 3, json!(1)),
                ("a5".to_string(), 4, json!(1)),
                ("a4".to_string(), 5, json!(2)),
            ]
        );
        // Clean fees untouched; counters raised to the highest stored number.
        assert_eq!(revisions(&db, "b").await, vec![1]);
        assert_eq!(revisions(&db, "c").await, vec![1]);
        assert_eq!(assembly(&db, "a").await["current_revision"], 5);
        assert_eq!(assembly(&db, "b").await["current_revision"], 1);

        // Uniqueness now enforced.
        let dup = db
            .query(
                "CREATE scope_revision SET fee_id = type::record('fee', 'b'), revision = 1, \
                 clauses = '[]', generated_text = 'x', trigger = 't';",
            )
            .await
            .unwrap()
            .check();
        assert!(
            dup.is_err(),
            "unique index must be live after the migration"
        );

        // Re-running changes nothing.
        run(&db, MIGRATION_V009).await;
        assert_eq!(revisions(&db, "a").await, vec![1, 2, 3, 4, 5]);

        // Schema version stamped.
        let mut res = db
            .query("SELECT VALUE value FROM app_state WHERE key = 'schema_version'")
            .await
            .unwrap();
        let v: Vec<String> = res.take(0).unwrap();
        assert_eq!(v, vec!["9".to_string()]);
    }

    #[tokio::test]
    async fn migration_on_clean_data_is_a_no_op_and_rollback_restores_numbers() {
        let db = test_db().await;
        downgrade_to_pre_v009(&db).await;
        seed_curated(&db, "clean").await;
        run(&db, MIGRATION_V009).await;
        assert_eq!(revisions(&db, "clean").await, vec![1, 2]);

        // Rollback after a repair puts the original (duplicate) numbers back.
        let db = test_db().await;
        downgrade_to_pre_v009(&db).await;
        seed_duplicates(&db).await;
        run(&db, MIGRATION_V009).await;
        run(&db, MIGRATION_V009_ROLLBACK).await;
        assert_eq!(revisions(&db, "a").await, vec![1, 1, 1, 2, 2]);
    }
}

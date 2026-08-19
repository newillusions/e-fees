//! Integration tests for the P0 historical-backfill schema migration
//! (`scripts/migration/v006_backfill_metadata_fields.surql`).
//!
//! These tests exercise the new `projects` fields (`project_type`,
//! `project_category`, `disciplines[*]`, `stage`, `outcome`,
//! `outcome_changed_at`, `successor`, plus the carried-over
//! `gfa_sqm`/`plot_area_sqm`/`scope_summary`/`loss_reason`/`source_document`/
//! `data_provenance` fields) directly against the DEV SurrealDB
//! (`10.0.23.12:8000`, ns `emittiv_dev`, db `projects`) - never prod
//! (`10.0.23.11`).
//!
//! WHY THESE TESTS SHELL OUT TO `surql.sh` RATHER THAN CONNECTING VIA THE
//! `surrealdb` CRATE DIRECTLY: `~/.claude/scripts/surql.sh` is the workspace's
//! sanctioned, credential-safe path for ad hoc SurrealQL (see
//! `docs/development/DATABASE_SCHEMA.md` and the workspace KB,
//! `obs:j3d1jpord0g8w3lgmo75`) - it reads `EFEES_SURREALDB_USER`/`_PASS` from
//! `~/.claude/.credentials.env` itself and never prints them. Reimplementing
//! credential loading here would be a second, unreviewed path to the same
//! secret. These tests therefore have NO credential requirement of their own:
//! they only need `surql.sh` to exist at the fixed path below and dev to be
//! reachable.
//!
//! Run with: `cargo test -p e-fees-api --test p0_schema_migration_tests -- --test-threads=1`
//! (serialized: several tests share fixed test-project record keys and must
//! not interleave).
//!
//! Test data uses the `DELETE ME` prefix per `.claude/rules/development-workflow.md`
//! and fixed record keys in an unused "year 39 / country 0" number-space so
//! repeated runs are idempotent (each test deletes its own key first).

use serde_json::Value;
use std::process::Command;

const SURQL_SH: &str = "/Users/martin/.claude/scripts/surql.sh";
const DEV_HOST: &str = "10.0.23.12:8000";
const DEV_NS: &str = "emittiv_dev";
const DEV_DB: &str = "projects";
const CREDS_PREFIX: &str = "EFEES_SURREALDB";

/// A known-existing, pre-migration dev row (see the P0 dispatch's baseline
/// snapshot, `dev-projects-before-p0.json`) - used to prove existing rows
/// survive the migration untouched.
const EXISTING_ROW_ID: &str = "projects:`22_96601`";
const EXISTING_ROW_NAME: &str = "TEST_Dammam Entertainment Development";
const EXISTING_ROW_STATUS: &str = "Lost";

/// A real `company` row on dev, used as the `sub_company` link target.
const EXISTING_COMPANY_ID: &str = "company:AFN";

/// Run one or more SurrealQL statements against DEV via the sanctioned
/// `surql.sh` wrapper and return the parsed per-statement result array.
/// Panics (with full stdout/stderr) if the script fails to run at all -
/// per-statement `ERR` status is NOT a panic here, since several tests
/// assert on an expected `ERR`.
fn run_dev_query(query: &str) -> Vec<Value> {
    // Hard-coded dev-only endpoint - never derived from an env var, so no
    // misconfiguration can point this at prod. See test_production_safety_guard.
    let output = Command::new(SURQL_SH)
        .args([
            "--host",
            DEV_HOST,
            "--ns",
            DEV_NS,
            "--db",
            DEV_DB,
            "--creds",
            CREDS_PREFIX,
            query,
        ])
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn {}: {}", SURQL_SH, e));

    assert!(
        output.status.success(),
        "surql.sh exited non-zero.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("failed to parse surql.sh JSON output: {}\nraw: {}", e, stdout))
}

/// Best-effort delete of a test row - ignores whether it existed.
fn cleanup_row(id: &str) {
    let _ = run_dev_query(&format!("DELETE {};", id));
}

// ---------------------------------------------------------------------------
// Safety guard
// ---------------------------------------------------------------------------

#[test]
fn test_production_safety_guard() {
    assert!(
        !DEV_HOST.contains("10.0.23.11"),
        "REFUSING: DEV_HOST constant has been pointed at the PRODUCTION database"
    );
    assert_eq!(DEV_NS, "emittiv_dev", "DEV_NS must be the dev namespace, never `emittiv`");
}

// ---------------------------------------------------------------------------
// Migration shape: all P0 fields defined on `projects`
// ---------------------------------------------------------------------------

#[test]
fn test_migration_fields_defined() {
    let result = run_dev_query("INFO FOR TABLE projects;");
    let info = &result[0]["result"];
    assert_eq!(result[0]["status"], "OK", "INFO FOR TABLE projects failed: {:?}", result[0]);

    let fields_json = info["fields"].to_string();
    // NOTE: dev (SurrealDB 3.1.4) echoes nested array-object field keys in
    // `INFO FOR TABLE` using dot-star notation (`disciplines.*.kind`) even
    // though the DDL is written with bracket-star (`disciplines[*].kind`) -
    // both are accepted as DEFINE FIELD syntax, this is purely how the
    // server reports the definition back. Prod is 3.1.2 (a point release
    // behind) - this has NOT been re-verified against prod and should be
    // checked when the migration is applied there, per CLAUDE.md's standing
    // "don't assume schema/behavior parity" rule.
    for expected in [
        "project_type",
        "project_category",
        "\"disciplines\"",
        "disciplines.*.kind",
        "disciplines.*.sub_discipline",
        "disciplines.*.sub_company",
        "\"stage\"",
        "\"outcome\"",
        "outcome_changed_at",
        "\"successor\"",
        "gfa_sqm",
        "plot_area_sqm",
        "scope_summary",
        "loss_reason",
        "source_document",
        "data_provenance",
    ] {
        assert!(
            fields_json.contains(expected),
            "expected field '{}' not found in INFO FOR TABLE projects; fields: {}",
            expected,
            fields_json
        );
    }
}

// ---------------------------------------------------------------------------
// Existing rows survive the migration untouched
// ---------------------------------------------------------------------------

#[test]
fn test_existing_rows_still_load() {
    let result = run_dev_query(&format!("SELECT * FROM {};", EXISTING_ROW_ID));
    assert_eq!(result[0]["status"], "OK", "SELECT of pre-existing row failed: {:?}", result[0]);

    let rows = result[0]["result"].as_array().expect("expected array result");
    assert_eq!(rows.len(), 1, "expected exactly one row for {}", EXISTING_ROW_ID);
    let row = &rows[0];

    assert_eq!(row["name"], EXISTING_ROW_NAME);
    assert_eq!(row["status"], EXISTING_ROW_STATUS);

    // New P0 fields must be absent/NONE for a row nothing has backfilled yet -
    // SurrealDB omits unset `option<T>` fields from the returned object.
    for new_field in ["project_type", "project_category", "disciplines", "stage", "outcome"] {
        assert!(
            row.get(new_field).is_none() || row[new_field].is_null(),
            "pre-existing row unexpectedly has a value for new field '{}': {:?}",
            new_field,
            row.get(new_field)
        );
    }
}

// ---------------------------------------------------------------------------
// D2: disciplines[*] with kind=Sub + sub_discipline + sub_company round-trips
// ---------------------------------------------------------------------------

#[test]
fn test_sub_discipline_round_trip() {
    let id = "projects:`39_00099`";
    cleanup_row(id);

    let create = format!(
        r#"CREATE {id} SET
            name = "DELETE ME - P0 sub-discipline round-trip test",
            name_short = "DELETE ME P0 Sub",
            status = "RFP",
            area = "Test Area",
            city = "Test City",
            country = "Test Country",
            folder = "DELETE ME - P0 sub-discipline test folder",
            number = {{ year: 39, country: 0, seq: 99, id: "39-00099" }},
            disciplines = [{{ kind: "Sub", sub_discipline: "Acoustics", sub_company: {company} }}];
        SELECT * FROM {id};"#,
        id = id,
        company = EXISTING_COMPANY_ID
    );

    let result = run_dev_query(&create);
    assert_eq!(result[0]["status"], "OK", "CREATE failed: {:?}", result[0]);
    assert_eq!(result[1]["status"], "OK", "SELECT failed: {:?}", result[1]);

    let rows = result[1]["result"].as_array().expect("expected array result");
    assert_eq!(rows.len(), 1);
    let disciplines = rows[0]["disciplines"].as_array().expect("disciplines should be an array");
    assert_eq!(disciplines.len(), 1);
    assert_eq!(disciplines[0]["kind"], "Sub");
    assert_eq!(disciplines[0]["sub_discipline"], "Acoustics");
    let sub_company_str = disciplines[0]["sub_company"].to_string();
    assert!(
        sub_company_str.contains("AFN"),
        "sub_company did not round-trip to {}: {}",
        EXISTING_COMPANY_ID,
        sub_company_str
    );

    cleanup_row(id);
}

// ---------------------------------------------------------------------------
// D2: an out-of-enum disciplines[*].kind is rejected by the ASSERT
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_discipline_kind_rejected() {
    let id = "projects:`39_00098`";
    cleanup_row(id);

    let create = format!(
        r#"CREATE {id} SET
            name = "DELETE ME - P0 invalid discipline kind test",
            name_short = "DELETE ME P0 Bad Kind",
            status = "RFP",
            area = "Test Area",
            city = "Test City",
            country = "Test Country",
            folder = "DELETE ME - P0 invalid discipline kind folder",
            number = {{ year: 39, country: 0, seq: 98, id: "39-00098" }},
            disciplines = [{{ kind: "NotARealDiscipline" }}];
        SELECT count() FROM projects WHERE number.id = "39-00098" GROUP ALL;"#,
        id = id
    );

    let result = run_dev_query(&create);
    assert_eq!(
        result[0]["status"], "ERR",
        "expected CREATE with an invalid disciplines[*].kind to be rejected, got: {:?}",
        result[0]
    );

    let created = row_count_from_group_all(&result[1]) > 0;
    assert!(!created, "row should not exist after a rejected CREATE: {:?}", result[1]);

    cleanup_row(id);
}

/// Extract the `count` value from a `SELECT count() ... GROUP ALL` result.
/// `GROUP ALL` always returns exactly one aggregate row - even when the
/// count is zero - so "is the result array non-empty" is NOT a valid
/// zero-check; the actual `count` field must be read.
fn row_count_from_group_all(statement_result: &Value) -> u64 {
    statement_result["result"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row["count"].as_u64())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// D3: an out-of-enum outcome value is rejected by the ASSERT ("Open" is not
// a value - absence of `outcome` already means open, per the plan's D3).
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_outcome_rejected() {
    let id = "projects:`39_00097`";
    cleanup_row(id);

    let create = format!(
        r#"CREATE {id} SET
            name = "DELETE ME - P0 invalid outcome test",
            name_short = "DELETE ME P0 Bad Outcome",
            status = "RFP",
            area = "Test Area",
            city = "Test City",
            country = "Test Country",
            folder = "DELETE ME - P0 invalid outcome folder",
            number = {{ year: 39, country: 0, seq: 97, id: "39-00097" }},
            outcome = "Open";
        SELECT count() FROM projects WHERE number.id = "39-00097" GROUP ALL;"#,
        id = id
    );

    let result = run_dev_query(&create);
    assert_eq!(
        result[0]["status"], "ERR",
        "expected CREATE with outcome='Open' to be rejected, got: {:?}",
        result[0]
    );

    let created = row_count_from_group_all(&result[1]) > 0;
    assert!(!created, "row should not exist after a rejected CREATE: {:?}", result[1]);

    cleanup_row(id);
}

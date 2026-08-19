//! Integration tests for the workspace migration-guard onboarding
//! (`scripts/migration/000-bootstrap-schema-version-tracking.surql` +
//! `scripts/migration/v006_backfill_metadata_fields.surql`'s trailing stamp
//! block).
//!
//! Verifies `app_state.schema_version` exists and reads `'6'` against DEV
//! (`10.0.23.12:8000`, ns `emittiv_dev`, db `projects`) - never prod. Written
//! RED-first: at the point this file was authored, dev carried the P0
//! `projects` fields (applied by hand in the prior P0 session) but had NO
//! `app_state` table at all, so both tests below failed. They turn GREEN
//! only after the bootstrap file is hand-applied and
//! `apply-migration.sh --version 6` completes a real (non-dry-run) apply
//! against dev.
//!
//! Same `surql.sh`-shellout pattern as `p0_schema_migration_tests.rs` - see
//! that file's header comment for the full rationale (credential-safe path,
//! no direct `surrealdb` crate connection needed for a black-box read).
//!
//! Run with: `cargo test -p e-fees-api --test p0_schema_version_stamp_tests`

use serde_json::Value;
use std::process::Command;

const SURQL_SH: &str = "/Users/martin/.claude/scripts/surql.sh";
const DEV_HOST: &str = "10.0.23.12:8000";
const DEV_NS: &str = "emittiv_dev";
const DEV_DB: &str = "projects";
const CREDS_PREFIX: &str = "EFEES_SURREALDB";

fn run_dev_query(query: &str) -> Vec<Value> {
    // Hard-coded dev-only endpoint - never derived from an env var, so no
    // misconfiguration can point this at prod.
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
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "failed to parse surql.sh JSON output: {}\nraw: {}",
            e, stdout
        )
    })
}

#[test]
fn test_production_safety_guard() {
    assert!(
        !DEV_HOST.contains("10.0.23.11"),
        "REFUSING: DEV_HOST constant has been pointed at the PRODUCTION database"
    );
    assert_eq!(
        DEV_NS, "emittiv_dev",
        "DEV_NS must be the dev namespace, never `emittiv`"
    );
}

/// `app_state` exists, is queryable, and carries exactly one
/// `key = 'schema_version'` row - the bootstrap file's shape
/// (`DEFINE TABLE app_state SCHEMAFULL` with `key`/`value`/`updated_at`
/// fields and a unique index on `key`), matching the pa-core-rs
/// `v026_app_state.surql` convention this migration reused.
#[test]
fn test_app_state_schema_version_row_exists() {
    let result = run_dev_query("SELECT key, value FROM app_state WHERE key = 'schema_version';");
    assert_eq!(
        result[0]["status"], "OK",
        "SELECT against app_state failed - table not bootstrapped yet? {:?}",
        result[0]
    );

    let rows = result[0]["result"]
        .as_array()
        .unwrap_or_else(|| panic!("expected an array result, got: {:?}", result[0]["result"]));
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one schema_version row (unique index on `key`), got {}: {:?}",
        rows.len(),
        rows
    );
    assert_eq!(rows[0]["key"], "schema_version");
}

/// After the migration-guard onboarding's dev proof cycle (bootstrap at '5'
/// -> apply v006 -> rollback -> re-apply v006, per the dispatch's required
/// proof sequence), dev's `schema_version` must read back exactly `'6'` -
/// the guard re-reads this same value after every apply
/// (`apply-migration.sh`'s `schema_version_of` + final read-back) and this
/// test is the project-repo-side equivalent check.
#[test]
fn test_schema_version_reads_six_after_v006() {
    // `value` alone right after SELECT parses as the `SELECT VALUE <field>`
    // shorthand (a reserved-position keyword, not the column name) - confirmed
    // live: "Parse error: Unexpected token `an identifier`, expected FROM".
    // Backtick-escape the column to disambiguate.
    let result =
        run_dev_query("SELECT `value` FROM app_state WHERE key = 'schema_version' LIMIT 1;");
    assert_eq!(result[0]["status"], "OK", "query failed: {:?}", result[0]);

    let value = result[0]["result"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row["value"].as_str())
        .unwrap_or_else(|| panic!("no schema_version row found: {:?}", result[0]["result"]));

    assert_eq!(
        value, "6",
        "expected dev schema_version='6' (post v006 apply); got '{}' - either the bootstrap \
         hasn't run yet (expect '5' would also be wrong - bootstrap alone doesn't stamp '6') \
         or v006 hasn't been applied/re-applied via apply-migration.sh yet",
        value
    );
}

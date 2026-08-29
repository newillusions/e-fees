//! Retroactively seed `fee.revisions[]` on every row created before the
//! fee-revisions fix (2026-08-28). See
//! `crates/e-fees-core/src/backfill/fee_revisions_seed.rs` for the pure
//! planning logic (unit-tested there) and its module doc comment for the
//! full background - in short, every fee-construction call site wrote
//! `revisions: []` before this fix, so the DB-computed `rev` field always
//! evaluated to `0`, confirmed live on dev (100% of 30 sampled rows,
//! 2026-08-28).
//!
//! Usage:
//!   cargo run -p e-fees-core --bin backfill_fee_revisions_seed -- \
//!     --target dev [--apply] [--report-file PATH]
//!
//! `--target prod --apply` is REFUSED unless `--confirm-prod` is also
//! passed, since prod writes are orchestrator-owned by default (same gate
//! as `backfill_p1_index_load.rs`). Default is `--dry-run` (i.e. omitting
//! `--apply` never writes, regardless of `--target`).
//!
//! Credentials: `EFEES_SURREALDB_USER` / `EFEES_SURREALDB_PASS` env vars -
//! never read from a file directly, never printed.
//!
//! IDEMPOTENT BY CONSTRUCTION: `plan_row_backfill` returns `AlreadyPopulated`
//! (a no-op) for any row whose `revisions[]` is already non-empty, so
//! re-running this binary after a partial or full apply is always safe.

use std::env;
use std::fs;
use std::process::ExitCode;

use serde::Serialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb::Surreal;

use e_fees_core::backfill::fee_revisions_seed::{
    plan_row_backfill, FeeRowFacts, ProvenanceSupersededRevision, RowBackfillPlan,
};
use e_fees_core::backfill::p1_index_load::check_prod_apply_gate;
use e_fees_core::models::record_key_string;

struct TargetConfig {
    name: &'static str,
    host: &'static str,
    ns: &'static str,
    db: &'static str,
}

const DEV: TargetConfig = TargetConfig {
    name: "dev",
    host: "10.0.23.12:8000",
    ns: "emittiv_dev",
    db: "projects",
};
const PROD: TargetConfig = TargetConfig {
    name: "prod",
    host: "10.0.23.11:8000",
    ns: "emittiv",
    db: "projects",
};

struct Args {
    target: &'static TargetConfig,
    apply: bool,
    report_file: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut target: Option<&'static TargetConfig> = None;
    let mut apply = false;
    let mut confirm_prod = false;
    let mut report_file = None;

    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--target" => {
                let v = iter.next().ok_or("--target requires a value (dev|prod)")?;
                target = Some(match v.as_str() {
                    "dev" => &DEV,
                    "prod" => &PROD,
                    other => return Err(format!("--target must be dev or prod, got {other:?}")),
                });
            }
            "--apply" => apply = true,
            "--dry-run" => apply = false,
            "--confirm-prod" => confirm_prod = true,
            "--report-file" => {
                report_file = Some(iter.next().ok_or("--report-file requires a path")?);
            }
            other => return Err(format!("unrecognized argument: {other}")),
        }
    }

    let target = target.ok_or("--target dev|prod is required")?;
    check_prod_apply_gate(target.name, apply, confirm_prod)?;
    Ok(Args {
        target,
        apply,
        report_file,
    })
}

#[derive(Debug, SurrealValue)]
struct FeeBackfillRow {
    id: RecordId,
    rev_len: i64,
    superseded_revisions: Option<Vec<ProvenanceSupersededRevisionRow>>,
}

#[derive(Debug, Clone, SurrealValue)]
struct ProvenanceSupersededRevisionRow {
    rev: i64,
    amount: Option<f64>,
    currency: Option<String>,
}

impl From<ProvenanceSupersededRevisionRow> for ProvenanceSupersededRevision {
    fn from(r: ProvenanceSupersededRevisionRow) -> Self {
        ProvenanceSupersededRevision {
            rev: r.rev,
            amount: r.amount,
            currency: r.currency,
        }
    }
}

#[derive(Serialize)]
struct RowReport {
    record_key: String,
    action: String,
    revision_count: Option<usize>,
    reason: Option<String>,
}

#[derive(Serialize, Default)]
struct ReportCounts {
    total_rows: usize,
    already_populated: usize,
    seeded: usize,
    skipped: usize,
}

#[derive(Serialize)]
struct Report {
    target: String,
    apply: bool,
    backfilled_at: String,
    counts: ReportCounts,
    seeded: Vec<RowReport>,
    already_populated: Vec<RowReport>,
    skipped: Vec<RowReport>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let user = match env::var("EFEES_SURREALDB_USER") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: EFEES_SURREALDB_USER must be set");
            return ExitCode::FAILURE;
        }
    };
    let pass = match env::var("EFEES_SURREALDB_PASS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: EFEES_SURREALDB_PASS must be set");
            return ExitCode::FAILURE;
        }
    };

    let backfilled_at = chrono::Utc::now().to_rfc3339();

    eprintln!(
        "backfill_fee_revisions_seed: connecting to {} ({}/{}), mode={}, backfilled_at={}",
        args.target.host,
        args.target.ns,
        args.target.db,
        if args.apply { "APPLY" } else { "DRY-RUN" },
        backfilled_at,
    );

    let db: Surreal<Client> = match Surreal::new::<Ws>(args.target.host).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error connecting to {}: {e}", args.target.host);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = db
        .signin(Root {
            username: user,
            password: pass,
        })
        .await
    {
        eprintln!("error signing in: {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = db.use_ns(args.target.ns).use_db(args.target.db).await {
        eprintln!("error selecting ns/db: {e}");
        return ExitCode::FAILURE;
    }

    let rows: Vec<FeeBackfillRow> = match db
        .query(
            "SELECT id, array::len(revisions) AS rev_len, \
                    data_provenance.superseded_revisions AS superseded_revisions \
             FROM fee",
        )
        .await
        .and_then(|mut r| r.take::<Vec<FeeBackfillRow>>(0))
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("error reading fee: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut seeded = Vec::new();
    let mut already_populated = Vec::new();
    let mut skipped = Vec::new();

    for row in &rows {
        let record_key = record_key_string(&row.id.key);
        let facts = FeeRowFacts {
            record_key: record_key.clone(),
            existing_revisions_len: row.rev_len,
            superseded_revisions: row
                .superseded_revisions
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(ProvenanceSupersededRevision::from)
                .collect(),
        };

        match plan_row_backfill(&facts, &backfilled_at) {
            RowBackfillPlan::AlreadyPopulated => {
                already_populated.push(RowReport {
                    record_key,
                    action: "already_populated".to_string(),
                    revision_count: None,
                    reason: None,
                });
            }
            RowBackfillPlan::Skip { reason } => {
                skipped.push(RowReport {
                    record_key,
                    action: "skip".to_string(),
                    revision_count: None,
                    reason: Some(reason),
                });
            }
            RowBackfillPlan::Seed(revisions) => {
                if args.apply {
                    let fee_id = RecordId::new("fee", record_key.clone());
                    // MERGE with a JSON $data object, NOT `SET revisions =
                    // $revisions` with a native-bound Vec<Revision> - the
                    // SET form failed live on dev this session with
                    // "Couldn't coerce value for field `revisions.*`...
                    // Expected `object` but found `NONE`" regardless of
                    // whether `revisions` was bound as a native
                    // SurrealValue-derived Vec<struct> or a pre-serialized
                    // serde_json::Value; MERGE is the exact strategy
                    // `db/client.rs`'s working `update_fee` already uses for
                    // this same field (`UPDATE fee:{id} MERGE $data`).
                    let data = serde_json::json!({
                        "revisions": serde_json::to_value(&revisions).unwrap_or(serde_json::json!([])),
                    });
                    let result = db
                        .query("UPDATE $fee_id MERGE $data;")
                        .bind(("fee_id", fee_id))
                        .bind(("data", data))
                        .await
                        .and_then(|r| r.check());
                    if let Err(e) = result {
                        eprintln!("error updating fee:{record_key}: {e}");
                        skipped.push(RowReport {
                            record_key,
                            action: "error".to_string(),
                            revision_count: None,
                            reason: Some(e.to_string()),
                        });
                        continue;
                    }
                }
                seeded.push(RowReport {
                    record_key,
                    action: "seed".to_string(),
                    revision_count: Some(revisions.len()),
                    reason: None,
                });
            }
        }
    }

    let report = Report {
        target: args.target.name.to_string(),
        apply: args.apply,
        backfilled_at,
        counts: ReportCounts {
            total_rows: rows.len(),
            already_populated: already_populated.len(),
            seeded: seeded.len(),
            skipped: skipped.len(),
        },
        seeded,
        already_populated,
        skipped,
    };

    let json_report = serde_json::to_string_pretty(&report).unwrap();
    println!("{json_report}");

    if let Some(path) = &args.report_file {
        if let Err(e) = fs::write(path, &json_report) {
            eprintln!("warning: failed to write report file {path}: {e}");
        }
    }

    ExitCode::SUCCESS
}
